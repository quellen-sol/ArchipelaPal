use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use anyhow::Result;
use ap_rs::protocol::HintData;
use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

use super::{
    chest::Chest,
    lib::{LocationID, OutputFileConfig, RegionID, CHEST_OFFSET, SAVE_FILE_DIRECTORY},
    player::Player,
    save_file::SaveFile,
};

#[derive(Debug, Default)]
pub struct FullGameState {
    pub player: Arc<RwLock<Player>>,
    pub map: Arc<RwLock<GameMap>>,
    pub seed_name: String,
    pub team: i32,
    pub last_checked_idx: Arc<RwLock<i32>>,
    pub slot_id: i32,
    /// A queue of hints that we are currently searching for in OUR world
    pub source_hint_queue: Arc<RwLock<HashSet<HintData>>>,
}

impl FullGameState {
    /// Returns a checked location's ID, if we check one
    pub async fn tick_game_state(&self) -> Option<LocationID> {
        let player = self.player.read().await;
        let player_region_keys = player.get_key_info();
        log::debug!("Region keys: {:?}", player_region_keys);

        // Check if we can get something from the hint list first
        let source_hint_queue = self.source_hint_queue.read().await;
        let hint_item = source_hint_queue.iter().find_map(|hint| {
            let loc_id = hint.item.location;
            let loc_id_bytes = loc_id.to_le_bytes();
            let region = loc_id_bytes[1];
            if player_region_keys.contains(&region) {
                return Some(hint.item.location);
            }

            None
        });

        if let Some(hint_loc) = hint_item {
            let region = hint_loc.to_le_bytes()[1];
            let mut map = self.map.write().await;
            let chest = map
                .map
                .get_mut(&region)
                .and_then(|chests| {
                    chests
                        .iter_mut()
                        .find(|chest| chest.full_id == hint_loc as LocationID)
                })
                .unwrap_or_else(|| panic!("Chest {hint_loc} should exist in game map"));
            if !chest.checked {
                chest.checked = true;
                return Some(hint_loc as LocationID);
            }
        }

        let map = self.map.read().await;
        let search_region = player.currently_exploring_region;
        let initial_chest = Self::choose_chest_in_region(&map, &search_region);

        let alternate_chest = map.map.iter().find_map(|(region, chests)| {
            if *region == search_region || !player_region_keys.contains(region) {
                return None;
            }

            chests.iter().enumerate().find_map(|(idx, chest)| {
                if !chest.checked {
                    return Some((*region, idx));
                }

                None
            })
        });

        drop(player);
        drop(map);

        let mapped_chest_options = initial_chest
            .map(|idx| (search_region, idx))
            .or(alternate_chest);

        let chosen_check = if let Some((chosen_region, chosen_chest_idx)) = mapped_chest_options {
            if chosen_region != search_region {
                let mut player = self.player.write().await;
                player.currently_exploring_region = chosen_region;
            }

            let mut map = self.map.write().await;
            let chest = map
                .map
                .get_mut(&chosen_region)
                .and_then(|region| region.get_mut(chosen_chest_idx));

            if let Some(chest) = chest {
                chest.checked = true;

                Some(chest.full_id)
            } else {
                None
            }
        } else {
            None
        };

        self.write_save_file()
            .await
            .inspect_err(|e| {
                log::error!("Error saving file: {e}");
            })
            .ok();

        chosen_check
    }

    pub fn choose_chest_in_region(map_guard: &GameMap, region: &RegionID) -> Option<usize> {
        let mut rng = thread_rng();
        map_guard
            .map
            .get(region)
            .map(|region| {
                region
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, chest)| if !chest.checked { Some(idx) } else { None })
            })
            .expect("Bad game mapping, could not find region")
            .choose(&mut rng)
    }

    pub async fn write_save_file(&self) -> Result<()> {
        let player_copy = self.player.read().await.clone();
        let map_copy = self.map.read().await.clone();
        let last_checked_idx = *self.last_checked_idx.read().await;
        let source_hint_queue = self.source_hint_queue.read().await.clone();

        let save_file = SaveFile {
            player: player_copy,
            map: map_copy,
            seed: self.seed_name.clone(),
            team: self.team,
            last_checked_idx,
            slot_id: self.slot_id,
            source_hint_queue,
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
        format!("{SAVE_FILE_DIRECTORY}/save-file-{seed_name}.json")
    }

    pub fn make_hints_get_key(&self, slot_id: i32) -> String {
        let team = self.team;
        format!("_read_hints_{team}_{slot_id}")
    }
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
            let chest = Chest::new_from_datapackage_entry(id, name.clone());
            let entry = map.entry(chest.region).or_insert(vec![]);
            entry.push(chest);
        }

        Self { map }
    }

    pub fn new_from_config(config: &OutputFileConfig) -> Self {
        let mut map = HashMap::new();

        for (region_idx, num_chests) in config.chests_per_region_list.iter().enumerate() {
            let region_real_num = region_idx as LocationID;
            for chest_i in 1..=(*num_chests as LocationID) {
                let chest_id = CHEST_OFFSET + (region_real_num << 8) + chest_i;
                let chest = Chest::new_from_id(chest_id);
                let entry = map.entry(chest.region).or_insert(vec![]);
                entry.push(chest);
            }
        }

        Self { map }
    }
}
