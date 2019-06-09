#!/bin/sh
set -e

cd "$(dirname "$0")/.."
mkdir -p run
cd run

ini_contents() {
    echo '# GENERATED FILE - DO NOT EDIT'
    echo "# Copied from ../util/outpost_auth.ini, $(date)"
    echo
    cat ../util/outpost_auth.ini
}
ini_contents >outpost_auth.ini

echo "starting auth server..."
python3 ../dist/auth/server.py
