use studfinder::{Config, StudFinder, ScanQuality, ProcessorType, Piece};
use uuid::Uuid;

// Helper function to create a test piece
fn create_test_piece() -> Piece {
    Piece {
        id: Uuid::new_v4().to_string(),
        part_number: "3001".to_string(),
        color: "Red".to_string(),
        category: "Brick".to_string(),
        quantity: 1,
        confidence: 0.95,
    }
}

#[tokio::test]
async fn test_studfinder_inventory_operations() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Create the config
    let config = Config {
        database_path: db_path,
        export_format: studfinder::ExportFormat::Json,
        scan_quality: ScanQuality::Fast,
        processor_type: ProcessorType::Scanner,
        confidence_threshold: 0.8,
    };
    
    // Create the finder
    let finder = StudFinder::new(config).unwrap();
    finder.init().unwrap();
    
    // Create a test piece
    let piece = create_test_piece();
    
    // Add the piece to the inventory
    finder.add_piece(piece.clone()).unwrap();
    
    // Test inventory
    let pieces = finder.list_inventory().unwrap();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].color, "Red");
    assert_eq!(pieces[0].part_number, "3001");
    
    // Test updating quantity
    let updated_piece = Piece {
        id: piece.id.clone(),
        part_number: piece.part_number.clone(),
        color: piece.color.clone(),
        category: piece.category.clone(),
        quantity: 2,
        confidence: piece.confidence,
    };
    
    finder.add_piece(updated_piece).unwrap();
    
    // Verify the update
    let pieces = finder.list_inventory().unwrap();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].quantity, 3); // 1 + 2 = 3
    
    // Test export
    let export_path = temp_dir.path().join("export.json");
    finder.export_inventory(export_path.clone()).unwrap();
    
    // Verify export file exists and contains the correct data
    assert!(export_path.exists());
    let export_content = std::fs::read_to_string(export_path).unwrap();
    assert!(export_content.contains(&piece.id));
    assert!(export_content.contains(&piece.color));
    assert!(export_content.contains(&piece.part_number));
    
    // Test import
    // First, reset the database
    finder.reset().unwrap();
    assert_eq!(finder.list_inventory().unwrap().len(), 0);
    
    // Then import the previously exported data
    let import_path = temp_dir.path().join("export.json");
    finder.import_inventory(import_path).unwrap();
    
    // Verify the import
    let pieces = finder.list_inventory().unwrap();
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].color, "Red");
    assert_eq!(pieces[0].part_number, "3001");
    assert_eq!(pieces[0].quantity, 3);
}
