use serde::{Deserialize, Serialize};

use super::lib::{LocationID, RegionID};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chest {
    pub region: RegionID,
    pub full_id: LocationID,
    pub number: u8,
    pub checked: bool,
}

impl Chest {
    pub fn new_from_id(id: LocationID) -> Self {
        let b = id.to_le_bytes();
        let number = b[0];
        let region = b[1];

        Self {
            checked: false,
            number,
            region,
            full_id: id,
        }
    }
}
