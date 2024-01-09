use std::fs::File;
use std::sync::Mutex;

use memmap2::Mmap;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use ahash::AHashMap as HashMap;
use core::cmp::Ordering;


/// chunked parallel memory-mapped file + split new line
// best so far, +- 9.1s for the full 1B lines (13.8 GB)

#[derive(PartialEq)]
pub struct CityTemp<'a> {
    city: &'a str,
    min_temp: f32,
    max_temp: f32,
    mean_temp: f32,
}

impl std::fmt::Display for CityTemp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}={}/{}/{}", self.city, self.min_temp, self.mean_temp, self.max_temp)
    }
}

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
    let global_city_temps: Mutex<HashMap<String, Vec<f32>>> = Mutex::default();

    let data = Data {
        mmap: &mmap,
        chunk_size,
        offset: 0,
    };

    let data_chunks = data.into_iter().collect::<Vec<&[u8]>>();
    // println!("data_chunks len = {:?}", data_chunks.len());

    data_chunks.par_iter().for_each(|chunk| {
        // line below is nice for debugging, shows the number of characters per chunk
        // all chunks combined have 1B newline characters
        // println!("chunk.len() = {}", chunk.len());
        let mut city_temps: HashMap<String, Vec<f32>> = HashMap::default();
        for line in chunk.split(|&b| b == b'\n') {
            let mut fields = line.split(|&b| b == b';');
            let city: Option<&str> = fields.next().map(|city_field| {
                std::str::from_utf8(city_field).unwrap_or("")
            });
            let city_temp: Option<f32> = fields.next().and_then(|temp_field| {
                std::str::from_utf8(temp_field).ok().and_then(|temp_str| {
                    temp_str.parse::<f32>().ok()
                })
            });
            if let (Some(city), Some(temperature)) = (city, city_temp) {
                city_temps.entry(city.to_string()).or_insert_with(Vec::new).push(temperature);
            }
        }
        for (city, temps) in city_temps {
            let mut global_city_temps = global_city_temps.lock().unwrap();
            global_city_temps.entry(city).or_insert_with(Vec::new).extend(temps);
        }
    });
    let global_city_temps = global_city_temps.lock().unwrap();

    let mut cities: Vec<CityTemp> = global_city_temps.par_iter().map(|(city_name, temps)| {
        let min_temp = temps.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(&0.0);
        let max_temp = temps.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(&0.0);
        let mean_temp = temps.iter().sum::<f32>() / temps.len() as f32;
        CityTemp { city: &city_name, min_temp: *min_temp, max_temp: *max_temp, mean_temp }
    }).collect();

    cities.sort_by(|a, b| a.city.partial_cmp(b.city).unwrap_or(Ordering::Equal));

    print!("{{");
    for (idx, city) in cities.iter().enumerate() {
        print!("{}", city);
        if idx == cities.len() - 1 {
            print!("}}");
        } else {
            print!(", ");
        }
    }
    Ok(())
}