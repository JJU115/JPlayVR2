pub mod nrom {
    use crate::cartridge::cartridge::Mapper;

    pub struct Nrom {

    }

    impl Mapper for Nrom {
        fn ppu_read(&self) {

        }
        fn cpu_read(&self) {

        }
        fn cpu_write(&self) {

        }
        fn ppu_write(&self) {

        }
    }

    impl Nrom {
        pub fn new() -> Nrom {
            Nrom {}
        }
    }
}