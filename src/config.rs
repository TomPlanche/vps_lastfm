use std::path::Path;

pub struct Config {
    pub last_fm_username: String,
    pub destination_folder: String,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let last_fm_username = std::env::var("LAST_FM_USERNAME")
            .map_err(|_| String::from("Missing env var: LAST_FM_USERNAME"))?;
        let destination_folder = std::env::var("DESTINATION_FOLDER")
            .map_err(|_| String::from("Missing env var: DESTINATION_FOLDER"))?;

        Ok(Self {
            last_fm_username,
            destination_folder,
        })
    }

    pub fn ensure_destination_folder(&self) -> Result<(), String> {
        let path = Path::new(&self.destination_folder);
        if path.exists() {
            if path.is_dir() {
                return Ok(());
            }
            return Err(format!(
                "Destination path exists but is not a directory: {}",
                self.destination_folder
            ));
        }
        std::fs::create_dir_all(path).map_err(|e| {
            format!(
                "Failed to create destination folder '{}': {}",
                self.destination_folder, e
            )
        })
    }
}
