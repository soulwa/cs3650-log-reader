# cs3650-log-reader
This script is used to analyze the log file produced as output in Northeastern University CS3650 Assignment 5 (Fall 2021).

## Analysis

This script will check the following:
- [x] all artists use unique colors when drawing
- [x] no two artists draw on the same spot
- [ ] the specified number of artists all exist in the program (by default, 54-- 4 main artists, and 50 rookies)
- [ ] all artists draw a pixel
- [ ] all artists draw the specified number of pixels (this might not need to happen, unsure if will be implemented) 
- [ ] artists don't have duplicated patterns (threads don't share random seeds)

## How to Use

The Khoury VM has Rust and `cargo` installed, so usage is simple:
```bash
$ git clone git@github.com:soulwa/cs3650-log-reader.git
$ cd cs3650-log-reader
$ cargo run <path-to-logfile>
```
In Vagrant, you can either install `cargo` using `apt-get install cargo`, or install rust with the command

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

and follow the installation prompt. From there, running the script is the same.
