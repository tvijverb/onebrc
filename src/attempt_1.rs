#![feature(slice_pattern)]
use core::cmp::Ordering;
use std::fs::File;
use std::sync::Mutex;
use memmap2::Mmap;
// use fxhash::FxHashMap as HashMap;
use ahash::AHashMap as HashMap;
use rayon::prelude::*;

struct Data {
    mmap: Mmap,
    global_city_temps: Mutex<HashMap<String, Vec<f32>>>,
}

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

fn main() -> std::io::Result<()> {
    let file = File::open("measurements_full.txt")?;
    let mmap = unsafe { Mmap::map(&file)? };

    let data = Data {
        mmap,
        global_city_temps: Mutex::default(),
    };

    let lines: Vec<&[u8]> = data.mmap.par_split(|&b| b == b'\n').collect();
    let num_chunks = num_cpus::get();
    let chunk_size = (lines.len() + num_chunks - 1) / num_chunks;
    lines.par_chunks(chunk_size).for_each(|chunk| {
        let mut city_temps: HashMap<String, Vec<f32>> = HashMap::default();
        for line in chunk {
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
            let mut global_city_temps = data.global_city_temps.lock().unwrap();
            global_city_temps.entry(city).or_insert_with(Vec::new).extend(temps);
        }
    });

    let mut global_city_temps = data.global_city_temps.lock().unwrap();

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
