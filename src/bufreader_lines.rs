use std::io::{self, prelude::*, BufReader};
use std::fs::File;

/// Bufreader + lines()
fn main() -> io::Result<()> {
    let file = File::open("measurements_full.txt")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        // println!("{}", line?);
    }
    println!("Done reading!");
    Ok(())
}