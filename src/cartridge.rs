pub mod cartridge {

    pub struct Cartridge {
        pub prg_rom: Vec<u8>,
        pub chr_rom: Vec<u8>,
    }


    impl Cartridge {

        /*
            Since all instructions require fetching two bytes, can just call this function once 
            from cpu to get those two bytes rather than having two separate calls to cpu_read
        */
        pub fn cpu_read(&self, addr: u16) -> u8 {
            0
        }

        pub fn cpu_write(&self, addr: u16, value: u8) {

        }

        pub fn ppu_read(&self) {

        }
    }
    




}