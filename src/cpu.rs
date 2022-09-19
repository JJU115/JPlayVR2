pub mod cpu {
    use std::ops::Add;

    use crate::cartridge::cartridge;


    enum AddressingMode {
        Implied,
        Accumulator,
        Immediate,
        ZeroPage,
        Absolute,
        Relative,
        Indirect,
        AbsoluteX,
        AbsoluteY,
        ZeroPageIndex(u8),
        IndirectX,
        IndirectY
    }

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
            let addr = self.cart.cpu_read(0xFFFC, true);
            self.prg_cnt = ((addr.0 as u16) << 8) | (addr.1 as u16);
        }


        pub fn execute_instruction(&mut self) -> u8 {
            //Fetch the opcode and the next byte
            let opcode = self.cart.cpu_read(self.prg_cnt, true);
            self.prg_cnt += 2;

            let instr_cycles = match opcode.0 {
                //ADC
                0x69 => self.adc(&AddressingMode::Immediate, opcode.1),
                0x65 => self.adc(&AddressingMode::ZeroPage, opcode.1),
                0x75 => self.adc(&AddressingMode::ZeroPageIndex(self.ind_x), opcode.1),
                0x6D => self.adc(&AddressingMode::Absolute, opcode.1),
                0x7D => self.adc(&AddressingMode::AbsoluteX, opcode.1),
                0x79 => self.adc(&AddressingMode::AbsoluteY, opcode.1),
                0x61 => self.adc(&AddressingMode::IndirectX, opcode.1),
                0x71 => self.adc(&AddressingMode::AbsoluteY, opcode.1),

                //AND
                _ => 0
            };
            0
        }


        //Returns a tuple where 1st element is data to operate on, 2nd is the writeback address
        fn fetch_instruction_data(&mut self, mode: &AddressingMode, data: u8) -> (u8, u16) {
            match mode {
                AddressingMode::Accumulator => (self.acc, 0),
                AddressingMode::Immediate => (data, 0),
                AddressingMode::ZeroPage => (self.cpu_ram[data as usize], data as u16),
                AddressingMode::Absolute => {
                    let addr = (self.cart.cpu_read(self.prg_cnt, false).0 as u16) << 8 | data as u16;
                    self.prg_cnt += 1;
                    (self.cart.cpu_read(addr, false).0, addr) 
                },
                AddressingMode::AbsoluteX => {
                    let addr = (self.cart.cpu_read(self.prg_cnt, false).0 as u16) << 8 | (data + self.ind_x) as u16;
                    self.prg_cnt += 1;
                    (self.cart.cpu_read(addr, false).0, addr)
                },
                AddressingMode::AbsoluteY => {
                    let addr = (self.cart.cpu_read(self.prg_cnt, false).0 as u16) << 8 | (data + self.ind_y) as u16;
                    self.prg_cnt += 1;
                    (self.cart.cpu_read(addr, false).0, addr)
                },
                AddressingMode::ZeroPageIndex(ind) => (self.cpu_ram[(data + ind) as usize], data as u16),
                AddressingMode::IndirectX => {
                    let low = self.cart.cpu_read((data + self.ind_x) as u16, false).0;
                    let high = self.cart.cpu_read((data + self.ind_x + 1) as u16, false).0;
                    let addr = (high as u16) << 8 | low as u16;
                    (self.cart.cpu_read(addr, false).0, addr) 
                },
                AddressingMode::IndirectY => {
                    let low = self.cart.cpu_read(data as u16, false).0;
                    let high = self.cart.cpu_read((data + 1) as u16, false).0;
                    let addr = ((high as u16) << 8 | low as u16) + self.ind_y as u16;
                    (self.cart.cpu_read(addr, false).0, addr) 
                },
                _ => (0,0)
            }
        }



        fn adc(&mut self, mode: &AddressingMode, operand: u8) -> u8 {
            let data = self.fetch_instruction_data(mode, operand);
            let sum = self.acc + data.0 + self.stat & 0x01;
            if sum < self.acc || sum < data.0 {
                //set carry
            }
            0
        }



        fn asl(&mut self, mode: &AddressingMode, operand: u8) -> u8 {
            let data = self.fetch_instruction_data(mode, operand);
            0
        }

    }
}