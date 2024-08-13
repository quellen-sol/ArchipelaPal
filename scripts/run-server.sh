#!/bin/bash

cd ~/Archipelago/output

APFILE=$(ls -t | grep '.archipelago' | head -n1)
../ArchipelagoServer $APFILE