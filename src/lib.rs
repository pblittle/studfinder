use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

pub mod config;
pub mod db;
pub mod detector;
pub mod error;
pub mod image_processor;
pub mod scanner;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProcessorType {
    Scanner,
    Detector,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub export_format: ExportFormat,
    pub scan_quality: ScanQuality,
    pub processor_type: ProcessorType,
    pub confidence_threshold: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScanQuality {
    Fast,
    Balanced,
    Accurate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Piece {
    pub id: String,
    pub part_number: String,
    pub color: String,
    pub category: String,
    pub quantity: i32,
    pub confidence: f32,
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x {} {} ({}) [confidence: {:.1}%]",
            self.quantity,
            self.part_number,
            self.category,
            self.color,
            self.confidence * 100.0
        )
    }
}

pub struct StudFinder {
    config: Config,
    db: db::Database,
    processor: Box<dyn image_processor::ImageProcessor>,
}

impl StudFinder {
    pub fn new(config: Config) -> Result<Self> {
        let db = db::Database::new(&config.database_path)
            .context("Failed to initialize database")?;
        
        // Choose processor based on configuration
        let processor: Box<dyn image_processor::ImageProcessor> = match config.processor_type {
            ProcessorType::Scanner => Box::new(scanner::Scanner::new(config.scan_quality.clone())),
            ProcessorType::Detector => Box::new(detector::Detector::new(config.confidence_threshold)),
        };

        let finder = Self { config, db, processor };
        Ok(finder)
    }

    pub fn init(&self) -> Result<()> {
        debug!("Initializing StudFinder");
        self.db.init()
            .context("Failed to initialize database schema")?;
        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        debug!("Resetting StudFinder");
        self.db.reset()
            .context("Failed to reset database")?;
        self.init()?;
        Ok(())
    }

    pub fn ensure_initialized(&self) -> Result<()> {
        if !self.config.database_path.exists() {
            debug!("Database not found, initializing...");
            self.init()?;
        }
        Ok(())
    }

    pub async fn scan_image(&self, path: PathBuf) -> Result<Piece> {
        self.ensure_initialized()?;

        info!("Starting image scan for: {}", path.display());

        // Image processing in a blocking task
        let processor = self.processor.clone();
        let path_clone = path.clone();
        let pieces = tokio::task::spawn_blocking(move || {
            processor.process_image(&path_clone)
        }).await.context("Failed to spawn processing task")?
          .context("Failed to process image")?;

        if pieces.is_empty() {
            return Err(anyhow::anyhow!("No pieces detected in image"));
        }

        let piece = pieces.into_iter().next().unwrap();
        info!("Successfully detected piece: {}", piece);

        Ok(piece)
    }

    pub fn add_piece(&self, piece: Piece) -> Result<()> {
        self.db.add_piece(&piece)
            .context("Failed to add piece to database")
    }

    pub fn list_inventory(&self) -> Result<Vec<Piece>> {
        self.db.list_pieces()
    }

    pub fn export_inventory(&self, path: PathBuf) -> Result<()> {
        let pieces = self.list_inventory()?;
        match self.config.export_format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&pieces)?;
                std::fs::write(&path, json)
                    .context("Failed to write JSON export")?;
            }
            ExportFormat::Csv => {
                let mut output = String::new();
                output.push_str("id,part_number,color,category,quantity,confidence\n");
                for piece in pieces {
                    output.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        piece.id,
                        piece.part_number,
                        piece.color,
                        piece.category,
                        piece.quantity,
                        piece.confidence
                    ));
                }
                std::fs::write(&path, output)
                    .context("Failed to write CSV export")?;
            }
        }
        Ok(())
    }

    pub fn import_inventory(&self, path: PathBuf) -> Result<()> {
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = std::fs::read_to_string(&path)
                .context("Failed to read JSON import file")?;
            let pieces: Vec<Piece> = serde_json::from_str(&data)
                .context("Failed to parse JSON data")?;
            for piece in pieces {
                self.add_piece(piece)?;
            }
        } else {
            // Assume CSV
            let data = std::fs::read_to_string(&path)
                .context("Failed to read CSV import file")?;
            for line in data.lines().skip(1) { // Skip header
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() == 6 {
                    let piece = Piece {
                        id: fields[0].to_string(),
                        part_number: fields[1].to_string(),
                        color: fields[2].to_string(),
                        category: fields[3].to_string(),
                        quantity: fields[4].parse()
                            .context("Failed to parse quantity")?,
                        confidence: fields[5].parse()
                            .context("Failed to parse confidence")?,
                    };
                    self.add_piece(piece)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    fn create_test_config() -> Config {
        Config {
            database_path: PathBuf::from(":memory:"),
            export_format: ExportFormat::Json,
            scan_quality: ScanQuality::Fast,
            processor_type: ProcessorType::Scanner,
            confidence_threshold: 0.8,
        }
    }

    #[test]
    fn test_new_studfinder() {
        let result = StudFinder::new(create_test_config());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scan_workflow() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = Config {
            database_path: db_path,
            export_format: ExportFormat::Json,
            scan_quality: ScanQuality::Fast,
            processor_type: ProcessorType::Scanner,
            confidence_threshold: 0.8,
        };

        let finder = StudFinder::new(config).unwrap();
        finder.init().unwrap();

        // Create a test image
        let image_path = temp_dir.path().join("test.jpg");
        let mut img = image::RgbImage::new(200, 200);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgb([255, 0, 0]); // Pure red
        }
        img.save(&image_path).unwrap();

        // Test scanning
        let piece = finder.scan_image(image_path).await.unwrap();
        assert_eq!(piece.color, "Red");
        assert!(piece.confidence > 0.8);

        // Test inventory
        let pieces = finder.list_inventory().unwrap();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].color, "Red");
    }
}
