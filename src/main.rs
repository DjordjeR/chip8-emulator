mod cpu;
mod memory;

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color};
use ggez::{input, timer, Context, ContextBuilder, GameResult};
use glam::*;

use cpu::{CPU, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use memory::{Memory, MEMORY_START_OFFSET};

const DISPLAY_GRID_SIZE: (i16, i16) = (DISPLAY_WIDTH as i16, DISPLAY_HEIGHT as i16);
const DISPLAY_PIXEL_SIZE: (i16, i16) = (25, 25);

const DISPLAY_SCREEN_SIZE: (f32, f32) = (
    DISPLAY_GRID_SIZE.0 as f32 * DISPLAY_PIXEL_SIZE.0 as f32,
    DISPLAY_GRID_SIZE.1 as f32 * DISPLAY_PIXEL_SIZE.1 as f32,
);

const DISPLAY_MAX_FPS: u32 = 60;

const KEYBOARD_KEYS: [KeyCode; 16] = [
    KeyCode::Key0,
    KeyCode::Key1,
    KeyCode::Key2,
    KeyCode::Key3,
    KeyCode::Key4,
    KeyCode::Key5,
    KeyCode::Key6,
    KeyCode::Key7,
    KeyCode::Key8,
    KeyCode::Key9,
    KeyCode::A,
    KeyCode::B,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::E,
    KeyCode::F,
];

struct Chip8 {
    memory: Memory,
    cpu: CPU,
}

impl Chip8 {
    pub fn from_rom(rom: String) -> Chip8 {
        Chip8 {
            memory: Memory::from_rom(rom),
            cpu: CPU::from_pc(MEMORY_START_OFFSET as u16),
        }
    }

    fn check_keys(&mut self, ctx: &mut Context) {
        for i in 0..KEYBOARD_KEYS.len() {
            if input::keyboard::is_key_pressed(ctx, KEYBOARD_KEYS[i]) {
                self.cpu.set_key(i);
            } else {
                self.cpu.reset_key(i);
            }
        }
    }
}

impl event::EventHandler<ggez::GameError> for Chip8 {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, DISPLAY_MAX_FPS) {
            // Fetch instruction from memoery, and increment pc
            println!("---------  START  -----------");
            let (high, low) = self.cpu.fetch(&self.memory);
            println!("Fetched:  0x{:02x}{:02x}", high, low);
            // Decode the fetched instruction
            let instruction = self.cpu.decode(high, low);
            println!("Decoded: {:?}", instruction);
            self.check_keys(ctx);
            // Execute instruction
            self.cpu.execute(&mut self.memory, &instruction);

            self.cpu.pp();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.cpu.vram()[x as usize][y as usize] == 0x0 {
                    continue;
                }

                let bound = graphics::Rect::new_i32(
                    x as i32 * DISPLAY_PIXEL_SIZE.0 as i32,
                    y as i32 * DISPLAY_PIXEL_SIZE.1 as i32,
                    DISPLAY_PIXEL_SIZE.0 as i32,
                    DISPLAY_PIXEL_SIZE.1 as i32,
                );

                let rect = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    bound,
                    Color::WHITE,
                )
                .unwrap();
                graphics::draw(ctx, &rect, (ggez::mint::Point2 { x: 0.0, y: 0.0 },)).unwrap();
            }
        }
        // TODO: check if sound should be played and play it here
        graphics::present(ctx).unwrap();
        ggez::timer::yield_now();
        Ok(())
    }
}

fn main() {
    let (ctx, events_loop) = ContextBuilder::new("chip8", "Djordje Rajic")
        .window_setup(ggez::conf::WindowSetup::default().title("Chip8 Emulator"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(DISPLAY_SCREEN_SIZE.0, DISPLAY_SCREEN_SIZE.1),
        )
        .build()
        .expect("Could not create ggez context!");

    let chip8 = Chip8::from_rom("data/PONG".to_string());
    event::run(ctx, events_loop, chip8);
}

#[test]
fn test_0x6_bit_masking() {
    let high = 0x61;
    let low = 0x1f;

    println!("Instruction 0x{:02x}", high >> 4);
    println!("Register 0x{:02x}", high & 0xf);
    println!("Value 0x{:02x}", low);
    assert_eq!(0x6, high >> 4);
    assert_eq!(0x1, high & 0xf);
    assert_eq!(0x1f, low);
}

#[test]
fn test_0x1_bit_masking() {
    let high = 0x12;
    let low = 0x08;
    println!("Instruction 0x{:02x}", high >> 4);
    println!("ADDR 0x{:x}", ((high & 0xf) << 8) | low);
}

#[test]
fn test_0x8xy2_bit_masking() {
    let high = 0x8a;
    let low = 0x12;
    println!("Instruction 0x{:02x}", high >> 4);
    println!("X 0x{:x}", high & 0xf);
    println!("Y 0x{:x}", low >> 4);
    println!("Low 0x{:x}", low & 0xf);
}
