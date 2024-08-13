from BaseClasses import Region, ItemClassification
from worlds.AutoWorld import World, WebWorld
from .Items import APBotItem
from .Locations import APBotLocation
from .Options import APBotOptions

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

class APBotWeb(WebWorld):
    tutorials = []
    theme = "ice"

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

        # Menu region
        menu = Region("Menu", self.player, self.multiworld)

        # Create Hub
        hub = Region("Hub", self.player, self.multiworld)

        # Create Single Chest that contains the starting location
        HUB_CHEST_ID = CHEST_ITEM_OFFSET + 1
        starting_chest = APBotLocation(self.player, "Hub Free Chest", HUB_CHEST_ID, hub)
        hub.locations.append(starting_chest)
        self.location_name_to_id["Hub Free Chest"] = HUB_CHEST_ID;

        total_junk_items = 0
        for region_num in range(num_regions):
            region_display_num = region_num + 1
            # Create Region
            region_name = f"Region {region_display_num}"
            region_obj = Region(region_name, self.player, self.multiworld)

            # Create Key for this region
            key_name = f"Key {region_display_num}"
            key_code = KEY_ITEM_OFFSET + region_display_num
            key_item = APBotItem(key_name, ItemClassification.progression, key_code, self.player)
            itempool.append(key_item)
            self.item_table[key_name] = {
                "classification": ItemClassification.progression,
                "code": key_code,
            }
            self.item_name_to_id[key_name] = key_code

            num_chests = self.random.randint(min_chests_per_region, max_chests_per_region)
            total_junk_items += num_chests - 1

            for chest_num in range(num_chests):
                real_chest = chest_num + 1
                chest_name = f"Chest {region_display_num}-{real_chest}"
                chest_code = CHEST_ITEM_OFFSET + (region_display_num << 8) + real_chest

                self.location_name_to_id[chest_name] = chest_code
                location = APBotLocation(self.player, chest_name, chest_code, region_obj)
                region_obj.locations.append(location)

            self.multiworld.regions.append(region_obj)

            # Link this region to Hub &
            # Rule that the key is required to access the region
            # Wtf is this referencing bs, Python??? I have to use a default argument to hold on to the correct value????
            def rule(state, key_name=key_name):
                # print(f"Checking for {key_name} in {state}")
                return state.has(key_name, self.player)
            # print(f"Connecting {hub.name} to {region_obj.name} with key rule checking for {key_name}")
            hub.connect(region_obj, None, rule)

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

        # Add Junk items
        self.item_table[JUNK_ITEM_NAME] = {
            "classification": ItemClassification.filler,
            "code": JUNK_CODE_OFFSET,
        }
        self.item_name_to_id[JUNK_ITEM_NAME] = JUNK_CODE_OFFSET
        junk_item = APBotItem(JUNK_ITEM_NAME, ItemClassification.filler, JUNK_CODE_OFFSET, self.player)
        for junk_num in range(total_junk_items - self.options.num_goal_items):
            itempool.append(junk_item)

        self.multiworld.regions.append(menu)
        self.multiworld.regions.append(hub)
        menu.connect(hub)

        # Debug prints
        # print(self.item_name_to_id)
        # print(self.location_name_to_id)
        # print(self.item_table)
        # print(itempool)
        # for r in self.multiworld.regions:
        #     print(f"{r.name} entrances:", [e for e in r.entrances])
        #     print(f"{r.name} exits:", [ex for ex in r.exits])
        #     print(f"{r.name} locations:", [loc for loc in r.locations])

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
