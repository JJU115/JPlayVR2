pub mod cartridge {

    pub struct Cartridge {
        pub prg_rom: Vec<u8>,
        pub chr_rom: Vec<u8>,
    }


    impl Cartridge {

        pub fn cpu_read(&self, addr: u16) -> u8 {
            0
        }


        pub fn ppu_read(&self) {

        }
    }
    




}