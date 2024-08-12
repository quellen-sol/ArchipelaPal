use std::collections::HashMap;

pub type ItemID = u64;
pub type ItemQuantity = u8;
pub type RegionID = u8;
pub type ChestAmount = u8;
pub type GameMap = HashMap<RegionID, Vec<Chest>>;

/// Basic flow should be
/// Python client calls a prerequisite fn that'll generate this config, then the game will play depending on what was generated.
pub struct GameConfig {
  /// In seconds
  min_time_between_checks: u64,
  /// In seconds
  max_time_bewteen_checks: u64,

  // WIP
  /// Max at RegionID::MAX :)
  num_regions: RegionID,

  min_chests_per_region: ChestAmount,
  max_chests_per_region: ChestAmount,
}

impl GameConfig {
  pub fn create_game_world(&self, map: GameMap) -> GameWorld {
    GameWorld { map }
  }
}

pub struct GameWorld {
  map: GameMap,
}

pub struct BotPlayer {
  inventory: HashMap<ItemID, ItemQuantity>,
}

pub struct Item {
  name: String,
  id: ItemID,
}

pub struct Chest {
  contains_item_id: ItemID,
  open: bool,
}
