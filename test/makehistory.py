#!/usr/bin/env python
"""
Remove random elements from a repodata.json, write.
"""

import json
import random
import pathlib

REVISIONS = 32

ORIGINAL = pathlib.Path("cfrepodata.json")

data = json.load(ORIGINAL.open())
packages = list(data["packages"].keys())

for revision in range(REVISIONS):
    removed = random.choices(packages, k=random.randint(0, 32))
    for item in removed:
        if item in data["packages"]:
            del data["packages"][item]
            print("nixed", item)
    print("write")
    json.dump(data, ORIGINAL.with_suffix(f".{revision:02d}.json").open("w+"))
