#[macro_use]
extern crate serde;
extern crate json_patch;
extern crate serde_json;
extern crate simple_error;

use simple_error::{bail, try_with};

use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use niffler;

use json_patch::{diff, patch};

use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct Patch {
    to: String,
    from: String,
    patch: json_patch::Patch, // [] is a valid, empty patch
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

    let (mut file, _format) = niffler::from_path(path)?;
    // U constants are numbers of bytes
    let mut hasher = Blake2b::<consts::U32>::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

// create new patch containing differences between left and right, and insert into patches
// if overwrite, write to patches, else stdout
// needs 'imputed' left hash value when patching a patched file: hash_from
pub fn patchy(
    left: &Path,
    right: &Path,
    patches: &Path,
    indent: bool,
    overwrite: bool,
) -> Result<i32, Box<dyn Error>> {
    let f_patches = File::open(patches)?;

    // transparently support compressed json
    let (mut lreader, _lformat) = niffler::from_path(left)?;
    let (mut rreader, _rformat) = niffler::from_path(right)?;

    let mut preader = BufReader::new(f_patches);

    let ldata: Value = try_with!(
        serde_json::from_reader(&mut lreader),
        format!("Error parsing {}", left.to_string_lossy())
    );
    let rdata: Value = try_with!(
        serde_json::from_reader(&mut rreader),
        format!("Error parsing {}", right.to_string_lossy())
    );
    let mut pset: PatchSet = try_with!(
        serde_json::from_reader(&mut preader),
        format!("Error parsing {}", patches.to_string_lossy())
    );

    drop(preader); // close it now for possible overwrite

    let patch = diff(&ldata, &rdata);

    // when patching a patched file you must keep track of what its hash "should" be
    let hash_left = hash(left)?;
    let hash_right = hash(right)?;

    // skip insert if pset.latest already equals pset.patches[0].to
    // also if hash_left == hash_right?
    if hash_right == pset.latest {
        eprintln!("Nothing to do");
    } else {
        pset.patches.insert(
            0,
            Patch {
                from: hash_left,
                to: hash_right,
                patch: patch
            },
        );
        pset.latest = pset.patches[0].to.clone();
    }

    if overwrite {
        // write to "right hand" file
        let writer = BufWriter::new(File::create(patches)?);
        if indent {
            serde_json::to_writer_pretty(writer, &pset)?;
        } else {
            serde_json::to_writer(writer, &pset)?;
        }
    } else {
        if indent {
            println!("{}", serde_json::to_string_pretty(&pset)?);
        } else {
            println!("{}", serde_json::to_string(&pset)?);
        }
    }

    Ok(0)
}

// bring left to latest by applying patches
// if overwrite, write to right, else stdout
pub fn apply(
    left: &Path,
    right: &Path,
    patches: &Path,
    indent: bool,
    overwrite: bool,
    hash_left_imputed: Option<String>,
) -> Result<i32, Box<dyn Error>> {
    let f_patches = File::open(patches)?;

    // transparently support compressed json
    let (mut lreader, _lformat) = niffler::from_path(left)?;

    let mut preader = BufReader::new(f_patches);

    // TODO show filename on error
    let mut ldata: Value = try_with!(
        serde_json::from_reader(&mut lreader),
        format!("Error parsing {}", left.to_string_lossy())
    );

    let pset: PatchSet = try_with!(
        serde_json::from_reader(&mut preader),
        format!("Error parsing {}", patches.to_string_lossy())
    );

    // when patching a patched file you must keep track of what its hash "should" be
    let hash_left = match hash_left_imputed {
        Some(hash_value) => hash_value,
        None => hash(left)?,
    };

    if hash_left == pset.latest {
        eprintln!("file is up to date");
        return Ok(0);
    }

    // follow chain of patches from latest to the version we currently have
    let mut target = Some(pset.latest);
    let to_apply = pset
        .patches
        .into_iter()
        .filter(|p| {
            match &mut target {
                // does this patch bring us closer to pset.latest?
                Some(target_hash) => {
                    if target_hash != &p.to {
                        return false;
                    }
                    if hash_left == p.from {
                        // we found it, include this patch
                        target = None; // skip rest of array
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

    if target.is_some() {
        bail!("hash {} not found in patches", hash_left);
    };

    // apply in reverse order
    for p in to_apply.iter().rev() {
        eprintln!("apply {} -> {}", &p.from, &p.to);
        patch(&mut ldata, &p.patch)?;
    }

    if overwrite {
        // write to "right hand" file
        let writer = BufWriter::new(File::create(right)?);
        if indent {
            serde_json::to_writer_pretty(writer, &ldata)?;
        } else {
            serde_json::to_writer(writer, &ldata)?;
        }
    } else {
        if indent {
            println!("{}", serde_json::to_string_pretty(&ldata)?);
        } else {
            println!("{}", serde_json::to_string(&ldata)?);
        }
    }

    Ok(0)
}
