use crate::{Piece, ScanQuality};
use crate::image_processor::ImageProcessor;
use crate::color_detector::{ColorDetector, ColorDetectorConfig, ColorStandard};
use crate::error::{Result, StudFinderError};
use image::{DynamicImage, GenericImageView};
use std::path::Path;
use tracing::{debug, info};
use uuid::Uuid;

/// Scanner implementation for LEGO piece identification
///
/// This implementation focuses on color detection and basic shape analysis
/// to identify LEGO pieces in images.
#[derive(Clone)]
pub struct Scanner {
    config: ScanConfig,
}

/// Configuration for the scanner
#[derive(Clone, Debug)]
struct ScanConfig {
    min_confidence: f32,
    min_region_size: u32,
    color_detector_config: ColorDetectorConfig,
}

impl Scanner {
    /// Create a new Scanner with the specified quality level
    ///
    /// # Arguments
    /// * `quality` - The scan quality level (Fast, Balanced, or Accurate)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use studfinder::{scanner::Scanner, ScanQuality};
    /// 
    /// let scanner = Scanner::new(ScanQuality::Balanced);
    /// ```
    #[must_use]
    pub fn new(quality: ScanQuality) -> Self {
        info!("Initializing scanner with quality: {:?}", quality);

        let config = match quality {
            ScanQuality::Fast => ScanConfig {
                min_confidence: 0.7,
                min_region_size: 50,
                color_detector_config: ColorDetectorConfig {
                    threshold: 0.6,
                    standard: ColorStandard::BrickLink,
                },
            },
            ScanQuality::Balanced => ScanConfig {
                min_confidence: 0.8,
                min_region_size: 100,
                color_detector_config: ColorDetectorConfig {
                    threshold: 0.75,
                    standard: ColorStandard::BrickLink,
                },
            },
            ScanQuality::Accurate => ScanConfig {
                min_confidence: 0.9,
                min_region_size: 150,
                color_detector_config: ColorDetectorConfig {
                    threshold: 0.85,
                    standard: ColorStandard::BrickLink,
                },
            },
        };

        debug!("Scanner configuration: confidence={}, size={}, color_threshold={}",
            config.min_confidence,
            config.min_region_size,
            config.color_detector_config.threshold
        );

        Self { config }
    }

    /// Scan an image to identify LEGO pieces
    ///
    /// # Arguments
    /// * `path` - Path to the image file to scan
    ///
    /// # Returns
    /// * `Result<Vec<Piece>>` - A list of identified pieces or an error
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The image file cannot be opened or read
    /// - The image validation fails (e.g., image is too small)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use studfinder::{scanner::Scanner, ScanQuality};
    /// use std::path::Path;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let scanner = Scanner::new(ScanQuality::Balanced);
    /// let pieces = scanner.scan_image(Path::new("test_data/test.jpg"))?;
    /// 
    /// for piece in pieces {
    ///     println!("Detected: {} {} ({})", piece.color, piece.part_number, piece.confidence);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn scan_image<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Piece>> {
        debug!("Starting image scan for: {}", path.as_ref().display());

        let img = image::open(&path)
            .map_err(|e| StudFinderError::Image(e))?;
        debug!("Image loaded successfully: {}x{}", img.width(), img.height());

        self.validate_image(&img)?;
        debug!("Image validation passed");

        // Use the ColorDetector to analyze the color
        let color_detector = ColorDetector::with_config(self.config.color_detector_config.clone());
        let color_info = color_detector.detect_color(&img);

        if color_info.confidence < self.config.min_confidence {
            debug!("Color detection confidence too low: {:.2}", color_info.confidence);
            return Ok(vec![]);
        }

        info!("Detected color: {} (confidence: {:.2}%)", color_info.name, color_info.confidence * 100.0);

        let part_number = self.detect_part_type(&img);
        let category = self.categorize_part(&part_number);

        let pieces = vec![Piece {
            id: Uuid::new_v4().to_string(),
            part_number,
            color: color_info.name,
            category,
            quantity: 1,
            confidence: color_info.confidence,
        }];

        debug!("Created piece record: {:?}", pieces[0]);
        Ok(pieces)
    }

    /// Validate that the image meets minimum requirements
    /// 
    /// # Errors
    /// 
    /// Returns an error if the image dimensions are below the minimum requirements
    fn validate_image(&self, img: &DynamicImage) -> Result<()> {
        let (width, height) = img.dimensions();
        debug!("Validating image dimensions: {}x{}", width, height);

        if width < self.config.min_region_size || height < self.config.min_region_size {
            debug!("Image dimensions below minimum requirement: {}x{}", width, height);
            return Err(StudFinderError::InvalidDimensions {
                width,
                height,
                min_width: self.config.min_region_size,
                min_height: self.config.min_region_size,
            });
        }
        Ok(())
    }

    /// Detect the part type from the image
    /// 
    /// In a real implementation, this would use more sophisticated image analysis.
    /// Currently, it returns a simulated result.
    fn detect_part_type(&self, _img: &DynamicImage) -> String {
        let part_number = "3001";
        debug!("Part type detection returned: {}", part_number);
        part_number.to_string()
    }

    /// Categorize a part based on its part number
    /// 
    /// Maps part numbers to their corresponding categories (e.g., Brick, Plate, Tile)
    fn categorize_part(&self, part_number: &str) -> String {
        let category = match part_number {
            "3001" => "Brick",
            "3020" => "Plate",
            "3062" => "Tile",
            _ => "Unknown",
        };
        debug!("Categorized part {} as {}", part_number, category);
        category.to_string()
    }
}

impl ImageProcessor for Scanner {
    fn process_image(&self, image_path: &Path) -> Result<Vec<Piece>> {
        self.scan_image(image_path)
    }
    
    fn validate_image(&self, image: &DynamicImage) -> Result<()> {
        // Call the struct's validate_image method
        Self::validate_image(self, image)
    }
    
    fn clone_box(&self) -> Box<dyn ImageProcessor> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;
    use tempfile;

    #[test]
    fn test_scan_qualities() {
        for quality in [ScanQuality::Fast, ScanQuality::Balanced, ScanQuality::Accurate] {
            let mut img = image::RgbImage::new(200, 200);
            for pixel in img.pixels_mut() {
                *pixel = Rgb([255, 0, 0]);  // Pure red
            }

            let temp_dir = tempfile::tempdir().unwrap();
            let path = temp_dir.path().join("red_test.png");
            img.save(&path).unwrap();

            let scanner = Scanner::new(quality.clone());
            let result = scanner.scan_image(path).unwrap();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].color, "Red");
            match quality {
                ScanQuality::Fast => assert!(result[0].confidence > 0.7),
                ScanQuality::Balanced => assert!(result[0].confidence > 0.8),
                ScanQuality::Accurate => assert!(result[0].confidence > 0.9),
            }
        }
    }
    
    #[test]
    fn test_image_processor_implementation() {
        let scanner = Scanner::new(ScanQuality::Balanced);
        
        // Create a test image
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.jpg");
        
        let mut img = image::RgbImage::new(200, 200);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([255, 0, 0]);  // Pure red
        }
        img.save(&path).unwrap();
        
        // Test through the ImageProcessor trait
        let processor: Box<dyn ImageProcessor> = Box::new(scanner);
        let pieces = processor.process_image(path.as_path()).unwrap();
        
        assert!(!pieces.is_empty());
        assert_eq!(pieces[0].color, "Red");
        assert!(pieces[0].confidence > 0.8);
    }
}
