pub mod ppu {
    use crate::cartridge::cartridge::Cartridge;


    pub struct Ricoh2c02<'a> {
        pub cart: &'a Cartridge,
        current_scanline: u16,
        scanline_cycle: u16,
        is_odd_cycle: bool,
    }



    impl Ricoh2c02<'_> {

        pub fn new(c: &Cartridge) -> Ricoh2c02 {
            Ricoh2c02 { 
                cart: c,
                current_scanline: 0,
                scanline_cycle: 0,
                is_odd_cycle: false
             }
        }

        pub fn reset(&self) {

        }

        //Based on the internal current cycle, perform one of several actions
        pub fn generate_signal(&self) {
            //PPU generates 262 scanlines per frame
            //Each scanline takes 341 PPU cycles, one pixel produced per cycle

            //Cycles 0-340 -- Pre-render scanline, this is one cycle shorter on odd frames

            //240 Visible scanlines

            //Post render scanline

            //20 VBlank scanlines

        }

        
    }

}