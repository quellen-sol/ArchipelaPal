use super::lib::RegionID;

pub enum Item {
    Junk,
    Goal,
    Key(RegionID),
    SpeedBoost,
}

impl Item {
    pub fn from_le_bytes(bytes: &[u8]) -> Self {
        let item_type = bytes[2];
        match item_type {
            0x00 => Item::Junk,
            0x01 => Item::Goal,
            0x02 => Item::Key(bytes[0]),
            0x04 => Item::SpeedBoost,
            _ => panic!("Unknown item type: 0x{:x}", item_type),
        }
    }
}
