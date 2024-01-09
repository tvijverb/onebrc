use std::fs::File;
use memmap2::Mmap;

/// memory-mapped file + split new line + collect Vec<&[u8]>
fn main() -> std::io::Result<()> {
    let file = File::open("measurements_full.txt")?;
    let mmap = unsafe { Mmap::map(&file)? };
    let lines: Vec<&[u8]> = mmap.split(|&b| b == b'\n').collect();
    Ok(())
}