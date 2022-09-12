pub mod console {
    use crate::cpu::cpu::Mos6502;
    use crate::ppu::ppu::Ricoh2c02;
    use crate::cartridge::cartridge::Cartridge;

    pub struct Console<'a> {
        pub cartridge: &'a Cartridge,
        pub cpu: &'a mut Mos6502<'a>,
        pub ppu: &'a Ricoh2c02<'a>,
    }


    impl Console<'_> {
        fn load_cartridge(&self) {

        }


        fn play_game(&mut self) {
            //Perform a reset on the cpu and ppu
            self.cpu.reset();


            //Execute a single CPU instruction and receive the cycle count
            //Run the PPU for 3 times that many cycles
        }
    }
}