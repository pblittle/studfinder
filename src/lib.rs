use std::path::PathBuf;
use tracing::{debug, info};

// Re-export core types
pub mod core;
pub use core::*;

// Re-export processing types
pub mod processing;
pub use processing::*;

// Re-export storage types
pub mod storage;

// Keep error module at the top level
pub mod error;
use crate::error::{Result, StudFinderError};

pub struct StudFinder {
    config: Config,
    db: storage::Database,
    processor: Box<dyn processing::ImageProcessor>,
}

impl StudFinder {
    pub fn new(config: Config) -> Result<Self> {
        let db = storage::Database::new(&config.database_path)?;

        // Choose processor based on configuration
        let processor: Box<dyn processing::ImageProcessor> = match config.processor_type {
            ProcessorType::Scanner => {
                Box::new(processing::Scanner::new(config.scan_quality.clone()))
            }
            ProcessorType::Detector => {
                Box::new(processing::Detector::new(config.confidence_threshold))
            }
        };

        let finder = Self {
            config,
            db,
            processor,
        };
        Ok(finder)
    }

    pub fn init(&self) -> Result<()> {
        debug!("Initializing StudFinder");
        self.db.init()?;
        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        debug!("Resetting StudFinder");
        self.db.reset()?;
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
        let pieces = tokio::task::spawn_blocking(move || processor.process_image(&path_clone))
            .await
            .map_err(|_| StudFinderError::NoPiecesDetected)??;

        if pieces.is_empty() {
            return Err(StudFinderError::NoPiecesDetected);
        }

        let piece = pieces
            .into_iter()
            .next()
            .ok_or(StudFinderError::NoPiecesDetected)?;
        info!("Successfully detected piece: {}", piece);

        Ok(piece)
    }

    pub fn add_piece(&self, piece: Piece) -> Result<()> {
        self.db.add_piece(&piece)
    }

    pub fn list_inventory(&self) -> Result<Vec<Piece>> {
        self.db.list_pieces()
    }

    pub fn export_inventory(&self, path: PathBuf) -> Result<()> {
        let pieces = self.list_inventory()?;
        storage::export::ExportManager::export_inventory(&pieces, path, &self.config.export_format)
    }

    pub fn import_inventory(&self, path: PathBuf) -> Result<()> {
        let pieces = storage::export::ExportManager::import_inventory(path)?;
        for piece in pieces {
            self.add_piece(piece)?;
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
            *pixel = image::Rgb([
                255, 0, 0,
            ]); // Pure red
        }
        img.save(&image_path).unwrap();

        // Test scanning
        let piece = finder.scan_image(image_path).await.unwrap();
        assert_eq!(piece.color, "Red");
        assert!(piece.confidence > 0.8);

        // Add the piece to the inventory
        finder.add_piece(piece).unwrap();

        // Test inventory
        let pieces = finder.list_inventory().unwrap();
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].color, "Red");
    }
}
