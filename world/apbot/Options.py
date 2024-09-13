from Options import Range, PerGameCommonOptions
from dataclasses import dataclass

class NumRegions(Range):
    """Number of \"regions\" (keys) that the bot can get. Each region has a number of chests behind the region gate"""
    range_start = 1
    range_end = 255
    default = 10


class MinChestsPerRegion(Range):
    """Minimum number of chests behind a region gate, must be greater than or equal to 1"""
    range_start = 1
    range_end = 255
    default = 5


class MaxChestsPerRegion(Range):
    """Maximum number of chests behind a region gate, must be greater than or equal to MinChestsPerRegion"""
    range_start = 1
    range_end = 255
    default = 10


class MinTimeBetweenChecks(Range):
    """Minimum time between checks in seconds (WARNING: high values will cause extremely long games!). Must be greater than or equal to 1"""
    range_start = 1
    range_end = 3600
    default = 60


class MaxTimeBetweenChecks(Range):
    """Maximum time between checks in seconds. Must be greater than or equal to MinTimeBetweenChecks"""
    range_start = 1
    range_end = 3600
    default = 300


class NumGoalItems(Range):
    """Number of goal items required to finish the game."""
    range_start = 1
    range_end = 255
    default = 10


class NumSphere0Chests(Range):
    """Number of chests that require no progression to access (AKA Sphere 0)"""
    range_start = 1
    range_end = 255
    default = 15


@dataclass
class APBotOptions(PerGameCommonOptions):
    num_regions:                        NumRegions
    min_chests_per_region:              MinChestsPerRegion
    max_chests_per_region:              MaxChestsPerRegion
    min_time_between_checks:            MinTimeBetweenChecks
    max_time_between_checks:            MaxTimeBetweenChecks
    num_goal_items:                     NumGoalItems
    num_sphere_0_chests:                NumSphere0Chests
