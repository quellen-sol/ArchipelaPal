use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::lib::{ItemID, RegionID};

/// Speed boost modifier percentage (1%).
pub const SPEED_BOOST_MODIFIER_PCT: f32 = 0.01;
pub const SPEED_BOOST_ITEM_OFFSET: ItemID = 0x040000;
pub const SPEED_BOOST_ITEM_ID: ItemID = SPEED_BOOST_ITEM_OFFSET + 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// K = ItemID, V = qty
    pub inventory: HashMap<ItemID, u16>,
    // pub checked_locations: HashSet<LocationID>,
    pub currently_exploring_region: RegionID,
    pub speed_modifier: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            inventory: HashMap::new(),
            currently_exploring_region: 0,
            speed_modifier: 1.0,
        }
    }
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

    pub fn get_num_boosts(&self) -> u16 {
        *self.inventory.get(&SPEED_BOOST_ITEM_ID).unwrap_or(&0)
    }

    pub fn get_total_speed_modifier(&self) -> f32 {
        // Testing multiple types of modifiers
        let speed_boosts = self.get_num_boosts();
        // Simple stacking 1%

        // Exponential 1%
        // let modifier = (1.0 + SPEED_BOOST_MODIFIER_PCT).powf(speed_boosts as f32);

        speed_boosts as f32 * SPEED_BOOST_MODIFIER_PCT + 1.0
    }

    pub fn set_speed_modifier(&mut self) {
        let modifier = self.get_total_speed_modifier();
        log::info!("Speed modifier set to: {}", modifier);
        self.speed_modifier = modifier;
    }
}
