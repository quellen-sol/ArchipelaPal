use crate::defs::lib::{LocationID, RegionID};

pub fn get_region_from_loc_id(loc_id: LocationID) -> RegionID {
    loc_id.to_le_bytes()[2]
}

pub fn get_offset_from_le_bytes(bytes: &[u8]) -> Option<u8> {
    bytes.get(3).and_then(|val| {
        if *val == 0 {
            // check if [3] == 0 because u32's are 4 bytes,
            // so if the last byte is 0, the offset is in the second last byte
            return bytes.get(2).cloned();
        }
        Some(*val)
    })
}
