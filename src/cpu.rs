pub mod cpu {
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
                0x4000..=0x4017 => {
                    //APU registers
                }
                0x4020..=0xFFFF => {
                    self.cart.cpu_write(addr, value);
                },
                _ => ()
            }
        }


        pub fn execute_instruction(&mut self) -> u8 {
            //Fetch the opcode and the next byte
            let opcode = self.cart.cpu_read(self.prg_cnt);
            self.prg_cnt += 1;

            match opcode {
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
                0x29 => self.and(&AddressingMode::Immediate),
                0x25 => self.and(&AddressingMode::ZeroPage),
                0x35 => self.and(&AddressingMode::ZeroPageIndex(self.ind_x)),
                0x2D => self.and(&AddressingMode::Absolute),
                0x3D => self.and(&AddressingMode::AbsoluteIndex(self.ind_x)),
                0x39 => self.and(&AddressingMode::AbsoluteIndex(self.ind_y)),
                0x21 => self.and(&AddressingMode::IndirectX),
                0x31 => self.and(&AddressingMode::IndirectY),

                //Branches
                0x90 | 0xB0 | 0xF0 | 0x30 | 0xD0 | 0x10 | 0x50 | 0x70 => self.branch(opcode >> 6, (opcode >> 5) &0x01),

                _ => ()
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
                    if (data + ind) as u16 > 255 {
                        //'oops' cycle
                    }

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
                    if (low + self.ind_y) as u16 > 255 {
                        //'oops' cycle
                    }
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
            let mut data = self.fetch_instruction_data(mode);
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
            self.stat |= data.0 & 0xC0;
            if self.acc & data.0 == 0 {
                self.stat |= 0x02;
            }
        }

        //Branch instructions can all be handled in one function
        //If flag equals value, take the branch, remember branching is signed
        //Another potential 'oops' cycle here
        fn branch(&mut self, flag: u8, value: u8) {
            let comp: u8 = match flag {
                0 => self.stat >> 7,
                1 => (self.stat & 0x40) >> 6,
                2 => self.stat & 0x01,
                3 => (self.stat & 0x02) >> 1,
                _ => 0
            };

            let op = self.cart.cpu_read(self.prg_cnt);
            self.prg_cnt += 1;

            if comp == value {
                if ((self.prg_cnt + op as u16) & 0x0F00) != (self.prg_cnt & 0x0F00) {
                    //Add cycle
                }
                self.prg_cnt += op as u16;
                //Add cycle
            }
        }


        //BRK instruction forcibly triggers interrupt
        /*
            #  address R/W description
            --- ------- --- -----------------------------------------------
            1    PC     R  fetch opcode, increment PC
            2    PC     R  read next instruction byte (and throw it away),
                            increment PC
            3  $0100,S  W  push PCH on stack, decrement S
            4  $0100,S  W  push PCL on stack, decrement S
            *** At this point, the signal status determines which interrupt vector is used ***
            5  $0100,S  W  push P on stack (with B flag set), decrement S
            6   $FFFE   R  fetch PCL, set I flag
            7   $FFFF   R  fetch PCH
        */
        fn brk(&mut self) {
            self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] = ((self.prg_cnt & 0xFF00) >> 8) as u8;
            self.cpu_ram[(0x0100 + (self.stck_pnt - 1) as u16) as usize] = (self.prg_cnt & 0xFF) as u8;
            self.stck_pnt -= 2;

            self.stat |= 0x10;
            self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] = self.stat | 0x30;
            self.stck_pnt -= 1;

            self.prg_cnt = self.cart.cpu_read(0xFFFE) as u16 | (self.cart.cpu_read(0xFFFF) << 8) as u16;
            self.stat |= 0x04;
        }

        //AND mask with status register
        fn clear(&mut self, mask: u8) {
            self.stat &= mask;
        }


        fn cmp(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.acc >= data.0 { self.stat |= 0x01; }
            if self.acc == data.0 { self.stat |= 0x02; }
            if (self.acc - data.0) & 0x80 == 0x80 { self.stat |= 0x80 }
        }

        
        fn cpx(&mut self, mode: &AddressingMode, reg: u8) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.ind_x >= data.0 { self.stat |= 0x01; }
            if self.ind_x == data.0 { self.stat |= 0x02; }
            if (self.ind_x - data.0) & 0x80 == 0x80 { self.stat |= 0x80 }
        }


        fn cpy(&mut self, mode: &AddressingMode, reg: u8) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.ind_y >= data.0 { self.stat |= 0x01; }
            if self.ind_y == data.0 { self.stat |= 0x02; }
            if (self.ind_y - data.0) & 0x80 == 0x80 { self.stat |= 0x80 }
        }


        fn dec(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, data.0 - 1);
            self.examine_status(data.0 - 1);
        }

        
        fn dex(&mut self, mode: &AddressingMode) {
            self.ind_x -= 1;
            self.examine_status(self.ind_x);
        }


        fn dey(&mut self, mode: &AddressingMode) {
            self.ind_y -= 1;
            self.examine_status(self.ind_y);
        }


        fn eor(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.acc ^= data.0;
            self.examine_status(self.acc);
        }


        fn inc(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, data.0 + 1);
            self.examine_status(data.0 + 1);
        }

        fn inx(&mut self, mode: &AddressingMode) {
            self.ind_x += 1;
            self.examine_status(self.ind_x);
        }

        fn iny(&mut self, mode: &AddressingMode) {
            self.ind_y += 1;
            self.examine_status(self.ind_y);
        }


        //JMP Instruction - Absolute or indirect addressing modes
        fn jmp(&mut self, mode: &AddressingMode) {
            if let AddressingMode::Absolute = mode {
                self.prg_cnt = (self.cart.cpu_read(self.prg_cnt + 1) << 8) as u16 | self.cart.cpu_read(self.prg_cnt) as u16;
            } else {
                let target = (self.cart.cpu_read(self.prg_cnt + 1) << 8) as u16 | 
                    self.cart.cpu_read(self.prg_cnt) as u16;
                
                self.prg_cnt = (self.cart.cpu_read(target + 1) << 8) as u16 | self.cart.cpu_read(target) as u16;
            }
        }

        /*
             #  address R/W description
            --- ------- --- -------------------------------------------------
            1    PC     R  fetch opcode, increment PC
            2    PC     R  fetch low address byte, increment PC
            3  $0100,S  R  internal operation (predecrement S?)
            4  $0100,S  W  push PCH on stack, decrement S
            5  $0100,S  W  push PCL on stack, decrement S
            6    PC     R  copy low address byte to PCL, fetch high address
                            byte to PCH
        */
        fn jsr(&mut self) {
            self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] = ((self.prg_cnt & 0xFF00) >> 8) as u8;
            self.cpu_ram[(0x0100 + (self.stck_pnt - 1) as u16) as usize] = (self.prg_cnt & 0xFF) as u8;
            self.stck_pnt -= 2;

            self.prg_cnt = (self.cart.cpu_read(self.prg_cnt + 1) << 8) as u16 | self.cart.cpu_read(self.prg_cnt) as u16;
        }


        fn lda(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.acc = data.0;
            self.examine_status(self.acc);
        }

        
        fn ldx(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.ind_x = data.0;
            self.examine_status(self.ind_x);
        }


        fn ldy(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.ind_y = data.0;
            self.examine_status(self.ind_y);
        }


        fn lsr(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            let temp = data.0 >> 1;
            match mode {
                AddressingMode::Accumulator => {self.acc = temp},
                _ => {self.writeback(data.1, temp)}
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x01);
            self.examine_status(temp);
        }


        //Does nothing!
        fn nop() {

        }

        
        fn ora(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.acc |= data.0;
            self.examine_status(self.acc);
        }


        //Push instructions all as one function - push register onto stack
        //PHA, PHP
        fn push(&mut self, register: u8) {
            self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] = register;
            self.stck_pnt -= 1;
        }


        fn pla(&mut self, mode: &AddressingMode) {
            self.stck_pnt += 1;
            self.acc = self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize];         
            self.examine_status(self.acc);
        }

        fn plp(&mut self, mode: &AddressingMode) {
            self.stck_pnt += 1;
            self.stat = self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize];          
        }


        fn rol(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            let temp = data.0 << 1;
            match mode {
                AddressingMode::Accumulator => {self.acc = temp;},
                _ => {self.writeback(data.1, temp);}
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x01) << 7;
            self.examine_status(temp);
        }

        fn ror(&mut self, mode: &AddressingMode, reg: u8) {
            let data = self.fetch_instruction_data(mode);
            let temp = data.0 >> 1;
            match mode {
                AddressingMode::Accumulator => {self.acc = temp;},
                _ => {self.writeback(data.1, temp);}
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x01);
            self.examine_status(temp);
        }

        /*
             #  address R/W description
        --- ------- --- -----------------------------------------------
            1    PC     R  fetch opcode, increment PC
            2    PC     R  read next instruction byte (and throw it away)
            3  $0100,S  R  increment S
            4  $0100,S  R  pull P from stack, increment S
            5  $0100,S  R  pull PCL from stack, increment S
            6  $0100,S  R  pull PCH from stack
        */
        fn rti(&mut self) {
            self.stck_pnt += 1;
            self.stat = self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize];
            self.prg_cnt = self.cpu_ram[(0x0101 + self.stck_pnt as u16) as usize] as u16 | (self.cpu_ram[(0x0102 + self.stck_pnt as u16) as usize] << 8) as u16;
            self.stck_pnt += 2;
        }

        /*
            #  address R/W description
        --- ------- --- -----------------------------------------------
            1    PC     R  fetch opcode, increment PC
            2    PC     R  read next instruction byte (and throw it away)
            3  $0100,S  R  increment S
            4  $0100,S  R  pull PCL from stack, increment S
            5  $0100,S  R  pull PCH from stack
            6    PC     R  increment PC
        */
        fn rts(&mut self) {
            self.stck_pnt += 1;
            self.prg_cnt = self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] as u16 | (self.cpu_ram[(0x0101 + self.stck_pnt as u16) as usize] << 8) as u16;
            self.stck_pnt += 1;
            self.prg_cnt += 1;
        }
       
        fn sbc(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            let temp = (self.acc as u16) + ((data.0 as u16) ^ 0xFF) + (self.stat & 0x01) as u16;
            self.stat &= 0xBE;
            self.stat |= if temp & 0xFF00 > 0 {1} else {0};
            self.stat |= if (temp ^ self.acc as u16) & (temp ^ ((data.0 as u16) ^ 0xFF)) & 0x0080 > 0 {0x40} else {0};
            self.acc = (temp & 0xFF) as u8;
            self.examine_status(self.acc);
        }


        //STA, STX, STY as one function
        fn store(&mut self, reg: u8, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, reg);
        }


        fn tax(&mut self) {
            self.ind_x = self.acc;
            self.examine_status(self.ind_x);
        }


        fn tay(&mut self) {
            self.ind_y = self.acc;
            self.examine_status(self.ind_y);
        }


        fn tsx(&mut self) {
            self.ind_x = self.stat;
            self.examine_status(self.ind_x);
        }


        fn txa(&mut self) {
            self.acc = self.ind_x;
            self.examine_status(self.acc);
        }


        fn txs(&mut self) {
            self.stat = self.ind_x;
        }


        fn tya(&mut self) {
            self.acc = self.ind_y;
            self.examine_status(self.acc);
        }


        //Checking zero and negative flags
        fn examine_status(&mut self, value: u8) {
            self.stat &= 0x7D;
            self.stat |= if value == 0 {0x02} else {0};
            self.stat |= if value & 0x80 == 0x80 {0x80} else {0}; 
        }

    }
}