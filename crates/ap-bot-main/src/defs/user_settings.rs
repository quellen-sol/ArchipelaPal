use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const USER_SETTINGS_FILE: &str = "user_settings.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserSettings {
    pub last_used_address: Option<String>,
    pub last_used_slot: Option<String>,
}

impl UserSettings {
    pub fn load() -> Result<Self> {
        Ok(fs::read_to_string(USER_SETTINGS_FILE)
            .and_then(|file_str| Ok(serde_json::from_str::<UserSettings>(&file_str)?))?)
    }

    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let savefile_json = serde_json::to_string(self)?;
        fs::write(USER_SETTINGS_FILE, savefile_json)?;

        Ok(())
    }
}
