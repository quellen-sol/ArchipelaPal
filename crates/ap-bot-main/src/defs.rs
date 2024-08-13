use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use ap_rs::protocol::RoomInfo;
use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

pub type RegionID = u8;
pub type LocationID = u32;
pub type ItemID = u32;
pub type GoalOneShotData = GoalData;

#[derive(Debug)]
pub struct GoalData {
    pub room_info: RoomInfo,
}

#[derive(Debug, Default)]
pub struct FullGameState {
    pub player: Arc<RwLock<Player>>,
    pub map: Arc<RwLock<GameMap>>,
    pub seed_name: String,
}

impl FullGameState {
    /// Returns a checked location's ID, if we check one
    pub async fn tick_game_state(&self) -> Option<LocationID> {
        // Check for all available checks
        // Check one random
        let mut player = self.player.write().await;
        let mut map = self.map.write().await;

        // Check available chests in current region, and choose one to open
        let chest = {
            let mut rng = thread_rng();
            map.map
                .get_mut(&player.currently_exploring_region)
                .map(|region| region.iter_mut().filter(|chest| !chest.checked))
                .expect("Bad game mapping, could not find region")
                .choose(&mut rng)
        };

        let checked_location = match chest {
            None => {
                // Find the first region we DO have a key for and change current to that
                let map = map.downgrade();
                let first_avail = map.map.iter().find(|(region, chests)| {
                    let key_id = **region as u32 + 0x020000;
                    (player.inventory.contains_key(&key_id) || **region == 0)
                        && chests.iter().any(|chest| !chest.checked)
                });

                if let Some((region, _)) = first_avail {
                    player.currently_exploring_region = *region;
                }

                // We have nothing... we're BK'd!
                None
            }
            Some(chest) => {
                let id = chest.full_id;
                chest.checked = true;

                Some(id)
            }
        };

        self.write_save_file()
            .await
            .inspect_err(|e| {
                log::error!("Error saving file: {e}");
            })
            .ok();

        checked_location
    }

    pub async fn write_save_file(&self) -> Result<()> {
        let player_copy = self.player.read().await.clone();
        let map_copy = self.map.read().await.clone();

        let save_file = SaveFile {
            player: player_copy,
            map: map_copy,
            seed: self.seed_name.clone(),
        };

        let savefile_json = serde_json::to_string(&save_file)?;

        let save_path = Self::make_save_file_name(&self.seed_name);
        fs::write(save_path, savefile_json).await?;

        Ok(())
    }

    pub fn from_file_or_default(seed_name: &str) -> Self {
        let name = Self::make_save_file_name(seed_name);
        std::fs::read_to_string(&name)
            .and_then(|file_str| serde_json::from_str::<SaveFile>(&file_str).map_err(|e| e.into()))
            .inspect_err(|e| log::error!("Unable to read save file: {e}\nLoading a fresh save...."))
            .unwrap_or_default()
            .into()
    }

    fn make_save_file_name(seed_name: &str) -> String {
        format!("save-file-{seed_name}.json")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveFile {
    player: Player,
    map: GameMap,
    seed: String,
}

impl From<SaveFile> for FullGameState {
    fn from(value: SaveFile) -> Self {
        let player = Arc::new(RwLock::new(value.player));
        let map = Arc::new(RwLock::new(value.map));

        Self {
            map,
            player,
            seed_name: value.seed,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub min_wait_time: u16,
    pub max_wait_time: u16,
    pub num_goal: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GameMap {
    pub map: HashMap<RegionID, Vec<Chest>>,
}

impl GameMap {
    /// from `location_name_to_id`
    pub fn new_from_data_package(data_pkg: &HashMap<String, i32>) -> Self {
        let mut map = HashMap::new();

        for (name, id) in data_pkg.iter() {
            let chest = Chest::new_from_entry(id, name.clone());
            let entry = map.entry(chest.region).or_insert(vec![]);
            entry.push(chest);
        }

        Self { map }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chest {
    pub name: String,
    pub region: RegionID,
    pub full_id: LocationID,
    pub number: u8,
    pub checked: bool,
}

impl Chest {
    pub fn new_from_entry(val: &i32, name: String) -> Self {
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
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Player {
    /// K = ItemID, V = qty
    pub inventory: HashMap<ItemID, u16>,
    // pub checked_locations: HashSet<LocationID>,
    pub currently_exploring_region: RegionID,
}
