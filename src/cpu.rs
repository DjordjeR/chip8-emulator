use crate::Memory;
use rand::rngs::ThreadRng;
use rand::Rng;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub const REGISTER_COUNT: usize = 16;

#[derive(Debug)]
pub enum Instruction {
    // Clear the display
    CLS,
    // Return from a subroutine.
    RET,
    // Jump to addr
    JP(u16),
    // Set reg Vx to value NN. (Vx = N)
    LD(u8, u8),
    // Add reg and value
    ADD(u8, u8),
    // Call subroutine at addr.
    CALL(u16),
    // Skip next instruction if reg == value.
    SE(u8, u8),
    // Skip next instruction if reg != value.
    SNE(u8, u8),
    // Stores the value of reg Vy in reg Vx.
    LDR(u8, u8),
    // Set Vx = Vx XOR Vy.
    XOR(u8, u8),
    // Set Vx = Vx AND Vy
    AND(u8, u8),
    // Set I = addr.
    LDI(u16),
    // DRW Vx, Vy, nibble
    DRW(u8, u8, u8),
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    LDB(u8),
    // Read registers V0 through Vx from memory starting at location I.
    LDV(u8),
    // Set I = location of sprite for digit Vx.
    LDF(u8),
    // Store registers V0 through Vx in memory starting at location I.
    LDRM(u8),
    // Set delay timer = Vx.
    LDDT(u8),
    //Set sound timer = Vx.
    LDST(u8),
    // Set Vx = random byte AND kk.
    RND(u8, u8),
    // Set Vx = delay timer value.
    DTLD(u8),
    // Set I = I + Vx.
    ADDI(u8),
    // Set Vx = Vx + Vy, set VF = carry.
    ADDC(u8, u8),
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    SUB(u8, u8),

    Unknown(u8, u8),
}

