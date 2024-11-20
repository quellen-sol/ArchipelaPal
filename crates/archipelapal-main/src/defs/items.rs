use super::lib::RegionID;

pub enum Item {
    Junk,
    Goal,
    Key(RegionID),
    GameAffector(Effect),
}

impl Item {
    pub fn try_from_le_bytes(bytes: &[u8]) -> Option<Self> {
        let item_offset = bytes.get(0)?;
        match item_offset {
            0x00 => Some(Item::Junk),
            0x01 => Some(Item::Goal),
            0x02 => Some(Item::Key(bytes[2])),
            0x04 => Some(Item::GameAffector(Effect::try_from_le_bytes(bytes)?)),
            _ => None,
        }
    }
}

pub enum Effect {
    SpeedBoost,
}

impl Effect {
    pub fn try_from_le_bytes(bytes: &[u8]) -> Option<Self> {
        let affector_type = bytes.get(1)?;
        match affector_type {
            0x00 => Some(Effect::SpeedBoost),
            _ => None,
        }
    }
}
