use std::vec;

pub mod cpu;
pub mod ppu;
pub mod cartridge;

fn main() {
    let cart = cartridge::cartridge::Cartridge {
        prg_rom: vec![],
        chr_rom: vec![]
    };


    let p = ppu::ppu::Ricoh2c02 {
        c: &cart,
    };


    let cp = cpu::cpu::Mos6502 {
        c: &cart
    };


    p.c.ppu_read();
    cp.c.cpu_read();
}