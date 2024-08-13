#!/bin/bash

set -e

cd ../world
./make_apworld.sh
mv apbot.apworld ~/Archipelago/lib/worlds/
cd ~/Archipelago/output

# Delete ALL files in this directory, except for .zip files
find . -type f ! -name '*.zip' -delete

cd ../

./ArchipelagoGenerate
cd ~/Archipelago/output

# There is now a new .zip file in this directory
# Unzip the latest .zip file in this directory, not in it's own folder
unzip $(ls -t | grep '.zip' | head -n1) -d .

# rm the .zip file
rm $(ls -t | grep '.zip' | head -n1)
