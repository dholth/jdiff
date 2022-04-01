#[macro_use]
extern crate serde;
extern crate json_patch;
extern crate serde_json;

use serde::Deserialize;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;
use std::path::Path;

use json_patch::{diff, patch};

use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct Patch {
    to: String,
    from: String,
    patch: Option<json_patch::Patch>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PatchSet {
    url: String,
    latest: String,
    patches: Vec<Patch>,
}

// compute BLAKE2(256) hash of file at path
pub fn hash(path: &Path) -> Result<String, Box<dyn Error>> {
    use blake2::{digest::consts, Blake2b, Digest};

    let mut file = fs::File::open(&path)?;
    // U constants are numbers of bytes
    let mut hasher = Blake2b::<consts::U32>::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

// create new patch containing differences between left and right, and insert into patches
pub fn patchy(
    left: &Path,
    right: &Path,
    patches: &Path,
    indent: bool,
) -> Result<i32, Box<dyn Error>> {
    let f_left = File::open(left)?;
    let f_right = File::open(right)?;
    let f_patches = File::open(patches)?;
    let mut lreader = BufReader::new(f_left);
    let mut rreader = BufReader::new(f_right);
    let mut preader = BufReader::new(f_patches);

    let ldata: Value = serde_json::from_reader(&mut lreader)?;
    let rdata: Value = serde_json::from_reader(&mut rreader)?;
    let mut pset: PatchSet = serde_json::from_reader(&mut preader)?;

    let patch = diff(&ldata, &rdata);

    lreader.seek(SeekFrom::Start(0)).expect("could not seek");

    eprintln!("{:#?}", ldata);
    eprintln!("{:#?}", rdata);
    eprintln!("{}", serde_json::to_string_pretty(&patch)?);

    let hash_left = hash(left)?;
    let hash_right = hash(right)?;

    // skip insert if pset.latest already equals pset.patches[0].to
    if hash_right == pset.latest {
        eprintln!("Nothing to do");
    } else {
        pset.patches.insert(
            0,
            Patch {
                from: hash_left,
                to: hash_right,
                patch: Some(patch),
            },
        );
        pset.latest = pset.patches[0].to.clone();
    }

    if indent {
        println!("{}", serde_json::to_string_pretty(&pset)?);
    } else {
        println!("{}", serde_json::to_string(&pset)?);
    }

    Ok(0)
}

// bring left to latest by applying patches, write to right
pub fn apply(left: &Path, patches: &Path, indent: bool) -> Result<i32, Box<dyn Error>> {
    let f_left = File::open(left)?;
    let f_patches = File::open(patches)?;
    let mut lreader = BufReader::new(f_left);
    let mut preader = BufReader::new(f_patches);

    let mut ldata: Value = serde_json::from_reader(&mut lreader)?;
    let pset: PatchSet = serde_json::from_reader(&mut preader)?;

    lreader.seek(SeekFrom::Start(0)).expect("could not seek");
    let hash_left = hash(left)?;

    if hash_left == pset.latest {
        eprintln!("Nothing to do");
        return Ok(0);
    }

    // follow chain of patches from latest to the version we currently have
    let mut target = Some(pset.latest);
    let to_apply = pset
        .patches
        .into_iter()
        .filter(|p| {
            match &mut target {
                Some(target_hash) => {
                    if target_hash != &p.to {
                        return false;
                    }
                    if target_hash == &p.from {
                        // we found it
                        target = None // skip rest of array
                    } else {
                        // look for next patch in the chain
                        target = Some(p.from.to_string());
                    }
                    true
                }
                None => false,
            }
        })
        .collect::<Vec<Patch>>();

    // apply in reverse order
    for p in to_apply.iter().rev() {
        println!("{:?}", &p);
        let q = match p.patch {
            Some(ref q) => q,
            None => continue,
        };
        patch(&mut ldata, &q)?;
    }

    if indent {
        println!("{}", serde_json::to_string_pretty(&ldata)?);
    } else {
        println!("{}", serde_json::to_string(&ldata)?);
    }

    Ok(0)
}
