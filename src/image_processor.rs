use crate::Piece;
use anyhow::Result;
use image::DynamicImage;
use std::path::Path;

/// Trait for image processing implementations
///
/// This trait defines the interface for different image processing strategies
/// that can be used to identify LEGO pieces in images.
pub trait ImageProcessor: Send + Sync {
    /// Process an image to identify LEGO pieces
    ///
    /// # Arguments
    /// * `image_path` - Path to the image file to process
    ///
    /// # Returns
    /// * `Result<Vec<Piece>>` - A list of identified pieces or an error
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The image file cannot be opened or read
    /// - The image validation fails
    /// - The processing algorithm encounters an error
    fn process_image(&self, image_path: &Path) -> Result<Vec<Piece>>;
    
    /// Validate that an image meets the requirements for processing
    ///
    /// # Arguments
    /// * `image` - The image to validate
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the image is valid, or an error
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The image dimensions are below the minimum requirements
    /// - The image format is not supported
    /// - The image quality is too low for processing
    fn validate_image(&self, image: &DynamicImage) -> Result<()>;
    
    /// Create a boxed clone of this processor
    ///
    /// This is needed because trait objects can't use the Clone trait directly
    fn clone_box(&self) -> Box<dyn ImageProcessor>;
}

impl Clone for Box<dyn ImageProcessor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
