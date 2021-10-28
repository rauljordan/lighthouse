use maplit::hashset;
use rayon::prelude::*;
use slasher::{
    config::DEFAULT_CHUNK_SIZE,
    test_utils::{att_slashing, indexed_att, logger, slashed_validators_from_slashings, E},
    Config, Slasher,
};
use ssz::{Decode, Encode};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Read;
use tempfile::tempdir;
use types::{AttesterSlashing, Epoch, IndexedAttestation};

#[test]
fn process_attestations_from_file() {
    // Get the directory path to the test data
    let items = fs::read_dir("/tmp/attestations").unwrap();
    // Decode all the SSZ files of indexed attestations in the directory.
    let mut attestations = vec![];
    for entry in items {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut f = File::open(path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let att: IndexedAttestation<E> = IndexedAttestation::from_ssz_bytes(&buffer).unwrap();
        attestations.push(att);
    }
    let epoch = 74395;

    // Running the test on the first batch.
    let tempdir = tempdir().unwrap();
    let config = Config::new(tempdir.path().into()).for_testing();
    let slasher = Slasher::open(config, logger()).unwrap();
    let current_epoch = Epoch::new(epoch);

    println!("Accepting attestations from first group");
    for (i, attestation) in attestations.iter().enumerate() {
        slasher.accept_attestation(attestation.clone());
    }
    let start = std::time::Instant::now();
    println!("Processing queue");
    slasher.process_queued(current_epoch).unwrap();
    println!("Took {} seconds", start.elapsed().as_secs());

    // Pruning should not error.
    slasher.prune_database(current_epoch).unwrap();

    // Get the directory path to the test data
    let items = fs::read_dir("/tmp/attestations2").unwrap();
    // Decode all the SSZ files of indexed attestations in the directory.
    let mut attestations = vec![];
    for entry in items {
        let entry = entry.unwrap();
        let path = entry.path();
        let mut f = File::open(path).unwrap();
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        let att: IndexedAttestation<E> = IndexedAttestation::from_ssz_bytes(&buffer).unwrap();
        attestations.push(att);
    }

    println!("Accepting attestations from second group");
    for (i, attestation) in attestations.iter().enumerate() {
        slasher.accept_attestation(attestation.clone());
    }
    let start = std::time::Instant::now();
    println!("Processing queue");
    slasher.process_queued(current_epoch).unwrap();
    println!("Took {} seconds", start.elapsed().as_secs());

    drop(slasher);
    assert_eq!(2, 1);
}