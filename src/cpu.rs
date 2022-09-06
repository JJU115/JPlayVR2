pub mod cpu {
    use crate::cartridge::cartridge;


    pub struct Mos6502<'a> {
        pub c: &'a cartridge::Cartridge,
    }
}