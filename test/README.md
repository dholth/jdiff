# Test jpatchset against cfrepodata.json

Get rust with rustup

Run cargo build -r in parent directory to make release build

`gunzip *.json.gz`

makehistory.py successively removes a random number of packages from cfrepodata.json to simulate 'older' versions of repadata

makepatches.py updates the patches file (start by writing an empty `{"url": "repodata.json", "latest": "", "patches": []}`)

applypatches.sh applies the patches to cfrepodata.31.json
