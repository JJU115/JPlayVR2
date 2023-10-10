pub mod console {
    use crate::cpu::cpu::Mos6502;
    use crate::cartridge::cartridge::Cartridge;

    pub struct Console<'a> {
        pub cartridge: &'a Cartridge,
        pub cpu: &'a mut Mos6502<'a>
    }


    impl Console<'_> {
        pub fn load_cartridge(&self, ) -> Result<String, &str> {
            Err("Not implemented")
        }


        pub fn start_console(&mut self) {
            //Perform a reset on the cpu and ppu
            self.cpu.reset();


            //Execute a single CPU instruction and receive the cycle count
            //Run the PPU for 3 times that many cycles
            loop {
                let cpu_cycles = self.cpu.execute_instruction();
            }
        }
    }
}