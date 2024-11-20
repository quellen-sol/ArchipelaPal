use crate::utils::get_offset_from_le_bytes;

use super::lib::{ItemID, RegionID};

pub enum Item {
    Junk,
    Goal,
    Key(RegionID),
    GameAffector(Effect),
}

impl Item {
    pub fn try_from_le_bytes(bytes: &[u8]) -> Option<Self> {
        let item_offset = get_offset_from_le_bytes(bytes)?;

        match item_offset {
            0x00 => Some(Item::Junk),
            0x01 => Some(Item::Goal),
            0x02 => Some(Item::Key(bytes[0])),
            0x04 => Some(Item::GameAffector(Effect::try_from_le_bytes(bytes)?)),
            _ => None,
        }
    }

    pub fn from_id(id: ItemID) -> Option<Self> {
        Self::try_from_le_bytes(&id.to_le_bytes())
    }
}

pub enum Effect {
    SpeedBoost,
}

impl Effect {
    pub fn try_from_le_bytes(bytes: &[u8]) -> Option<Self> {
        let affector_type = bytes.get(2)?;
        match affector_type {
            0x00 => Some(Effect::SpeedBoost),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::defs::lib::ItemID;

    use super::*;

    #[test]
    fn test_junk_item() {
        let value: ItemID = 0x000001;
        let item = Item::from_id(value).unwrap();
        assert!(matches!(item, Item::Junk));
    }

    #[test]
    fn test_goal_item() {
        let value: ItemID = 0x010001;
        let item = Item::from_id(value).unwrap();
        assert!(matches!(item, Item::Goal));
    }

    #[test]
    fn test_key_deser() {
        let value: ItemID = 0x020001;
        let item = Item::from_id(value).unwrap();
        assert!(matches!(item, Item::Key(1)));
    }

    #[test]
    fn test_speed_boost() {
        let value: ItemID = 0x04000001;
        let item = Item::from_id(value).unwrap();
        assert!(matches!(item, Item::GameAffector(Effect::SpeedBoost)));
    }
}
