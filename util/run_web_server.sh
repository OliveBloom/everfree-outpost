#!/bin/sh
set -e

cd "$(dirname "$0")/../dist"
python3 ../util/web_server.py
