#!/usr/bin/env python
"""
Make patches for numbered json
"""

import subprocess
from pathlib import Path

REVISIONS = 32

ORIGINAL = Path("cfrepodata.json")
PATCHES = Path("cfpatches.json")

for revision in reversed(range(1, REVISIONS)):
    command = [
        "jpatchset",
        "-l",
        str(ORIGINAL.with_suffix(f".{revision:02d}.json")),
        "-r",
        str(ORIGINAL.with_suffix(f".{revision-1:02d}.json")),
        "-p",
        str(PATCHES),
        "-i",
        "-o",
    ]
    print(command)
    subprocess.run(command, check=True)

