#!/bin/bash

cd ~/Archipelago/output

../ArchipelagoServer $(ls -t | grep '.archipelago' | head -n1)
