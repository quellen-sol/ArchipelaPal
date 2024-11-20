use serde::{Deserialize, Serialize};

use crate::utils::get_region_from_loc_id;

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
        let number = id.to_le_bytes()[0];
        let region = get_region_from_loc_id(id);

        Self {
            checked: false,
            number,
            region,
            full_id: id,
        }
    }
}
