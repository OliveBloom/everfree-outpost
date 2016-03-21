#!/bin/sh
out_dir="$(dirname "$0")/png"
mkdir -p "$out_dir"
rm -f "$out_dir/*.png"

cat >"$out_dir/AUTOGENERATED_FILES.txt" <<EOF
AUTOGENERATED FILES - DO NOT EDIT
(Generated by $0 on $(date))

These PNGs are automatically extracted from the XCFs in the directory above.
Edit the XCF originals and run regenerate_pngs.sh to generate new PNGs.

The PNGs should be committed to the repo, despite being auto-generated, so that
the build process doesn't require GIMP (and GTK+, libX11, etc.).
EOF

export GIMP_LAYER_EXPORT_DIR="$out_dir"
exec gimp --new-instance \
    --no-interface --no-data --no-fonts \
    --batch-interpreter python-fu-eval \
    --batch - <"$(dirname "$0")/export_layers.py" \
    "$(dirname "$0")"/*.xcf