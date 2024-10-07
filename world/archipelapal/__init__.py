from BaseClasses import Region, ItemClassification
from worlds.AutoWorld import World, WebWorld
from .Errors import ArchipelaPalError
from .Items import ArchipelaPalItem, item_names_table, JUNK_ITEM_CODE, JUNK_ITEM_NAME, GOAL_ITEM_OFFSET, GOAL_ITEM_NAME, KEY_ITEM_OFFSET, SPEED_BOOST_NAME, SPEED_BOOST_CODE
from .Locations import ArchipelaPalLocation, loc_table, CHEST_ITEM_OFFSET
from .Options import ArchipelaPalOptions
from .utils import *

class ArchipelaPalWeb(WebWorld):
    tutorials = []
    theme = "ice"

class ArchipelaPal(World):
    """
    An automatic world-playing bot for Archipelago Randomizer
    """

    game = "ArchipelaPal"
    options_dataclass = ArchipelaPalOptions
    options: ArchipelaPalOptions
    web = ArchipelaPalWeb()

    item_name_to_id = item_names_table
    location_name_to_id = loc_table

    item_table = {}

    chests_per_region_result: list[int] = []

    # Might just do everything here?
    # Kinda wanna stay as far back as possible with this type of gen
    def generate_early(self) -> None:
        num_regions = self.options.num_regions

        min_chests_per_region = self.options.min_chests_per_region
        max_chests_per_region = self.options.max_chests_per_region

        min_time = self.options.min_time_between_checks
        max_time = self.options.max_time_between_checks

        num_sphere_0_chests = self.options.num_sphere_0_chests

        num_goal_items = self.options.num_goal_items

        pct_speed_boosts = self.options.pct_speed_boosts

        min_expected_chests = num_regions * min_chests_per_region + 1 # +1 for the starting chest

        if min_chests_per_region > max_chests_per_region:
            raise ArchipelaPalError(f"min_chests_per_region ({min_chests_per_region}) must be less than or equal to max_chests_per_region ({max_chests_per_region})")

        if min_time > max_time:
            raise ArchipelaPalError(f"min_time_between_checks ({min_time}) must be less than or equal to max_time_between_checks ({max_time})")

        if num_goal_items > min_expected_chests:
            raise ArchipelaPalError(f"num_goal_items ({num_goal_items}) must be less than or equal to the minimum expected number of chests ({min_expected_chests})")

        itempool = []

        # Menu region
        menu = Region("Menu", self.player, self.multiworld)

        # Create Hub
        hub = Region("Hub", self.player, self.multiworld)

        # Create Hub Chests (Sphere 0)
        for chest_num in range(num_sphere_0_chests):
            real_chest = chest_num + 1
            chest_name = f"Hub Chest {real_chest}"
            chest_code = CHEST_ITEM_OFFSET + real_chest

            location = ArchipelaPalLocation(self.player, chest_name, chest_code, hub)
            hub.locations.append(location)
        self.chests_per_region_result.append(num_sphere_0_chests.value)

        total_junk_items = num_sphere_0_chests
        total_checks = num_sphere_0_chests
        for region_num in range(num_regions):
            region_display_num = region_num + 1
            # Create Region
            region_name = f"Region {region_display_num}"
            region_obj = Region(region_name, self.player, self.multiworld)

            # Create Key for this region
            key_name = f"Key {region_display_num}"
            key_code = KEY_ITEM_OFFSET + region_display_num
            key_item = ArchipelaPalItem(key_name, ItemClassification.progression, key_code, self.player)
            itempool.append(key_item)
            self.item_table[key_name] = {
                "classification": ItemClassification.progression,
                "code": key_code,
            }

            num_chests = self.random.randint(min_chests_per_region, max_chests_per_region)
            total_checks += num_chests
            self.chests_per_region_result.append(num_chests)
            total_junk_items += num_chests - 1

            for chest_num in range(num_chests):
                real_chest = chest_num + 1
                chest_name = f"Chest {region_display_num}-{real_chest}"
                chest_code = CHEST_ITEM_OFFSET + (region_display_num << 8) + real_chest

                location = ArchipelaPalLocation(self.player, chest_name, chest_code, region_obj)
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
        goal_item = ArchipelaPalItem(GOAL_ITEM_NAME, ItemClassification.progression, GOAL_ITEM_OFFSET, self.player)
        self.item_table[GOAL_ITEM_NAME] = {
            "classification": ItemClassification.progression,
            "code": GOAL_ITEM_OFFSET,
        }
        for goal_num in range(num_goal_items):
            itempool.append(goal_item)
        
        # Add completion goal
        self.multiworld.completion_condition[self.player] = lambda state: state.has_all_counts({
            GOAL_ITEM_NAME: num_goal_items,
        }, self.player)

        # Add Game-affecting Items
        pct_float = pct_speed_boosts / 100
        num_speed_boosts = int(total_checks * pct_float)
        # Clamp speed boosts to the number of junk items
        num_speed_boosts = min(num_speed_boosts, total_junk_items)
        self.item_table[SPEED_BOOST_NAME] = {
            "classification": ItemClassification.useful,
            "code": SPEED_BOOST_CODE,
        }
        speed_boost_item = ArchipelaPalItem(SPEED_BOOST_NAME, ItemClassification.useful, SPEED_BOOST_CODE, self.player)
        for speed_boost_num in range(num_speed_boosts):
            itempool.append(speed_boost_item)
        total_junk_items -= num_speed_boosts
        
        # Add Junk items
        self.item_table[JUNK_ITEM_NAME] = {
            "classification": ItemClassification.filler,
            "code": JUNK_ITEM_CODE,
        }
        junk_item = ArchipelaPalItem(JUNK_ITEM_NAME, ItemClassification.filler, JUNK_ITEM_CODE, self.player)
        for junk_num in range(total_junk_items - num_goal_items):
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

    def create_item(self, name: str) -> ArchipelaPalItem:
        item = self.item_table[name]
        return ArchipelaPalItem(name, item.classification, item.code, self.player)

    def fill_slot_data(self):
        min_wait_time = self.options.min_time_between_checks.value
        max_wait_time = self.options.max_time_between_checks.value
        num_goal = self.options.num_goal_items.value
        slot_name = self.player_name
        num_regions = self.options.num_regions.value
        chests_per_region_list = self.chests_per_region_result

        return {
            "min_wait_time": min_wait_time,
            "max_wait_time": max_wait_time,
            "num_goal": num_goal,
            "slot_name": slot_name,
            "num_regions": num_regions,
            "chests_per_region_list": chests_per_region_list,
        }
