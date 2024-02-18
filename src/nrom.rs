pub mod nrom {
    use crate::cartridge::cartridge::Mapper;

    pub struct Nrom {
        prg_bank_mirror: u16 //One 32KB bank or two 16KB banks where 0xC000-0xFFFF mirrors 0x8000-0xBFFF
    }

    impl Mapper for Nrom {
        fn ppu_read(&self) -> usize {0}
        fn cpu_read(&self, addr: u16) -> usize {
            ((addr & self.prg_bank_mirror) - 0x8000) as usize
        }
        fn cpu_write(&self, _addr: u16, _value: u8) {}
        fn ppu_write(&self) {}
    }

    impl Nrom {
        pub fn new(prg_mirrored: bool) -> Nrom {
            Nrom {
                prg_bank_mirror: if prg_mirrored {0xBFFF} else {0xFFFF}
            }
        }
    }
}