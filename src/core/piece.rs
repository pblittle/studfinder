use serde::{Deserialize, Serialize};

/// Represents a LEGO piece with its properties and metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Piece {
    /// Unique identifier for the piece
    pub id: String,
    /// LEGO part number
    pub part_number: String,
    /// Color name
    pub color: String,
    /// Category (e.g., Brick, Plate, Tile)
    pub category: String,
    /// Quantity of this piece
    pub quantity: i32,
    /// Confidence level of the detection (0.0-1.0)
    pub confidence: f32,
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}x {} {} ({}) [confidence: {:.1}%]",
            self.quantity,
            self.part_number,
            self.category,
            self.color,
            self.confidence * 100.0
        )
    }
}

/// Type of image processor to use
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProcessorType {
    /// Scanner implementation (color-based detection)
    Scanner,
    /// Detector implementation (template matching)
    Detector,
}

/// Format for exporting inventory data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

/// Quality level for scanning
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ScanQuality {
    /// Fast scanning (lower accuracy, higher speed)
    Fast,
    /// Balanced scanning (medium accuracy and speed)
    Balanced,
    /// Accurate scanning (higher accuracy, lower speed)
    Accurate,
}
