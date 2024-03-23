# One Billion Row challenge
## Language: Rust
## Rules: None, try to make the fastest implementation possible

link to the original challenge: [https://github.com/gunnarmorling/1brc](https://github.com/gunnarmorling/1brc)

## Hardware
- AMD 7840HS, Ubuntu Linux 23.10, balanced power profile
- SK Hynix 512 GB PCIE 4.0 SSD (HFS512GEJ9X115N)
- 32 GB RAM DDR5 6400 MT/s

```bash
sudo hdparm -Tt /dev/nvme0n1
/dev/nvme0n1:
 Timing cached reads:   53416 MB in  1.99 seconds = 26844.81 MB/sec
 Timing buffered disk reads: 7084 MB in  3.00 seconds = 2361.16 MB/sec
```

## Timing
- Lowest of 3 runs

## Optimizing the file reader
The following techniques have been tested to get the fastest possible file read speed.

- (1) BufReader::new + reader.lines()                          =>   real    0m35.496s
- (2) no alloc BufReader + custom read_line                    =>   real    0m21.801s
- (3) memory-mapped file + split new line + collect Vec<&[u8]> =>   real    0m17.810s
- (4) memory-mapped file + split new line + for_each &[u8]     =>   real    0m9.517s
- (5) chunked parallel memory-mapped file + split new line     =>   real    0m1.252s

(5) Was the fastest, reading the full 13.8 GB file in 1.252 seconds. Averaging read speeds of over 11 GB/s.
Note that this is just reading the file, it still needs to be processed. See below for the results.

# Rust Solving Attempts
Attempt 2 is the fastest with 9.135 seconds. This time would be 5th place on the original Java challenge leaderboard. (early jan 2024)

## Attempt 1:
See attempt_1.rs. Using memory-mapped file + split new line + collect Vec<&[u8]> => 23.547s

## Attempt 2:
See attempt_2.rs chunked parallel memory-mapped file + split new line => 9.135s

## Attempt 3:
See main.rs chunked parallel memory-mapped file + sum of temperatures => 6.713s

## Create Samples:
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

# Measure runtime
```bash
time <binary_name>
```