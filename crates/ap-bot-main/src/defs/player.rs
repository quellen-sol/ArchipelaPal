use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::lib::{ItemID, RegionID};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Player {
    /// K = ItemID, V = qty
    pub inventory: HashMap<ItemID, u16>,
    // pub checked_locations: HashSet<LocationID>,
    pub currently_exploring_region: RegionID,
}

impl Player {
    pub fn get_key_info(&self) -> Vec<RegionID> {
        log::debug!("{:?}", self.inventory);

        self.inventory
            .iter()
            .filter_map(|(id, _)| {
                let id_bytes = id.to_le_bytes();
                let item_type = id_bytes[2];
                let region = id_bytes[0];
                if item_type == 0x02 {
                    return Some(region);
                }

                None
            })
            .collect()
    }
}
