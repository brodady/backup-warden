use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupWardenConfig {
    pub watch_folder: String,
    pub backup_locations: Vec<String>,
    pub retention_days: usize,
}
