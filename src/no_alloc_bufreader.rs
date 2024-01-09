use std::io::{self, prelude::*, BufReader};

/// no alloc BufReader + custom read_line()
fn main() -> std::io::Result<()> {
    let mut reader = my_reader::BufReader::open("measurements_full.txt")?;
    let mut buffer = String::new();

    while let Some(line) = reader.read_line(&mut buffer) {
        // println!("{}", line?.trim());
    }
    println!("Done reading!");
    Ok(())
}

mod my_reader {
    use std::{
        fs::File,
        io::{self, prelude::*},
    };

    pub struct BufReader {
        reader: io::BufReader<File>,
    }

    impl BufReader {
        pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);

            Ok(Self { reader })
        }

        pub fn read_line<'buf>(
            &mut self,
            buffer: &'buf mut String,
        ) -> Option<io::Result<&'buf mut String>> {
            buffer.clear();

            self.reader
                .read_line(buffer)
                .map(|u| if u == 0 { None } else { Some(buffer) })
                .transpose()
        }
    }
}