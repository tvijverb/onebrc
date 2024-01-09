## One Billion Row challenge
# Language: Rust
# Rules: None, try to make the fastest implementation possible

link to the original challenge: [https://github.com/gunnarmorling/1brc](https://github.com/gunnarmorling/1brc)

# Hardware
- AMD 7840HS, Ubuntu Linux 23.10, balanced power profile

# Timing
- Lowest of 3 runs

# Optimizing the file reader
- BufReader::new + reader.lines()                          =>   real    0m35.496s
- no alloc BufReader + custom read_line                    =>   real    0m21.801s
- memory-mapped file + split new line + collect Vec<&[u8]> =>   real    0m17.810s
- memory-mapped file + split new line + for_each &[u8]     =>   real    0m9.517s
- chunked parallel memory-mapped file + split new line     =>   real    0m1.252s

# Attempt 1:
- memory-mapped file + split new line + collect Vec<&[u8]>
real    0m23.547s
user    1m50.033s
sys     0m46.479s

# Attempt 2:
- chunked parallel memory-mapped file + split new line
real    0m9.135s
user    1m40.257s
sys     0m6.188s

# Create Samples:
Compile the sample creator on your platform
```bash
cc -o create_sample create-sample.c -lm
```

And create the measurments file, rename it to measurements_full.txt and remove last empty line
```bash
./create_sample 1000000000
mv measurements.txt measurements_full.txt
sed -i '$d' measurements_full.txt
```
