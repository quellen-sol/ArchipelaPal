from BaseClasses import Item, ItemClassification

# Junk Item
JUNK_CODE_OFFSET = 0x000000
JUNK_ITEM_NAME = "APBot Junk"

# Goal Item
GOAL_ITEM_OFFSET = 0x010000
GOAL_ITEM_NAME = "Magic Crystal"

# Key Item
KEY_ITEM_OFFSET = 0x020000

class APBotItem(Item):
    game = "APBot"

item_names_table = {}

# Populate item_names_to_id for all possible Keys, and Junk items
for i in range(1, 256):
    key_code = KEY_ITEM_OFFSET + i
    item_names_table[f"Key {i}"] = key_code

# Populate Junk Item
item_names_table[JUNK_ITEM_NAME] = JUNK_CODE_OFFSET

# Populate Goal Item
item_names_table[GOAL_ITEM_NAME] = GOAL_ITEM_OFFSET
