use crate::core::piece::{ExportFormat, Piece};
use crate::error::{Result, StudFinderError};
use std::path::Path;

/// Functions for exporting and importing inventory data
pub struct ExportManager;

impl ExportManager {
    /// Export inventory data to a file
    ///
    /// # Arguments
    /// * `pieces` - The pieces to export
    /// * `path` - The path to export to
    /// * `format` - The format to export in
    ///
    /// # Returns
    /// * `Result<()>` - Ok if the export was successful, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to serialize the data
    /// - Failed to write to the file
    pub fn export_inventory<P: AsRef<Path>>(
        pieces: &[Piece],
        path: P,
        format: &ExportFormat,
    ) -> Result<()> {
        match format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(pieces)
                    .map_err(|e| StudFinderError::Config(e.to_string()))?;
                std::fs::write(&path, json).map_err(StudFinderError::Io)?;
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
                std::fs::write(&path, output).map_err(StudFinderError::Io)?;
            }
        }
        Ok(())
    }

    /// Import inventory data from a file
    ///
    /// # Arguments
    /// * `path` - The path to import from
    ///
    /// # Returns
    /// * `Result<Vec<Piece>>` - The imported pieces, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to read the file
    /// - Failed to parse the data
    pub fn import_inventory<P: AsRef<Path>>(path: P) -> Result<Vec<Piece>> {
        let path = path.as_ref();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = std::fs::read_to_string(path).map_err(StudFinderError::Io)?;
            let pieces: Vec<Piece> = serde_json::from_str(&data).map_err(|e| {
                StudFinderError::Config(format!("Failed to parse JSON data: {}", e))
            })?;
            Ok(pieces)
        } else {
            // Assume CSV
            let data = std::fs::read_to_string(path).map_err(StudFinderError::Io)?;
            let mut pieces = Vec::new();

            for line in data.lines().skip(1) {
                // Skip header
                let fields: Vec<&str> = line.split(',').collect();
                if fields.len() == 6 {
                    let piece = Piece {
                        id: fields[0].to_string(),
                        part_number: fields[1].to_string(),
                        color: fields[2].to_string(),
                        category: fields[3].to_string(),
                        quantity: fields[4].parse().map_err(|_| {
                            StudFinderError::Config("Failed to parse quantity".to_string())
                        })?,
                        confidence: fields[5].parse().map_err(|_| {
                            StudFinderError::Config("Failed to parse confidence".to_string())
                        })?,
                    };
                    pieces.push(piece);
                }
            }
            Ok(pieces)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    use uuid::Uuid;

    fn create_test_pieces() -> Vec<Piece> {
        vec![
            Piece {
                id: Uuid::new_v4().to_string(),
                part_number: "3001".to_string(),
                color: "Red".to_string(),
                category: "Brick".to_string(),
                quantity: 1,
                confidence: 0.95,
            },
            Piece {
                id: Uuid::new_v4().to_string(),
                part_number: "3020".to_string(),
                color: "Blue".to_string(),
                category: "Plate".to_string(),
                quantity: 2,
                confidence: 0.85,
            },
        ]
    }

    #[test]
    fn test_json_export_import() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.json");

        let pieces = create_test_pieces();

        // Export
        ExportManager::export_inventory(&pieces, &path, &ExportFormat::Json).unwrap();

        // Import
        let imported = ExportManager::import_inventory(&path).unwrap();

        // Verify
        assert_eq!(imported.len(), pieces.len());
        assert_eq!(imported[0].part_number, pieces[0].part_number);
        assert_eq!(imported[1].color, pieces[1].color);
    }

    #[test]
    fn test_csv_export_import() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.csv");

        let pieces = create_test_pieces();

        // Export
        ExportManager::export_inventory(&pieces, &path, &ExportFormat::Csv).unwrap();

        // Import
        let imported = ExportManager::import_inventory(&path).unwrap();

        // Verify
        assert_eq!(imported.len(), pieces.len());
        assert_eq!(imported[0].part_number, pieces[0].part_number);
        assert_eq!(imported[1].color, pieces[1].color);
    }
}
