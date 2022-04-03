#!/bin/sh
# first, make sure to gunzip *.json.gz
# and run cargo build -r

# --overwrite writes to -r argument instead of stdout
# (if not doing patch, --overwrite writes to -p argument)
time ../target/release/jpatchset \
    -l cfrepodata.31.json \
    -r cfrepodata-patched.json \
    -p cfpatches.json \
    --indent --overwrite patch