pub mod cartridge {

    pub struct Cartridge {
        pub prg_rom: Vec<u8>,
        pub chr_rom: Vec<u8>,
    }


    impl Cartridge {

        pub fn cpu_read(&self) {

        }


        pub fn ppu_read(&self) {

        }
    }
    




}