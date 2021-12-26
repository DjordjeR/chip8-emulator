# Chip8 Emulator - WIP

Just another Chip8 emulator written in Rust. This is my first time doing
something remotley serious in Rust, it is also my first time writing an emulator.

It is still work in progress and not all instructions are implemented. None of 
the keyboard instructions work, which means that you cannot play any games. 

## Running 

Use rusts cargo to build and run this project. Only tested on Windows 10, 
however it should work fine on other systems, since **ggez** is a cross platform
graphics library, and none of the other dependencies are system dependant.

Rust version: *rustc 1.57.0*
Cargo version: *cargo 1.57.0*

```shell
cargo build
cargo run 
cargo run --release
```

## Disclamer

None of the roms in the data directory are mine, or written by me. I just found
some collections online.