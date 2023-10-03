pub mod ppu {
    use crate::cartridge::cartridge::Cartridge;


    pub struct Ricoh2c02<'a> {
        pub c: &'a Cartridge,
    }



    impl Ricoh2c02<'_> {

        pub fn new(c: &Cartridge) -> Ricoh2c02 {
            Ricoh2c02 { 
                c: c
             }
        }

        pub fn reset(&self) {

        }


        pub fn generate_signal(&self) {
            
        }
    }

}