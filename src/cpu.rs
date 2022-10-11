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
        AbsoluteIndex(u8),
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
            self.prg_cnt = ((self.cart.cpu_read(0xFFFC) as u16) << 8) | (self.cart.cpu_read(0xFFFD) as u16);
        }


        fn writeback(&mut self, addr: u16, value: u8) {
            match addr {
                0x0000..=0x1FFF => {
                    self.cpu_ram[(addr & 0x07FF) as usize] = value;
                },
                0x2000..=0x3FFF => {
                    //PPU registers
                },
                0x4020..=0xFFFF => {
                    self.cart.cpu_write(addr, value);
                }
            }
        }


        pub fn execute_instruction(&mut self) -> u8 {
            //Fetch the opcode and the next byte
            let opcode = self.cart.cpu_read(self.prg_cnt);
            self.prg_cnt += 1;

            let instr_cycles = match opcode {
                //ADC
                0x69 => self.adc(&AddressingMode::Immediate),
                0x65 => self.adc(&AddressingMode::ZeroPage),
                0x75 => self.adc(&AddressingMode::ZeroPageIndex(self.ind_x)),
                0x6D => self.adc(&AddressingMode::Absolute),
                0x7D => self.adc(&AddressingMode::AbsoluteIndex(self.ind_x)),
                0x79 => self.adc(&AddressingMode::AbsoluteIndex(self.ind_y)),
                0x61 => self.adc(&AddressingMode::IndirectX),
                0x71 => self.adc(&AddressingMode::IndirectY),

                //AND
                _ => 0
            };
            0
        }


        //Returns a tuple where 1st element is data to operate on, 2nd is the writeback address
        fn fetch_instruction_data(&mut self, mode: &AddressingMode) -> (u8, u16) {
            let data = self.cart.cpu_read(self.prg_cnt);
            self.prg_cnt += 1;
            match mode {
                AddressingMode::Accumulator => (self.acc, 0),
                AddressingMode::Immediate => (data, 0),
                AddressingMode::ZeroPage => (self.cpu_ram[data as usize], data as u16),
                AddressingMode::Absolute => {
                    let addr = (self.cart.cpu_read(self.prg_cnt) as u16) << 8 | data as u16;
                    self.prg_cnt += 1;
                    (self.cart.cpu_read(addr), addr) 
                },
                AddressingMode::AbsoluteIndex(ind) => {
                    //Possible oops cycle here
                    let addr = (self.cart.cpu_read(self.prg_cnt) as u16) << 8 | (data + ind) as u16;
                    self.prg_cnt += 1;
                    (self.cart.cpu_read(addr), addr)
                },
                AddressingMode::ZeroPageIndex(ind) => (self.cpu_ram[(data + ind) as usize], data as u16),
                AddressingMode::IndirectX => {
                    let low = self.cart.cpu_read((data + self.ind_x) as u16);
                    let high = self.cart.cpu_read((data + self.ind_x + 1) as u16);
                    let addr = (high as u16) << 8 | low as u16;
                    (self.cart.cpu_read(addr), addr) 
                },
                AddressingMode::IndirectY => {
                    //oops cycle here possibly
                    let low = self.cart.cpu_read(data as u16);
                    let high = self.cart.cpu_read((data + 1) as u16);
                    let addr = ((high as u16) << 8 | low as u16) + self.ind_y as u16;
                    (self.cart.cpu_read(addr), addr) 
                },
                _ => (0,0)
            }
        }



        fn adc(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            let sum = self.acc + data.0 + self.stat & 0x01;
            self.stat &= 0xBE;
            //Carry
            if ((self.acc & 0x80) == (data.0 & 0x80)) && ((sum & 0x80) != (self.acc & 0x80)) {
                self.stat |= 0x40;
            }
            //Overflow
            if sum <= self.acc && sum <= data.0 {
                self.stat |= 0x01;
            }
            self.acc = sum;
            self.examine_status(self.acc);          
        }


        fn and(&mut self, mode: &AddressingMode) {
            self.acc &= self.fetch_instruction_data(mode).0;
            self.examine_status(self.acc);
        }


        fn asl(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x80) >> 7;
            data.0 = data.0 << 1;
            if let mode = AddressingMode::Accumulator {
                self.acc = data.0;
            } else {
                self.writeback(data.1, data.0)
            }
            self.examine_status(data.0);
        }


        fn bit(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x3D;
            self.stat |= (data.0 & 0xC0);
            if self.acc & data.0 == 0 {
                self.stat |= 0x02;
            }
        }

        //Branch instructions can all be handled in one function
        //If flag equals value, take the branch, remember branching is signed
        //Another potential 'oops' cycle here
        fn branch(&mut self, flag: u8, value: u8) -> u8 {
            0
        }


        //BRK instruction forcibly triggers interrupt
        fn brk(&mut self) {

        }

        //AND mask with status register
        fn clear(&mut self, mask: u8) {
            self.stat &= mask;
        }


        fn cmp(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.acc >= data.0 { self.stat |= 0x01; }
            if self.acc == data.0 { self.stat |= 0x02; }
            if (self.acc - data.0) & 0x80 == 0x80 { self.stat |= 0x80 }
            0
        }

        
        fn cpx(&mut self, mode: &AddressingMode, reg: u8) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.ind_x >= data.0 { self.stat |= 0x01; }
            if self.ind_x == data.0 { self.stat |= 0x02; }
            if (self.ind_x - data.0) & 0x80 == 0x80 { self.stat |= 0x80 }
            0
        }


        fn cpy(&mut self, mode: &AddressingMode, reg: u8) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.ind_y >= data.0 { self.stat |= 0x01; }
            if self.ind_y == data.0 { self.stat |= 0x02; }
            if (self.ind_y - data.0) & 0x80 == 0x80 { self.stat |= 0x80 }
            0
        }


        fn dec(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, data.0 - 1);
            self.examine_status(data.0 - 1);
            0
        }

        
        fn dex(&mut self, mode: &AddressingMode) -> u8 {
            self.ind_x -= 1;
            self.examine_status(self.ind_x);
            0
        }


        fn dey(&mut self, mode: &AddressingMode) -> u8 {
            self.ind_y -= 1;
            self.examine_status(self.ind_y);
            0
        }


        fn eor(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.acc ^= data.0;
            self.examine_status(self.acc);
            0
        }


        fn inc(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, data.0 + 1);
            self.examine_status(data.0 + 1);
            0
        }

        fn inx(&mut self, mode: &AddressingMode) -> u8 {
            self.ind_x += 1;
            self.examine_status(self.ind_x);
            0
        }

        fn iny(&mut self, mode: &AddressingMode) -> u8 {
            self.ind_y += 1;
            self.examine_status(self.ind_y);
            0
        }


        //JMP Instruction
        fn jmp(&mut self, mode: &AddressingMode) {

        }


        fn jsr(&mut self) {

        }


        fn lda(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.acc = data.0;
            self.examine_status(self.acc);
            0
        }

        
        fn ldx(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.ind_x = data.0;
            self.examine_status(self.ind_x);
            0
        }


        fn ldy(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.ind_y = data.0;
            self.examine_status(self.ind_y);
            0
        }


        fn lsr(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            let temp = data.0 >> 1;
            match mode {
                AddressingMode::Accumulator => {self.acc = temp},
                _ => {self.writeback(data.1, temp)}
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x01);
            self.examine_status(temp);
            0
        }


        //Does nothing!
        fn nop() {

        }

        
        fn ora(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            self.acc |= data.0;
            self.examine_status(self.acc);
            0
        }


        //Push instructions all as one function - push register onto stack
        //PHA, PHP
        fn push(&mut self, register: u8) {
            self.cpu_ram[(0x0100 + self.stck_pnt) as usize] = register;
            self.stck_pnt -= 1;
        }


        fn pla(&mut self, mode: &AddressingMode) -> u8 {
            self.acc = self.cpu_ram[(0x0100 + self.stck_pnt) as usize];
            self.stck_pnt += 1;
            self.examine_status(self.acc);
            0
        }

        fn plp(&mut self, mode: &AddressingMode) -> u8 {
            self.stat = self.cpu_ram[(0x0100 + self.stck_pnt) as usize];
            self.stck_pnt += 1;
            0
        }


        fn rol(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            let temp = data.0 << 1;
            match mode {
                AddressingMode::Accumulator => {self.acc = temp;},
                _ => {self.writeback(data.1, temp);}
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x01) << 7;
            self.examine_status(temp);
            0
        }

        fn ror(&mut self, mode: &AddressingMode, reg: u8) -> u8 {
            let data = self.fetch_instruction_data(mode);
            let temp = data.0 >> 1;
            match mode {
                AddressingMode::Accumulator => {self.acc = temp;},
                _ => {self.writeback(data.1, temp);}
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x01);
            self.examine_status(temp);
            0
        }


        fn rti() {

        }


        fn rts() {

        }
       
        fn sbc(&mut self, mode: &AddressingMode) -> u8 {
            let data = self.fetch_instruction_data(mode);
            let temp = (self.acc as u16) + ((data.0 as u16) ^ 0xFF) + (self.stat & 0x01) as u16;
            self.stat &= 0xBE;
            self.stat |= if temp & 0xFF00 > 0 {1} else {0};
            self.stat |= if (temp ^ self.acc as u16) & (temp ^ ((data.0 as u16) ^ 0xFF)) & 0x0080 > 0 {0x40} else {0};
            self.acc = (temp & 0xFF) as u8;
            self.examine_status(self.acc);
            0
        }


        //STA, STX, STY as one function
        fn store(&mut self, reg: u8, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, reg);
        }


        fn tax(&mut self) -> u8 {
            self.ind_x = self.acc;
            self.examine_status(self.ind_x);
            0
        }


        fn tay(&mut self) -> u8 {
            self.ind_y = self.acc;
            self.examine_status(self.ind_y);
            0
        }


        fn tsx(&mut self) -> u8 {
            self.ind_x = self.stat;
            self.examine_status(self.ind_x);
            0
        }


        fn txa(&mut self) -> u8 {
            self.acc = self.ind_x;
            self.examine_status(self.acc);
            0
        }


        fn txs(&mut self) -> u8 {
            self.stat = self.ind_x;
            0
        }


        fn tya(&mut self) -> u8 {
            self.acc = self.ind_y;
            self.examine_status(self.acc);
            0
        }


        //Checking zero and negative flags
        fn examine_status(&mut self, value: u8) {
            self.stat &= 0x7D;
            self.stat |= if value == 0 {0x02} else {0};
            self.stat |= if value & 0x80 == 0x80 {0x80} else {0}; 
        }

    }
}