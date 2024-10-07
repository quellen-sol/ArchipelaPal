#!/bin/bash
# Personal script used by quellen, but, of course, can be used by anyone.
# Note the dangerous `rm` and `find -delete` commands throughout the script.
# This script is mainly used to generate a new world and bot, and then delete all files in the output directory except for the latest .zip file.
# This is useful for testing the bot and world generation process.

# run from "./scripts" directory, not root.

set -e

START_DIR=$(pwd)

cd ../world
./make_apworld.sh
mv archipelapal.apworld ~/Archipelago/lib/worlds/
cd ~/Archipelago/output

# Delete ALL files in this directory, except for .zip files
find . -type f ! -name '*.zip' -delete

cd ../

./ArchipelagoGenerate
cd ~/Archipelago/output

# There is now a new .zip file in this directory
# Unzip the latest .zip file in this directory, not in it's own folder
unzip $(ls -t | grep '.zip' | head -n1) -d .

# rm all .zip files
rm *.zip
