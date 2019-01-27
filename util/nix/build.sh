#!/bin/bash
if [ -n "$1" ]; then
    initrev="$1"
    shift 1
else
    initrev="HEAD"
fi
rev=$(git rev-parse "$initrev")
nix-build -E "import $(dirname "$0")/outpost.nix { rev = \"$rev\"; }" "$@"
