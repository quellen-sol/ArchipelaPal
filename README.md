# ArchipelaPal

ArchipelaPal is a bot that plays its own world in an Archipelago Randomizer, which can be useful for world developers or for those who prefer not to or are unable to organize a game with others.

## Prerequisites

- [Archipelago Randomizer](https://github.com/ArchipelagoMW/Archipelago)

## Installation

1. Grab the latest release for your OS from the [releases page](https://github.com/quellen-sol/ArchipelaPal/releases/latest). Don't forget the `archipelapal.apworld` and `EXAMPLE.yaml` file!

2. Place the `archipelapal.apworld` file in `/path/to/Archipelago/custom_worlds` (`C:\ProgramData\Archipelago` on Windows).

3. Extract the zip file to any directory you like.

## Joining a MultiWorld

1. Edit your yaml file to your liking, then submit it to the host, along with the `.apworld` file if required. You can grab the `EXAMPLE.yaml` file from the releases page for a template.

2. When the AP server is up, run the `ArchipelaPal` program from the release you downloaded.

3. When prompted, enter the IP and port of the server, and the name of the slot.

4. Another prompt will appear, allowing you to wait to start the game, (i.e., waiting for a server countdown).

5. Watch the checks flow in! ArchipelaPal will alert you (using its terminal window) when it's in BK mode.

## Gameplay

ArchipelaPal's game world and gameplay are laid out as follows:

- ArchipelaPal spawns at its `Hub` region in the "center" of its own world. This area has a configurable number of Sphere 0 chests it will check first (`num_sphere_0_chests`).
- There are a configurable number of `Regions` locked behind `Keys` in the world. These regions must have a minimum of `min_chests_per_region` chests and a maximum of `max_chests_per_region`. `Regions` are simply numbered and correspond with one `Key` (i.e., `Region 1` is locked by `Key 1` which must be found). This is to simulate progression items similar to other games, which unlock a certain number of checks each.
- ArchipelaPal will check these regions in a random order, in a random interval between `min_time_between_checks` and `max_time_between_checks` (in seconds), but cannot check another region until it has the required key.
- ArchipelaPal will check all available checks in a region before moving on to the next region. However, if a `progression` or `useful` item is hinted to be in ArchipelaPal's world, it will check that location as soon as it is logically available.
- ArchipelaPal's goal is to collect `num_goal_items` amount of `Magic Crystals`, which, of course, are placed anywhere in any world (unless set to local). After collecting the required amount, it will automatically send a `Goal` status to the AP server, and release its remaining items, if allowed to do so.
- Speed Boosts are also placed throughout the world, which shorten its interval between checks. This is to simulate the player's progression in the game, and to make the game more interesting. The number of Speed Boosts is configurable, but the absolute minimum time between checks is `min_time_between_checks`, no matter what.
- The remaining of items after Keys, Magic Crystals, and Speed Boosts in the world are filled with `junk` items, which are not useful to ArchipelaPal.
