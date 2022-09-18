pub mod cpu {
    use crate::cartridge::cartridge;


    pub struct Mos6502<'a> {
        pub cart: &'a cartridge::Cartridge,

        //Registers
        pub acc: u8,
        pub ind_x: u8,
        pub ind_y: u8,
        pub stat: u8,
        pub stck_pnt: u8,
        pub prg_cnt: u16,

        //Internal RAM
        pub cpu_ram: Vec<u8>,
    }



    impl Mos6502<'_> {

        pub fn reset(&mut self) {
            self.stat = self.stat | 0x04;
            self.stck_pnt = self.stck_pnt - 3;
            self.prg_cnt = ((self.cart.cpu_read(0xFFFD) as u16) << 8) | (self.cart.cpu_read(0xFFFC) as u16);
        }


        pub fn execute_instruction(&mut self) -> u8 {
            let opcode = self.cart.cpu_read(self.prg_cnt);
            self.prg_cnt += 1;

            match opcode {
                0x69 => 0,
                _ => 0
            };
            0
        }



        fn adc() -> u8 {
            0
        }

    }
}