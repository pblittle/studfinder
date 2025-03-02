use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub export_format: crate::ExportFormat,
    pub scan_quality: crate::ScanQuality,
    pub processor_type: crate::ProcessorType,
    pub confidence_threshold: f32,
}

/// Initialize configuration from default locations
///
/// # Errors
///
/// Return
/// Initis an error if:
/// - Unable to determine the project directories
/// - Failed to create the data directory
pub fn init_config() -> anyhow::Result<Config> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "studfinder", "studfinder") {
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;

        Ok(Config {
            database_path: data_dir.join("studfinder.db"),
            export_format: crate::ExportFormat::Json,
            scan_quality: crate::ScanQuality::Balanced,
            processor_type: crate::ProcessorType::Scanner,
            confidence_threshold: 0.8,
        })
    } else {
        Err(anyhow::anyhow!("Could not determine config directory"))
    }
}
