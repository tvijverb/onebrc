use std::fs::File;
use std::sync::Mutex;

use memmap2::Mmap;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use ahash::AHashMap as HashMap;
use core::cmp::Ordering;


/// chunked parallel memory-mapped file + split new line
// best so far, +- 9.1s for the full 1B lines (13.8 GB)

struct Data<'a> {
    mmap: &'a Mmap,
    chunk_size: usize,
    offset: usize,
}

impl<'a> Iterator for Data<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.mmap.len() {
            return None;
        }
        let current_chunk_end_char = std::cmp::min(self.offset + self.chunk_size, self.mmap.len());
        // find next new line character from current_chunk_end_char
        // also account for the case where there is no new line character in the current chunk
        // or the end of the file is reached
        let next_new_line_char = self.mmap[current_chunk_end_char..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(self.mmap.len() - current_chunk_end_char);
        let end = current_chunk_end_char + next_new_line_char;

        let chunk = &self.mmap[self.offset..end];
        self.offset = end + 1;
        Some(chunk)
    }
}

fn main() -> std::io::Result<()> {
    let file = File::open("measurements_full.txt")?;
    let mmap = unsafe { Mmap::map(&file)? };
    let num_chunks = num_cpus::get();
    let chunk_size = (mmap.len() + num_chunks - 1) / num_chunks;

    let data = Data {
        mmap: &mmap,
        chunk_size,
        offset: 0,
    };

    let data_chunks = data.into_iter().collect::<Vec<&[u8]>>();
    // println!("data_chunks len = {:?}", data_chunks.len());

    data_chunks.par_iter().for_each(|chunk| {
        for line in chunk.split(|&b| b == b'\n') {
            // std::str::from_utf8(line).ok();
        }
    });
    Ok(())
}