use assert_fs::prelude::*;
use image::{Rgb, RgbImage};
use studfinder::{Config, ProcessorType, ScanQuality, StudFinder};
use test_case::test_case;

// Helper function to create a test image with a specific color
fn create_test_image(r: u8, g: u8, b: u8) -> (assert_fs::TempDir, assert_fs::fixture::ChildPath) {
    let temp = assert_fs::TempDir::new().unwrap();
    let image_path = temp.child("test_image.jpg");

    let mut img = RgbImage::new(200, 200);
    for pixel in img.pixels_mut() {
        *pixel = Rgb([
            r, g, b,
        ]);
    }
    img.save(image_path.path()).unwrap();

    (temp, image_path)
}

// Test cases for different colors
#[test_case(255, 0, 0 => "Red"; "pure red")]
#[test_case(0, 255, 0 => "Green"; "pure green")]
#[test_case(0, 0, 255 => "Blue"; "pure blue")]
#[test_case(255, 255, 0 => "Yellow"; "pure yellow")]
#[test_case(255, 255, 255 => "White"; "pure white")]
#[test_case(0, 0, 0 => "Black"; "pure black")]
#[tokio::test]
async fn test_color_detection(r: u8, g: u8, b: u8) -> String {
    // Create test image with the specified color
    let (temp, image_path) = create_test_image(r, g, b);

    // Initialize StudFinder with Scanner processor
    let config = Config {
        database_path: temp.child("test.db").path().to_path_buf(),
        export_format: studfinder::ExportFormat::Json,
        scan_quality: ScanQuality::Fast,
        processor_type: ProcessorType::Scanner,
        confidence_threshold: 0.7,
    };

    let finder = StudFinder::new(config).unwrap();
    finder.init().unwrap();

    // Scan the image and return the detected color
    let piece = finder
        .scan_image(image_path.path().to_path_buf())
        .await
        .unwrap();
    piece.color
}

// Test that confidence decreases as colors become less pure
#[tokio::test]
async fn test_color_confidence_decreases_with_impurity() {
    // Create a pure red image
    let (temp1, pure_path) = create_test_image(255, 0, 0);

    // Create an impure red image (with some green and blue)
    let (_temp2, impure_path) = create_test_image(255, 50, 50);

    // Initialize StudFinder with Scanner processor
    let config = Config {
        database_path: temp1.child("test.db").path().to_path_buf(),
        export_format: studfinder::ExportFormat::Json,
        scan_quality: ScanQuality::Fast,
        processor_type: ProcessorType::Scanner,
        confidence_threshold: 0.7,
    };

    let finder = StudFinder::new(config).unwrap();
    finder.init().unwrap();

    // Scan both images
    let pure_piece = finder
        .scan_image(pure_path.path().to_path_buf())
        .await
        .unwrap();
    let impure_piece = finder
        .scan_image(impure_path.path().to_path_buf())
        .await
        .unwrap();

    // Both should be detected as red
    assert_eq!(pure_piece.color, "Red");
    assert_eq!(impure_piece.color, "Red");

    // But the pure red should have higher confidence
    assert!(pure_piece.confidence > impure_piece.confidence);
}
