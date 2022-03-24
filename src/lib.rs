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

use json_patch::diff;

use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct Patch {
    to: String,
    from: String,
    patch: Option<json_patch::Patch>,
}

#[derive(Serialize, Deserialize)]
struct PatchSet {
    url: String,
    latest: String,
    patches: Vec<Patch>,
}

pub fn hash(path: &str) -> Result<String, Box<dyn Error>> {
    // U constants are numbers of bytes
    use blake2::{digest::consts, Blake2b, Digest};

    let mut file = fs::File::open(&path)?;
    let mut hasher = Blake2b::<consts::U32>::new();
    let n = io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();

    println!("Path: {}", path);
    println!("Bytes processed: {}", n);
    println!("Hash value: {:x}", hash);

    Ok(format!("{:x}", hash))
}

pub fn patchy(left: &str, right: &str, patches: &str) -> Result<i32, Box<dyn Error>> {
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
    // now compute a

    println!("{:#?}", ldata);
    println!("{:#?}", rdata);
    // println!("{:?}", patch);
    println!("{}", serde_json::to_string_pretty(&patch)?);

    hash(left)?;
    hash(right)?;
    hash(patches)?;

    // let pset = PatchSet {
    //     url: "./repodata.json".to_owned(),
    //     latest: hash(right)?.to_owned(),
    //     patches: vec![
    //         Patch {
    //             from: hash(left)?,
    //             to: hash(right)?,
    //             patch: Some(patch),
    //         },
    //         Patch {
    //             from: "".to_owned(),
    //             to: hash(left)?,
    //             patch: None,
    //         },
    //     ],
    // };

    pset.patches.push(Patch {
        from: hash(left)?,
        to: hash(right)?,
        patch: Some(patch),
    });

    println!("{}", serde_json::to_string_pretty(&pset)?);

    Ok(0)
}
