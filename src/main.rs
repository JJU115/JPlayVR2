pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod console;

use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Single argument required");
        std::process::exit(0);
    } 
  

    let cart = cartridge::cartridge::Cartridge {
        prg_rom: vec![],
        chr_rom: vec![],
        mapper_num: 0
    };


    let p = ppu::ppu::Ricoh2c02::new(&cart);
    let mut cp = cpu::cpu::Mos6502::new(&cart, &p);

    let mut nes  = console::console::Console {
        cartridge: &cart,
        cpu: &mut cp,
    };

     if let Err(msg) = nes.load_cartridge(&args[1]) {
        println!("{}", msg);
        std::process::exit(-1);
     }
    
    nes.start_console();
    
}