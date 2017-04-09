use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

use rand::{self, Rng};

// Intended to read names and name parts from a file. For now, just use a simple
// list of names.
struct NameGen {

}

fn lines_from_file<P>(filename: P) -> Vec<String>
    where P: AsRef<Path> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines().map(|l| l.expect("Could not parse line")).collect()
}

const NAME_FILE: &'static str = "names.txt";

lazy_static!(
    static ref NAMES: Vec<String> = lines_from_file(NAME_FILE);
);

pub fn gen() -> String {
    // NOTE: rng inside NameGen is probably overkill.
    let mut rng = rand::thread_rng();
    rng.choose(NAMES.as_slice()).unwrap().clone()
}
