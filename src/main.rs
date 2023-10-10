pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod console;

fn main() {
    let cart = cartridge::cartridge::Cartridge {
        prg_rom: vec![],
        chr_rom: vec![]
    };


    let p = ppu::ppu::Ricoh2c02::new(&cart);
    let mut cp = cpu::cpu::Mos6502::new(&cart, &p);

    let mut nes  = console::console::Console {
        cartridge: &cart,
        cpu: &mut cp,
    };

    nes.load_cartridge();
    
    nes.start_console();
    
}