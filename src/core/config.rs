use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::piece::{ExportFormat, ProcessorType, ScanQuality};

/// Configuration for the StudFinder application
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Path to the SQLite database file
    pub database_path: PathBuf,
    /// Format for exporting inventory data
    pub export_format: ExportFormat,
    /// Quality level for scanning
    pub scan_quality: ScanQuality,
    /// Type of image processor to use
    pub processor_type: ProcessorType,
    /// Confidence threshold for detection (0.0-1.0)
    pub confidence_threshold: f32,
}

impl Config {
    /// Initialize configuration from default locations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to determine the project directories
    /// - Failed to create the data directory
    pub fn init_default() -> anyhow::Result<Self> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "studfinder", "studfinder") {
            let data_dir = proj_dirs.data_dir();
            std::fs::create_dir_all(data_dir)?;

            Ok(Config {
                database_path: data_dir.join("studfinder.db"),
                export_format: ExportFormat::Json,
                scan_quality: ScanQuality::Balanced,
                processor_type: ProcessorType::Scanner,
                confidence_threshold: 0.8,
            })
        } else {
            Err(anyhow::anyhow!("Could not determine config directory"))
        }
    }
}