#[derive(Debug)]
pub struct CPU {
    pc: u16, // Program counter
    stack: Vec<u16>,
    vram: [[u8; DISPLAY_HEIGHT as usize]; DISPLAY_WIDTH as usize],
    registers: [u8; REGISTER_COUNT],
    i: u16, // 16-bit index reg used to point at locations in memory
    delay_timer: u8,
    sound_timer: u8,
    rnd: ThreadRng,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0,
            stack: Vec::<u16>::new(),
            vram: [[0; DISPLAY_HEIGHT as usize]; DISPLAY_WIDTH as usize],
            registers: [0; REGISTER_COUNT],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            rnd: rand::thread_rng(),
        }
    }
    pub fn from_pc(pc: u16) -> Self {
        let mut cpu = CPU::new();
        cpu.pc = pc;
        cpu
    }
    pub fn fetch(&mut self, memory: &Memory) -> (u8, u8) {
        let (high, low) = (memory.read(self.pc), memory.read(self.pc + 1));
        self.pc += 2;
        (high, low)
    }

    pub fn decode(&self, high: u8, low: u8) -> Instruction {
        let code = high >> 4;
        let reg = high & 0xf;

        let addr = ((high as u16 & 0xf as u16) << 8 as u16) | low as u16;

        match code {
            0x0 => match low {
                0xe0 => Instruction::CLS,
                0xee => Instruction::RET,
                _ => Instruction::Unknown(high, low),
            },
            0x1 => Instruction::JP(addr),
            0x2 => Instruction::CALL(addr),
            0x3 => Instruction::SE(reg, low),
            0x4 => Instruction::SNE(reg, low),
            0x6 => Instruction::LD(reg, low),
            0x7 => Instruction::ADD(reg, low),
            0x8 => {
                let variant = low & 0xf;
                let y = low >> 4;
                match variant {
                    0x0 => Instruction::LDR(reg, y),
                    0x2 => Instruction::AND(reg, y),
                    0x3 => Instruction::XOR(reg, y),
                    0x4 => Instruction::ADDC(reg, y),
                    0x5 => Instruction::SUB(reg, y),
                    _ => Instruction::Unknown(high, low),
                }
            }
            0xa => Instruction::LDI(addr),
            0xc => Instruction::RND(reg, low),
            0xd => {
                let y = low >> 4;
                let n = low & 0xf;
                Instruction::DRW(reg, y, n)
            }
            0xf => match low {
                0x07 => Instruction::DTLD(reg),
                0x15 => Instruction::LDDT(reg),
                0x18 => Instruction::LDST(reg),
                0x33 => Instruction::LDB(reg),
                0x1e => Instruction::ADDI(reg),
                0x29 => Instruction::LDF(reg),
                0x55 => Instruction::LDRM(reg),
                0x65 => Instruction::LDV(reg),
                _ => Instruction::Unknown(high, low),
            },
            _ => Instruction::Unknown(high, low),
        }
    }

    pub fn execute(&mut self, memory: &mut Memory, instruction: &Instruction) {
        self.decrement_timers();
        // Execute the instruction
        match instruction {
            Instruction::CLS => self.clear_screen(),
            Instruction::RET => self.pc = self.stack.pop().unwrap(),
            Instruction::JP(addr) => self.pc = *addr,
            Instruction::LD(reg, value) => self.registers[*reg as usize] = *value,
            Instruction::ADD(reg, value) => {
                self.registers[*reg as usize] = self.registers[*reg as usize].wrapping_add(*value);
            }
            Instruction::CALL(addr) => {
                self.stack.push(self.pc);
                self.pc = *addr
            }
            Instruction::SE(reg, value) => {
                if self.registers[*reg as usize] == *value {
                    self.pc += 2;
                }
            }
            Instruction::SNE(reg, value) => {
                if self.registers[*reg as usize] != *value {
                    self.pc += 2;
                }
            }
            Instruction::LDI(addr) => self.i = *addr,
            Instruction::RND(reg, value) => {
                let mut r = self.rnd.gen::<u8>();
                println!("Random {} {}", r, *value);
                r &= *value;
                self.registers[*reg as usize] = r;
            }
            Instruction::LDR(x, y) => self.registers[*x as usize] = self.registers[*y as usize],
            Instruction::XOR(x, y) => self.registers[*x as usize] ^= self.registers[*y as usize],
            Instruction::AND(x, y) => {
                self.registers[*x as usize] =
                    self.registers[*x as usize] & self.registers[*y as usize];
            }
            Instruction::DRW(x, y, n) => {
                self.registers[0xf] = 0;
                // For n rows
                for byte in 0..*n {
                    let y = (self.registers[*y as usize] + byte) as usize % DISPLAY_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.registers[*x as usize] + bit) as usize % DISPLAY_WIDTH;
                        let is_on = (memory.read(self.i + byte as u16) >> (7 - bit)) & 0x1;
                        self.registers[0xf] = is_on & self.vram[x as usize][y as usize];
                        self.vram[x as usize][y as usize] ^= is_on;
                    }
                }
            }
            Instruction::LDB(x) => {
                let mut val = self.registers[*x as usize];
                memory.write(self.i + 2, val % 10);
                val /= 10;
                memory.write(self.i + 1, val % 10);
                val /= 10;
                memory.write(self.i, val % 10);
            }
            Instruction::LDF(x) => {
                self.i = self.registers[*x as usize] as u16 * 5;
            }
            Instruction::LDV(x) => {
                for i in 0..*x {
                    println!("Mem: {}", memory.read(self.i + i as u16));
                    self.registers[i as usize] = memory.read(self.i + i as u16);
                }
            }
            Instruction::LDDT(x) => {
                self.delay_timer = self.registers[*x as usize];
            }
            Instruction::LDST(x) => {
                self.sound_timer = self.registers[*x as usize];
            }
            Instruction::DTLD(reg) => {
                self.registers[*reg as usize] = self.delay_timer;
            }
            Instruction::LDRM(reg) => {
                for i in 0..*reg {
                    memory.write(self.i + i as u16, self.registers[i as usize]);
                }
            }
            Instruction::ADDI(reg) => {
                self.i += self.registers[*reg as usize] as u16;
            }
            Instruction::ADDC(reg, y) => {
                let res = self.registers[*reg as usize].checked_add(self.registers[*y as usize]);
                if res == None {
                    self.registers[0xf] = 0x1;
                    return;
                }
                self.registers[*reg as usize] = res.unwrap();
            }
            Instruction::SUB(reg, y) => {
                let x = self.registers[*reg as usize];
                let y = self.registers[*y as usize];
                if x > y {
                    self.registers[0xf] = 0x1;
                    self.registers[*reg as usize] = x - y;
                }
            }
            Instruction::Unknown(high, low) => {
                use std::{thread, time::Duration};
                thread::sleep(Duration::from_secs(15));
                panic!("Unknown instruction 0x{:02x}{:02x}", high, low);
            }
        }
    }
    pub fn clear_screen(&mut self) {
        for i in 0..DISPLAY_WIDTH {
            for j in 0..DISPLAY_HEIGHT {
                self.vram[i][j] = 0;
            }
        }
    }
    pub fn play_sound(&self) -> bool {
        self.sound_timer > 0
    }
    pub fn vram(&self) -> &[[u8; DISPLAY_HEIGHT]; DISPLAY_WIDTH] {
        &self.vram
    }

    pub fn pp(&self) {
        println!("Registers:");
        println!("-----------------------------");
        for i in 0..self.registers.len() {
            print!("V{0:<2}|", i);
        }
        print!("I  |");
        println!();
        for i in 0..self.registers.len() {
            print!("{0:<3}|", self.registers[i]);
        }
        print!("{:#01x}|", self.i);
        println!();

        print!("Sound: {:#01x}|", self.sound_timer);
        print!("Delay: {:#01x}|", self.delay_timer);
    }

    fn decrement_timers(&mut self) {
        // TODO: This only works since we are running at 60fps
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

#[test]
fn test_digits() {
    let mut n = 156;

    println!("3 {}", n % 10);
    n /= 10;
    println!("2 {}", n % 10);
    n /= 10;
    println!("1 {}", n % 10);
}
