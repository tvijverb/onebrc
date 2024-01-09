use std::fs::File;
use memmap2::Mmap;

/// memory-mapped file + split new line + collect Vec<&[u8]>
fn main() -> std::io::Result<()> {
    let file = File::open("measurements_full.txt")?;
    let mmap = unsafe { Mmap::map(&file)? };
    mmap.split(|&b| b == b'\n').for_each(|line| {
        // println!("{}", std::str::from_utf8(line).unwrap());
    });
    Ok(())
}