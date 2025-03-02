use std::path::Path;
use anyhow::Result;
use image::DynamicImage;
use crate::Piece;

/// Trait for processing images to identify LEGO pieces
///
/// This trait defines the contract for any component that can process
/// images to extract information about LEGO pieces. Implementations
/// may use different computer vision techniques or libraries.
pub trait ImageProcessor: Send + Sync {
    /// Process an image to identify LEGO pieces
    ///
    /// # Arguments
    /// * `image_path` - Path to the image file to process
    ///
    /// # Returns
    /// * `Result<Vec<Piece>>` - A list of identified pieces or an error
    fn process_image(&self, image_path: &Path) -> Result<Vec<Piece>>;
    
    /// Validate that an image meets the requirements for processing
    ///
    /// # Arguments
    /// * `image` - The image to validate
    ///
    /// # Returns
    /// * `Result<()>` - Ok if valid, Error otherwise
    fn validate_image(&self, image: &DynamicImage) -> Result<()>;
    
    /// Clone the processor into a boxed trait object
    ///
    /// This is needed because we can't directly derive Clone for trait objects
    fn clone_box(&self) -> Box<dyn ImageProcessor>;
}

// Implement Clone for boxed ImageProcessor
impl Clone for Box<dyn ImageProcessor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
