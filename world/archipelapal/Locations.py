from BaseClasses import Location

# Chest ID offset
CHEST_ITEM_OFFSET = 0x030000

class ArchipelaPalLocation(Location):
    game = "ArchipelaPal"

loc_table = {}

# Populate Hub chests
for chest_n in range(1, 256):
    chest_code = CHEST_ITEM_OFFSET + chest_n
    loc_table[f"Hub Chest {chest_n}"] = chest_code

# Populate item_names_to_id for all possible Chests
for region_n in range(1, 256):
    for chest_n in range(1, 256):
        chest_code = CHEST_ITEM_OFFSET + (region_n << 8) + chest_n
        loc_table[f"Chest {region_n}-{chest_n}"] = chest_code
