use std::fs::File;
use std::io::Read;
use std::path::Path;

pub const MEMORY_SIZE: usize = 4096;
pub const MEMORY_START_OFFSET: usize = 0x200;

const TEXT_SPRITE: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug)]
pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

// TODO: Implement disply or something

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn from_rom(file_name: String) -> Self {
        // Read memory from the file
        // TODO: Maybe check if file exists
        let rom_path = Path::new(&file_name);
        let mut file = File::open(&rom_path).unwrap();
        let mut buffer = Vec::new();
        // Read all data from the file
        file.read_to_end(&mut buffer).unwrap();
        // initialize memory
        let mut mem = Memory::new();

        // Copy font sprite
        mem.memory[..TEXT_SPRITE.len()].clone_from_slice(&TEXT_SPRITE);

        // Copy rom to memory, start at MEMORY_OFFEST = 0x200
        for i in MEMORY_START_OFFSET..buffer.len() + MEMORY_START_OFFSET {
            mem.memory[i] = buffer[i - MEMORY_START_OFFSET];
        }
        mem
    }

    pub fn read(&self, addr: u16) -> u8 {
        // TODO: ADd checks if the addr is valid
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value
    }
}
