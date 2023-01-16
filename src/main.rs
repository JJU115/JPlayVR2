use std::vec;

pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod console;

fn main() {
    let cart = cartridge::cartridge::Cartridge {
        prg_rom: vec![],
        chr_rom: vec![]
    };


    let p = ppu::ppu::Ricoh2c02 {
        c: &cart,
    };


    let mut cp = cpu::cpu::Mos6502 {
        cart: &cart,
        acc: 0,
        ind_x: 0,
        ind_y: 0,
        stat: 0x34,
        stck_pnt: 0xFD,
        prg_cnt: 0xFFFC,
        cpu_ram: vec![0; 2048],
        extra_cycles: 0,
    };

    let nes  = console::console::Console {
        cartridge: &cart,
        cpu: &mut cp,
        ppu: &p
    };


    
}