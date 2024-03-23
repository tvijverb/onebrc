use std::fs::File;
use std::sync::Mutex;

use memmap2::Mmap;
use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use ahash::AHashMap as HashMap;
use core::cmp::Ordering;


/// chunked parallel memory-mapped file + split new line
// best so far, +- 9.1s for the full 1B lines (13.8 GB)

#[derive(PartialEq, Clone)]
pub struct CityTemp<'a> {
    city: &'a str,
    min_temp: f32,
    max_temp: f32,
    mean_temp: f32,
    total_temp: f32,
    count: usize,
}

impl std::fmt::Display for CityTemp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}={:.1}/{:.1}/{:.1}", self.city, self.min_temp, self.mean_temp, self.max_temp)
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
    let global_city_temps: Mutex<HashMap<String, CityTemp>> = Mutex::default();

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
        let mut local_hashmap_citytemp: HashMap<String, CityTemp> = HashMap::default();

        // split chunk at newline character and iterate over the lines
        // split line at semicolon character and iterate over the fields
        // parse the city name and temperature
        chunk.split(|&b| b == b'\n').for_each(|line| {
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
                    local_hashmap_citytemp
                        .entry(city.to_string())
                        .and_modify(|exiisting_city|
                        {
                            exiisting_city.min_temp = exiisting_city.min_temp.min(temperature);
                            exiisting_city.max_temp = exiisting_city.max_temp.max(temperature);
                            exiisting_city.total_temp += temperature;
                            exiisting_city.count += 1;
                        })
                        .or_insert(CityTemp{
                            city: city,
                            min_temp: temperature,
                            max_temp: temperature,
                            mean_temp: temperature,
                            total_temp: temperature,
                            count: 1,
                        });
                }
        });

        let mut global_city_temps = global_city_temps.lock().unwrap();
        for (city, city_temp) in local_hashmap_citytemp {
            global_city_temps
                .entry(city)
                .and_modify(|existing_city| {
                    existing_city.min_temp = existing_city.min_temp.min(city_temp.min_temp);
                    existing_city.max_temp = existing_city.max_temp.max(city_temp.max_temp);
                    existing_city.total_temp += city_temp.total_temp;
                    existing_city.count += city_temp.count;
                })
                .or_insert(city_temp);
        }
    });

    // set mean temperature for each city
    let mut global_city_temps = global_city_temps.lock().unwrap();
    for (_, city_temp) in global_city_temps.iter_mut() {
        city_temp.mean_temp = city_temp.total_temp / city_temp.count as f32;
    }

    let mut cities: Vec<CityTemp> = global_city_temps.values_mut().into_iter().map(|city_temp| {
        city_temp.clone()
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