"""
Cache several conda repodata plus history.
"""

import requests_cache
import pathlib
import gzip

session = requests_cache.CachedSession(cache_control=True)

REPOS = [
    "repo.anaconda.com/pkgs/main",
    "conda.anaconda.org/conda-forge",
]

SUBDIRS = [
    "linux-32",
    "linux-64",
    "linux-aarch64",
    "linux-armv6l",
    "linux-armv7l",
    "linux-ppc64le",
    "linux-s390x",
    "noarch",
    "osx-64",
    "osx-arm64",
    "win-32",
    "win-64",
    "zos-z",
]

for repo in REPOS:
    for subdir in SUBDIRS:
        for url in [
            f"https://{repo}/{subdir}/repodata.json",
            f"https://{repo}/{subdir}/current_repodata.json",
        ]:
            response = session.get(url)
            print(response.from_cache, url)
            print(response.cache_key)
            print(response.headers)
            output = pathlib.Path(url.lstrip("https://"))
            stem = output.stem
            if not output.exists() or not response.from_cache:
                if output.exists():
                    i = 0
                    while output.with_stem(f"{stem}-{i:03d}").exists():
                        i += 1
                    output.rename(output.with_stem(f"{stem}-{i:03d}"))
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_bytes(gzip.compress(response.content))
