#!/usr/bin/env bash
# Complete deployment - pushes everything including update scripts

chmod +x push_update_scripts.sh update.sh pull_from_github.sh
./push_update_scripts.sh
