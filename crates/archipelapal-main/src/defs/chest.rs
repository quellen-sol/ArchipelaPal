use serde::{Deserialize, Serialize};

use super::lib::{LocationID, RegionID};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chest {
    pub name: String,
    pub region: RegionID,
    pub full_id: LocationID,
    pub number: u8,
    pub checked: bool,
}

impl Chest {
    pub fn new_from_datapackage_entry(val: &i32, name: String) -> Self {
        let b = val.to_le_bytes();
        let number = b[0];
        let region = b[1];

        Self {
            checked: false,
            name,
            number,
            region,
            full_id: *val as LocationID,
        }
    }

    pub fn new_from_id(id: LocationID) -> Self {
        let b = id.to_le_bytes();
        let number = b[0];
        let region = b[1];
        let (base_name, num_name) = if region == 0 {
            ("Hub Chest", number.to_string())
        } else {
            ("Chest", format!("{region}-{number}"))
        };
        let name = format!("{base_name} {num_name}");

        Self {
            checked: false,
            name,
            number,
            region,
            full_id: id,
        }
    }
}
