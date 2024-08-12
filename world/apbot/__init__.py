from BaseClasses import Region, ItemClassification, Entrance
from worlds.AutoWorld import World, WebWorld
from worlds.generic.Rules import set_rule
from .Items import APBotItem
from .Locations import APBotLocation
from .Options import APBotOptions

class APBotWeb(WebWorld):
    tutorials = []
    theme = "ice"

HUB_NAME = "Menu"

# Junk Item
JUNK_CODE_OFFSET = 0x000000
JUNK_ITEM_NAME = "APBot Junk"

# Goal Item
GOAL_ITEM_OFFSET = 0x010000
GOAL_ITEM_NAME = "Magic Crystal"

# Key ID offset
KEY_ITEM_OFFSET = 0x020000

# Chest ID offset
CHEST_ITEM_OFFSET = 0x030000

class APBot(World):
    """
    An automatically world-playing bot for Archipelago Randomizer
    """

    game = "APBot"
    options_dataclass = APBotOptions
    options: APBotOptions
    web = APBotWeb()

    # We dont know any items prior to generation!
    item_name_to_id = {}
    location_name_to_id = {}

    item_table = {}

    # Might just do everything here?
    # Kinda wanna stay as far back as possible with this type of gen
    def generate_early(self) -> None:
        num_regions = self.options.num_regions

        min_chests_per_region = self.options.min_chests_per_region
        max_chests_per_region = self.options.max_chests_per_region


        if min_chests_per_region > max_chests_per_region:
            raise ValueError("min_chests_per_region must be less than or equal to max_chests_per_region")

        itempool = []

        # Create Hub
        hub = Region(HUB_NAME, self.player, self.multiworld)

        # Create Single Chest that contains the starting location
        starting_chest = APBotLocation(self.player, "Hub Free Chest", JUNK_CODE_OFFSET, hub)
        hub.locations.append(starting_chest)

        # + 1 for the starting location
        total_junk_items = 0
        for region_num in range(num_regions):
            # 0x00__ is reserved for the junk item id & the starting location id
            real_region = region_num + 1
            # Create Region
            region_name = f"Region {real_region}"
            region_obj = Region(region_name, self.player, self.multiworld)

            # Create Key for this region
            key_name = f"Key {real_region}"
            key_code = KEY_ITEM_OFFSET + real_region
            item = APBotItem(key_name, ItemClassification.progression, key_code, self.player)
            itempool.append(item)
            self.item_table[key_name] = {
                "classification": ItemClassification.progression,
                "code": key_code,
            }
            self.item_name_to_id[key_name] = key_code

            # Link this region to Hub + 
            # Rule that the key is required to access the region
            hub.connect(region_obj, f"Region {real_region} Entrance", lambda state: state.has(key_name, self.player))

            num_chests = self.random.randint(min_chests_per_region, max_chests_per_region)
            total_junk_items += num_chests - 1

            for chest_num in range(num_chests):
                real_chest = chest_num + 1
                chest_name = f"Chest {real_region}-{real_chest}"
                # First byte = region num, second byte = chest num
                # This guarantees that each chest has a unique ID
                chest_id = CHEST_ITEM_OFFSET + (real_region << 8) + real_chest

                self.location_name_to_id[chest_name] = chest_id
                location = APBotLocation(self.player, chest_name, chest_id, region_obj)
                region_obj.locations.append(location)

            self.multiworld.regions.append(region_obj)

        # Add Goal items
        goal_item = APBotItem(GOAL_ITEM_NAME, ItemClassification.progression, GOAL_ITEM_OFFSET, self.player)
        self.item_table[GOAL_ITEM_NAME] = {
            "classification": ItemClassification.progression,
            "code": GOAL_ITEM_OFFSET,
        }
        self.item_name_to_id[GOAL_ITEM_NAME] = GOAL_ITEM_OFFSET
        for goal_num in range(self.options.num_goal_items):
            itempool.append(goal_item)
        
        # Add completion goal
        self.multiworld.completion_condition[self.player] = lambda state: state.has_all_counts({
            GOAL_ITEM_NAME: self.options.num_goal_items,
        }, self.player)


        # Add junk items
        self.item_table[JUNK_ITEM_NAME] = {
            "classification": ItemClassification.filler,
            "code": JUNK_CODE_OFFSET,
        }
        self.item_name_to_id[JUNK_ITEM_NAME] = JUNK_CODE_OFFSET
        # Create num_chests - 1 junk items
        item = APBotItem(JUNK_ITEM_NAME, ItemClassification.filler, JUNK_CODE_OFFSET, self.player)
        for junk_num in range(total_junk_items - self.options.num_goal_items):
            itempool.append(item)

        self.multiworld.regions.append(hub)
        self.multiworld.itempool += itempool

    # Create Hub -> Regions entrances
    def create_regions(self) -> None:
        pass

    # Append `Item`s to self.multiworld.itempool
    def create_items(self) -> None:
        pass

    def create_item(self, name: str) -> APBotItem:
        item = self.item_table[name]
        return APBotItem(name, item.classification, item.code, self.player)