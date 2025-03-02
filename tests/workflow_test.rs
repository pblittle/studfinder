use assert_fs::prelude::*;
use studfinder::{Config, StudFinder, ScanQuality, ProcessorType, ExportFormat};
use predicates::prelude::*;

#[tokio::test]
async fn test_full_workflow() {
    // Setup test environment
    let temp = assert_fs::TempDir::new().unwrap();
    let db_path = temp.child("test.db");
    let image_path = temp.child("red_brick.jpg");
    let export_path = temp.child("export.json");
    
    // Create test image (a red brick)
    let mut img = image::RgbImage::new(200, 200);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 0, 0]); // Pure red
    }
    img.save(image_path.path()).unwrap();
    
    // Initialize StudFinder with test config
    let config = Config {
        database_path: db_path.path().to_path_buf(),
        export_format: ExportFormat::Json,
        scan_quality: ScanQuality::Fast,
        processor_type: ProcessorType::Scanner,
        confidence_threshold: 0.8,
    };
    
    let finder = StudFinder::new(config).unwrap();
    finder.init().unwrap();
    
    // Test scanning
    let piece = finder.scan_image(image_path.path().to_path_buf()).await.unwrap();
    assert_eq!(piece.color, "Red");
    assert!(piece.confidence > 0.7);
    
    // Add the piece to the inventory
    finder.add_piece(piece.clone()).unwrap();
    
    // Test inventory
    let pieces = finder.list_inventory().unwrap();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].color, "Red");
    assert_eq!(pieces[0].part_number, piece.part_number);
    
    // Test export
    finder.export_inventory(export_path.path().to_path_buf()).unwrap();
    
    // Verify export file exists and contains the correct data
    export_path.assert(predicate::path::exists());
    let export_content = std::fs::read_to_string(export_path.path()).unwrap();
    assert!(export_content.contains(&piece.id));
    assert!(export_content.contains(&piece.color));
    assert!(export_content.contains(&piece.part_number));
}
