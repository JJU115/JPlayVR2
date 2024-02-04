pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod console;
pub mod nrom;

use std::env;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Single argument required");
        std::process::exit(0);
    } 
  

    let cart = match cartridge::cartridge::Cartridge::load_rom(&args[1]) {
        Ok(cart) => cart,
        Err(msg) => {
            println!("{}", msg);
            std::process::exit(-1);
        }
    };

    let p = ppu::ppu::Ricoh2c02::new(&cart);
    let mut nes  = console::console::Console {
        cpu: &mut cpu::cpu::Mos6502::new(&cart, &p),
    };
    
    nes.start_console();
    
}