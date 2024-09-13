use std::{collections::HashSet, sync::Arc};

use ap_rs::protocol::HintData;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{
    game_state::{FullGameState, GameMap},
    player::Player,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveFile {
    pub player: Player,
    pub map: GameMap,
    pub seed: String,
    pub team: i32,
    pub last_checked_idx: i32,
    pub slot_id: i32,
    pub source_hint_queue: HashSet<HintData>,
}

impl From<SaveFile> for FullGameState {
    fn from(value: SaveFile) -> Self {
        let player = Arc::new(RwLock::new(value.player));
        let map = Arc::new(RwLock::new(value.map));
        let last_checked_idx = Arc::new(RwLock::new(value.last_checked_idx));
        let source_hint_queue = Arc::new(RwLock::new(value.source_hint_queue));

        Self {
            map,
            player,
            seed_name: value.seed,
            team: value.team,
            last_checked_idx,
            slot_id: value.slot_id,
            source_hint_queue,
        }
    }
}
