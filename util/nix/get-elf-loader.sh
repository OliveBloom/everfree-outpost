#!/bin/bash

# Record the interpreter of the first listed binary that's dynamically linked.
for f in /bin/sh /bin/bash /usr/bin/env; do
    path=$("$patchelf" --print-interpreter "$f") || continue
    echo -n "$path" >"$out"
    break
done

