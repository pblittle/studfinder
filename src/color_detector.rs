use image::DynamicImage;
use std::collections::HashMap;
use tracing::debug;

/// Represents a detected color with its confidence score
#[derive(Debug, Clone)]
pub struct ColorInfo {
    pub name: String,
    pub confidence: f32,
}

/// Enum representing different color standards
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorStandard {
    /// `BrickLink` color standard
    BrickLink,
    /// LEGO official color standard
    LegoOfficial,
}

/// Configuration for color detection
#[derive(Debug, Clone)]
pub struct ColorDetectorConfig {
    /// Threshold for color detection (0.0-1.0)
    pub threshold: f32,
    /// Color standard to use
    pub standard: ColorStandard,
}

impl Default for ColorDetectorConfig {
    fn default() -> Self {
        Self {
            threshold: 0.75,
            standard: ColorStandard::BrickLink,
        }
    }
}

/// Color detector for identifying colors in images
pub struct ColorDetector {
    config: ColorDetectorConfig,
    color_profiles: HashMap<String, Vec<(u8, u8, u8)>>,
}

impl Default for ColorDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorDetector {
    /// Create a new `ColorDetector` with default configuration
    /// 
    /// # Examples
    /// 
    /// ```
    /// use studfinder::color_detector::ColorDetector;
    /// 
    /// let detector = ColorDetector::new();
    /// let img = image::DynamicImage::new_rgb8(100, 100);
    /// let color_info = detector.detect_color(&img);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(ColorDetectorConfig::default())
    }

    /// Create a new `ColorDetector` with custom configuration
    /// 
    /// # Examples
    /// 
    /// ```
    /// use studfinder::color_detector::{ColorDetector, ColorDetectorConfig, ColorStandard};
    /// 
    /// let config = ColorDetectorConfig {
    ///     threshold: 0.8,
    ///     standard: ColorStandard::LegoOfficial,
    /// };
    /// let detector = ColorDetector::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: ColorDetectorConfig) -> Self {
        let mut detector = Self {
            config,
            color_profiles: HashMap::new(),
        };
        
        // Initialize color profiles based on the selected standard
        detector.initialize_color_profiles();
        
        detector
    }
    
    /// Initialize color profiles based on the selected standard
    fn initialize_color_profiles(&mut self) {
        match self.config.standard {
            ColorStandard::BrickLink => {
                // BrickLink color profiles (simplified for demonstration)
                self.color_profiles.insert("Red".to_string(), vec![(255, 0, 0)]);
                self.color_profiles.insert("Green".to_string(), vec![(0, 255, 0)]);
                self.color_profiles.insert("Blue".to_string(), vec![(0, 0, 255)]);
                self.color_profiles.insert("Yellow".to_string(), vec![(255, 255, 0)]);
                self.color_profiles.insert("White".to_string(), vec![(255, 255, 255)]);
                self.color_profiles.insert("Black".to_string(), vec![(0, 0, 0)]);
            },
            ColorStandard::LegoOfficial => {
                // LEGO official color profiles (would be more accurate in a real implementation)
                self.color_profiles.insert("Bright Red".to_string(), vec![(255, 0, 0)]);
                self.color_profiles.insert("Dark Green".to_string(), vec![(0, 255, 0)]);
                self.color_profiles.insert("Bright Blue".to_string(), vec![(0, 0, 255)]);
                self.color_profiles.insert("Bright Yellow".to_string(), vec![(255, 255, 0)]);
                self.color_profiles.insert("White".to_string(), vec![(255, 255, 255)]);
                self.color_profiles.insert("Black".to_string(), vec![(0, 0, 0)]);
            },
        }
    }
    
    /// Detect the color of an image
    /// 
    /// Analyzes the image to determine its predominant color and returns
    /// a `ColorInfo` struct containing the color name and confidence score.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use studfinder::color_detector::ColorDetector;
    /// use image::DynamicImage;
    /// 
    /// let detector = ColorDetector::new();
    /// let img = DynamicImage::new_rgb8(100, 100);
    /// let color_info = detector.detect_color(&img);
    /// 
    /// println!("Detected color: {} with confidence {:.2}",
    ///          color_info.name, color_info.confidence);
    /// ```
    #[must_use]
    pub fn detect_color(&self, img: &DynamicImage) -> ColorInfo {
        let mut colors = [0u32; 3];
        let mut pixel_count = 0;

        for pixel in img.to_rgb8().pixels() {
            colors[0] += u32::from(pixel[0]);
            colors[1] += u32::from(pixel[1]);
            colors[2] += u32::from(pixel[2]);
            pixel_count += 1;
        }

        if pixel_count == 0 {
            debug!("No pixels found in image");
            return ColorInfo {
                name: "Unknown".to_string(),
                confidence: 0.0,
            };
        }

        let avg_r = (colors[0] / pixel_count) as f32;
        let avg_g = (colors[1] / pixel_count) as f32;
        let avg_b = (colors[2] / pixel_count) as f32;

        debug!("Average RGB values: ({:.1}, {:.1}, {:.1})", avg_r, avg_g, avg_b);

        let threshold = self.config.threshold * 255.0;
        let low_threshold = (1.0 - self.config.threshold) * 255.0;

        let (color, confidence) = match () {
            // Red: high R, low G&B
            () if avg_r > threshold && avg_g < low_threshold && avg_b < low_threshold => {
                let conf = (avg_r - avg_g.max(avg_b)) / 255.0;
                (self.get_color_name("Red"), conf)
            },
            // Green: high G, low R&B
            () if avg_r < low_threshold && avg_g > threshold && avg_b < low_threshold => {
                let conf = (avg_g - avg_r.max(avg_b)) / 255.0;
                (self.get_color_name("Green"), conf)
            },
            // Blue: high B, low R&G
            () if avg_r < low_threshold && avg_g < low_threshold && avg_b > threshold => {
                let conf = (avg_b - avg_r.max(avg_g)) / 255.0;
                (self.get_color_name("Blue"), conf)
            },
            // Yellow: high R&G, low B
            () if avg_r > threshold && avg_g > threshold && avg_b < low_threshold => {
                let conf = (avg_r.min(avg_g) - avg_b) / 255.0;
                (self.get_color_name("Yellow"), conf.min(1.0))
            },
            // White: all high
            () if avg_r > threshold && avg_g > threshold && avg_b > threshold => {
                let min_val = avg_r.min(avg_g).min(avg_b);
                let conf = min_val / 255.0;
                (self.get_color_name("White"), conf)
            },
            // Black: all low
            () if avg_r < low_threshold && avg_g < low_threshold && avg_b < low_threshold => {
                let max_val = avg_r.max(avg_g).max(avg_b);
                let conf = 1.0 - (max_val / low_threshold);
                (self.get_color_name("Black"), conf)
            },
            () => {
                debug!("Could not determine color definitively");
                ("Unknown".to_string(), 0.0)
            },
        };

        debug!("Color detection result: {} with {:.2}% confidence", color, confidence * 100.0);
        
        ColorInfo {
            name: color,
            confidence,
        }
    }
    
    /// Get the color name based on the selected standard
    /// 
    /// Converts a base color name to the appropriate name in the selected color standard.
    /// For example, "Red" might become "Bright Red" in the LEGO official standard.
    fn get_color_name(&self, base_color: &str) -> String {
        match self.config.standard {
            ColorStandard::BrickLink => base_color.to_string(),
            ColorStandard::LegoOfficial => {
                match base_color {
                    "Red" => "Bright Red",
                    "Green" => "Dark Green",
                    "Blue" => "Bright Blue",
                    "Yellow" => "Bright Yellow",
                    _ => base_color,
                }.to_string()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbImage, Rgb};
    
    fn create_test_image(r: u8, g: u8, b: u8) -> DynamicImage {
        let mut img = RgbImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgb([r, g, b]);
        }
        DynamicImage::ImageRgb8(img)
    }
    
    #[test]
    fn test_color_detection_bricklink() {
        let detector = ColorDetector::new();
        
        // Test red
        let img = create_test_image(255, 0, 0);
        let color_info = detector.detect_color(&img);
        assert_eq!(color_info.name, "Red");
        assert!(color_info.confidence > 0.9);
        
        // Test green
        let img = create_test_image(0, 255, 0);
        let color_info = detector.detect_color(&img);
        assert_eq!(color_info.name, "Green");
        assert!(color_info.confidence > 0.9);
        
        // Test blue
        let img = create_test_image(0, 0, 255);
        let color_info = detector.detect_color(&img);
        assert_eq!(color_info.name, "Blue");
        assert!(color_info.confidence > 0.9);
    }
    
    #[test]
    fn test_color_detection_lego_official() {
        let config = ColorDetectorConfig {
            threshold: 0.75,
            standard: ColorStandard::LegoOfficial,
        };
        let detector = ColorDetector::with_config(config);
        
        // Test red
        let img = create_test_image(255, 0, 0);
        let color_info = detector.detect_color(&img);
        assert_eq!(color_info.name, "Bright Red");
        assert!(color_info.confidence > 0.9);
        
        // Test green
        let img = create_test_image(0, 255, 0);
        let color_info = detector.detect_color(&img);
        assert_eq!(color_info.name, "Dark Green");
        assert!(color_info.confidence > 0.9);
        
        // Test blue
        let img = create_test_image(0, 0, 255);
        let color_info = detector.detect_color(&img);
        assert_eq!(color_info.name, "Bright Blue");
        assert!(color_info.confidence > 0.9);
    }
    
    #[test]
    fn test_confidence_decreases_with_impurity() {
        let detector = ColorDetector::new();
        
        // Pure red
        let pure_img = create_test_image(255, 0, 0);
        let pure_color = detector.detect_color(&pure_img);
        
        // Impure red (with some green and blue)
        let impure_img = create_test_image(255, 50, 50);
        let impure_color = detector.detect_color(&impure_img);
        
        // Both should be detected as red
        assert_eq!(pure_color.name, "Red");
        assert_eq!(impure_color.name, "Red");
        
        // But pure red should have higher confidence
        assert!(pure_color.confidence > impure_color.confidence);
    }
}
