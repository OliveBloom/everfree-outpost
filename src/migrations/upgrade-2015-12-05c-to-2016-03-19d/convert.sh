#!/bin/bash
set -e

[ "$#" -eq "2" ] || { echo "usage: $0 dist_old dist_new"; exit 1; }

old=$(readlink -f "$1")
new=$(readlink -f "$2")
cd "$(dirname "$0")"

export RUST_BACKTRACE=1

[ -d "$new/data" ] || { echo "error: $new/data shoud already exist"; exit 1; }
rm -rf "$new/save"
mkdir "$new/save/"{,clients,planes,terrain_chunks,summary}

# Determine which chunks need to be saved
./survey_chunks "$old/save"
./survey_clients "$old/save"
./process_survey "$old/save"

# Convert essential components
./convert_chunks "$old" "$new"
./convert_clients "$old" "$new"
./convert_world "$old" "$new"
./create_limbo "$new"

# Additional conversions and post-processing
./pin_entities "$new"
./convert_world_extra "$old" "$new"
./convert_structure_extra "$new"
./convert_summary "$old" "$new"
./place_ramps "$new"

cp -v "$old/save/auth.sqlite" "$new/save/auth.sqlite"
cp -v "$old/www/server.json" "$new/www/server.json"
