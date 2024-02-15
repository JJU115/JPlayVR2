//TODO
/*
    1. sort out signed branching in branch function --DONE
    2. check 'oops' cycles are properly calculated --TODO
    3. finish instruction match block -- array of the instruction enum? then match on that?
    4. verify cycle accuracy and proper number returned from execute_instruction function


*/


pub mod cpu {

    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};
    use crate::cartridge::cartridge;
    use crate::ppu::ppu::Ricoh2c02;

    #[derive(Debug, Copy, Clone)]
    enum AddressingMode {
        Implied,
        Accumulator,
        Immediate,
        ZeroPage,
        Absolute,
        Relative,
        Indirect,
        AbsoluteIndexX,
        AbsoluteIndexY,
        ZeroPageX,
        ZeroPageY,
        IndirectX,
        IndirectY
    }

    /*
        All MOS6502 instructions minus 'unofficial' opcodes, which have been replaced with NAI = Not An Instruction
     */
    enum Instruction {
        ADC(AddressingMode),AND(AddressingMode),ASL(AddressingMode),BCC(AddressingMode),BCS(AddressingMode),
        BEQ(AddressingMode),BIT(AddressingMode),BMI(AddressingMode),BNE(AddressingMode),BPL(AddressingMode),
        BRK(AddressingMode),BVC(AddressingMode),BVS(AddressingMode),CLC(AddressingMode),CLD(AddressingMode),
        CLI(AddressingMode),CLV(AddressingMode),CMP(AddressingMode),CPX(AddressingMode),CPY(AddressingMode),
        DEC(AddressingMode),DEX(AddressingMode),DEY(AddressingMode),EOR(AddressingMode),INC(AddressingMode),
        INX(AddressingMode),INY(AddressingMode),JMP(AddressingMode),JSR(AddressingMode),LDA(AddressingMode),
        LDX(AddressingMode),LDY(AddressingMode),LSR(AddressingMode),NOP(AddressingMode),ORA(AddressingMode),
        PHA(AddressingMode),PHP(AddressingMode),PLA(AddressingMode),PLP(AddressingMode),ROL(AddressingMode),
        ROR(AddressingMode),RTI(AddressingMode),RTS(AddressingMode),SBC(AddressingMode),SEC(AddressingMode),
        SED(AddressingMode),SEI(AddressingMode),STA(AddressingMode),STX(AddressingMode),STY(AddressingMode),
        TAX(AddressingMode),TAY(AddressingMode),TSX(AddressingMode),TXA(AddressingMode),TXS(AddressingMode),
        TYA(AddressingMode),NAI
    }

    pub struct Mos6502<'a> {
        pub cart: &'a cartridge::Cartridge,
        pub ppu: &'a Ricoh2c02<'a>,

        //Registers
        acc: u8,
        ind_x: u8,
        ind_y: u8,
        stat: u8,
        stck_pnt: u8,
        prg_cnt: u16,

        //Internal RAM
        cpu_ram: Vec<u8>,

        //'oops' cycles and the like
        extra_cycles: u8,

        instruction_array: Vec<Instruction>,
        instruction_cycles: Vec<u8>,

        log: File,
        log_comp: BufReader<File>,
        total_cycles: u64
    }


    impl Mos6502<'_> {

        pub fn new<'a>(cart: &'a cartridge::Cartridge, ppu: &'a Ricoh2c02<'a>) -> Mos6502<'a> {
            let instructions: Vec<Instruction> = vec![
                Instruction::BRK(AddressingMode::Immediate), Instruction::ORA(AddressingMode::IndirectX), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPage), Instruction::ORA(AddressingMode::ZeroPage), Instruction::ASL(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::PHP(AddressingMode::Implied), Instruction::ORA(AddressingMode::Immediate), Instruction::ASL(AddressingMode::Accumulator), Instruction::NAI, Instruction::NOP(AddressingMode::Absolute), Instruction::ORA(AddressingMode::Absolute), Instruction::ASL(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BPL(AddressingMode::Relative), Instruction::ORA(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPageX), Instruction::ORA(AddressingMode::ZeroPageX), Instruction::ASL(AddressingMode::ZeroPageX), Instruction::NAI, 
                Instruction::CLC(AddressingMode::Implied), Instruction::ORA(AddressingMode::AbsoluteIndexY), Instruction::NOP(AddressingMode::Implied), Instruction::NAI, Instruction::NOP(AddressingMode::AbsoluteIndexX), Instruction::ORA(AddressingMode::AbsoluteIndexX), Instruction::ASL(AddressingMode::AbsoluteIndexX), Instruction::NAI,
                Instruction::JSR(AddressingMode::Immediate), Instruction::AND(AddressingMode::IndirectX), Instruction::NAI, Instruction::NAI, Instruction::BIT(AddressingMode::ZeroPage), Instruction::AND(AddressingMode::ZeroPage), Instruction::ROL(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::PLP(AddressingMode::Implied), Instruction::AND(AddressingMode::Immediate), Instruction::ROL(AddressingMode::Accumulator), Instruction::NAI, Instruction::BIT(AddressingMode::Absolute), Instruction::AND(AddressingMode::Absolute), Instruction::ROL(AddressingMode::Absolute), Instruction::NAI,
                Instruction::BMI(AddressingMode::Relative), Instruction::AND(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPageX), Instruction::AND(AddressingMode::ZeroPageX), Instruction::ROL(AddressingMode::ZeroPageX), Instruction::NAI,
                Instruction::SEC(AddressingMode::Implied), Instruction::AND(AddressingMode::AbsoluteIndexY), Instruction::NOP(AddressingMode::Implied), Instruction::NAI, Instruction::NOP(AddressingMode::AbsoluteIndexX), Instruction::AND(AddressingMode::AbsoluteIndexX), Instruction::ROL(AddressingMode::AbsoluteIndexX), Instruction::NAI,
                Instruction::RTI(AddressingMode::Immediate), Instruction::EOR(AddressingMode::IndirectX), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPage), Instruction::EOR(AddressingMode::ZeroPage), Instruction::LSR(AddressingMode::ZeroPage), Instruction::NAI,
                Instruction::PHA(AddressingMode::Implied), Instruction::EOR(AddressingMode::Immediate), Instruction::LSR(AddressingMode::Accumulator), Instruction::NAI, Instruction::JMP(AddressingMode::Absolute), Instruction::EOR(AddressingMode::Absolute), Instruction::LSR(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BVC(AddressingMode::Relative), Instruction::EOR(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPageX), Instruction::EOR(AddressingMode::ZeroPageX), Instruction::LSR(AddressingMode::ZeroPageX), Instruction::NAI, 
                Instruction::CLI(AddressingMode::Implied), Instruction::EOR(AddressingMode::AbsoluteIndexY), Instruction::NOP(AddressingMode::Implied), Instruction::NAI, Instruction::NOP(AddressingMode::AbsoluteIndexX), Instruction::EOR(AddressingMode::AbsoluteIndexX), Instruction::LSR(AddressingMode::AbsoluteIndexX), Instruction::NAI,
                Instruction::RTS(AddressingMode::Immediate), Instruction::ADC(AddressingMode::IndirectX), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPage), Instruction::ADC(AddressingMode::ZeroPage), Instruction::ROR(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::PLA(AddressingMode::Implied), Instruction::ADC(AddressingMode::Immediate), Instruction::ROR(AddressingMode::Accumulator), Instruction::NAI, Instruction::JMP(AddressingMode::Indirect), Instruction::ADC(AddressingMode::Absolute), Instruction::ROR(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BVS(AddressingMode::Relative), Instruction::ADC(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPageX), Instruction::ADC(AddressingMode::ZeroPageX), Instruction::ROR(AddressingMode::ZeroPageX), Instruction::NAI, 
                Instruction::SEI(AddressingMode::Implied), Instruction::ADC(AddressingMode::AbsoluteIndexY), Instruction::NOP(AddressingMode::Implied), Instruction::NAI, Instruction::NOP(AddressingMode::AbsoluteIndexX), Instruction::ADC(AddressingMode::AbsoluteIndexX), Instruction::ROR(AddressingMode::AbsoluteIndexX), Instruction::NAI,
                Instruction::NOP(AddressingMode::Immediate), Instruction::STA(AddressingMode::IndirectX), Instruction::NOP(AddressingMode::Immediate), Instruction::NAI, Instruction::STY(AddressingMode::ZeroPage), Instruction::STA(AddressingMode::ZeroPage), Instruction::STX(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::DEY(AddressingMode::Implied), Instruction::NOP(AddressingMode::Immediate), Instruction::TXA(AddressingMode::Implied), Instruction::NAI, Instruction::STY(AddressingMode::Absolute), Instruction::STA(AddressingMode::Absolute), Instruction::STX(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BCC(AddressingMode::Relative), Instruction::STA(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::STY(AddressingMode::ZeroPageX), Instruction::STA(AddressingMode::ZeroPageX), Instruction::STX(AddressingMode::ZeroPageY), Instruction::NAI, 
                Instruction::TYA(AddressingMode::Implied), Instruction::STA(AddressingMode::AbsoluteIndexY), Instruction::TXS(AddressingMode::Implied), Instruction::NAI, Instruction::NAI, Instruction::STA(AddressingMode::AbsoluteIndexX), Instruction::NAI, Instruction::NAI,
                Instruction::LDY(AddressingMode::Immediate), Instruction::LDA(AddressingMode::IndirectX), Instruction::LDX(AddressingMode::Immediate), Instruction::NAI, Instruction::LDY(AddressingMode::ZeroPage), Instruction::LDA(AddressingMode::ZeroPage), Instruction::LDX(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::TAY(AddressingMode::Implied), Instruction::LDA(AddressingMode::Immediate), Instruction::TAX(AddressingMode::Implied), Instruction::NAI, Instruction::LDY(AddressingMode::Absolute), Instruction::LDA(AddressingMode::Absolute), Instruction::LDX(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BCS(AddressingMode::Relative), Instruction::LDA(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::LDY(AddressingMode::ZeroPageX), Instruction::LDA(AddressingMode::ZeroPageX), Instruction::LDX(AddressingMode::ZeroPageY), Instruction::NAI, 
                Instruction::CLV(AddressingMode::Implied), Instruction::LDA(AddressingMode::AbsoluteIndexY), Instruction::TSX(AddressingMode::Implied), Instruction::NAI, Instruction::LDY(AddressingMode::AbsoluteIndexX), Instruction::LDA(AddressingMode::AbsoluteIndexX), Instruction::LDX(AddressingMode::AbsoluteIndexY), Instruction::NAI,
                Instruction::CPY(AddressingMode::Immediate), Instruction::CMP(AddressingMode::IndirectX), Instruction::NOP(AddressingMode::Immediate), Instruction::NAI, Instruction::CPY(AddressingMode::ZeroPage), Instruction::CMP(AddressingMode::ZeroPage), Instruction::DEC(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::INY(AddressingMode::Implied), Instruction::CMP(AddressingMode::Immediate), Instruction::DEX(AddressingMode::Implied), Instruction::NAI, Instruction::CPY(AddressingMode::Absolute), Instruction::CMP(AddressingMode::Absolute), Instruction::DEC(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BNE(AddressingMode::Relative), Instruction::CMP(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPageX), Instruction::CMP(AddressingMode::ZeroPageX), Instruction::DEC(AddressingMode::ZeroPageX), Instruction::NAI, 
                Instruction::CLD(AddressingMode::Implied), Instruction::CMP(AddressingMode::AbsoluteIndexY), Instruction::NOP(AddressingMode::Implied), Instruction::NAI, Instruction::NOP(AddressingMode::AbsoluteIndexX), Instruction::CMP(AddressingMode::AbsoluteIndexX), Instruction::DEC(AddressingMode::AbsoluteIndexX), Instruction::NAI,
                Instruction::CPX(AddressingMode::Immediate), Instruction::SBC(AddressingMode::IndirectX), Instruction::NOP(AddressingMode::Immediate), Instruction::NAI, Instruction::CPX(AddressingMode::ZeroPage), Instruction::SBC(AddressingMode::ZeroPage), Instruction::INC(AddressingMode::ZeroPage), Instruction::NAI, 
                Instruction::INX(AddressingMode::Implied), Instruction::SBC(AddressingMode::Immediate), Instruction::NOP(AddressingMode::Implied), Instruction::SBC(AddressingMode::Immediate), Instruction::CPX(AddressingMode::Absolute), Instruction::SBC(AddressingMode::Absolute), Instruction::INC(AddressingMode::Absolute), Instruction::NAI, 
                Instruction::BEQ(AddressingMode::Relative), Instruction::SBC(AddressingMode::IndirectY), Instruction::NAI, Instruction::NAI, Instruction::NOP(AddressingMode::ZeroPageX), Instruction::SBC(AddressingMode::ZeroPageX), Instruction::INC(AddressingMode::ZeroPageX), Instruction::NAI, 
                Instruction::SED(AddressingMode::Implied), Instruction::SBC(AddressingMode::AbsoluteIndexY), Instruction::NOP(AddressingMode::Implied), Instruction::NAI, Instruction::NOP(AddressingMode::AbsoluteIndexX), Instruction::SBC(AddressingMode::AbsoluteIndexX), Instruction::INC(AddressingMode::AbsoluteIndexX), Instruction::NAI,
            ];

            let cycles: Vec<u8> = vec![
                7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
                2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
                2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
                2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
                2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
                2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
                2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7
            ];

            Mos6502 { 
                cart: cart, 
                ppu: ppu,
                acc: 0, 
                ind_x: 0, 
                ind_y: 0, 
                stat: 0x24, 
                stck_pnt: 0xFD, 
                prg_cnt: 0xFFFC, 
                cpu_ram: vec![0; 2048], 
                extra_cycles: 0, 
                instruction_array: instructions,
                instruction_cycles: cycles,
                log: File::create("CPU_LOG.txt").unwrap(),
                log_comp: BufReader::new(File::open("nestest.log").unwrap()),
                total_cycles: 7
            }
        }


        pub fn reset(&mut self, power_up: bool) {
            self.prg_cnt = 0xC000;//((self.cart.cpu_read(0xFFFD) as u16) << 8) | (self.cart.cpu_read(0xFFFC) as u16);
            if power_up { return; }
            self.stat = self.stat | 0x04;
            self.stck_pnt = self.stck_pnt - 3;
        }


        fn fetch_from_address(&mut self, addr: u16) -> u8 {
            match addr {
                //$0000–$1FFF Internal ram, mirrors every $0800 addresses
                0x0000..=0x1FFF => self.cpu_ram[(addr & 0x07FF) as usize],

                //$2000–$2007 PPU registers, $2008–$3FFF mirrors $2000–$2007 every 8 bytes
                0x2000..=0x3FFF => 0,
              
                //$4000–$4017 NES APU registers, anything other than $4015 produces open bus behavior
                0x4000..=0x4013 => 0,

                //$4020–$FFFF Cartridge space: PRG ROM, PRG RAM, and mapper registers 
                0x4020..=0xFFFF => self.cart.cpu_read(addr),

                _ => 0
            }
        }


        fn writeback(&mut self, addr: u16, value: u8) {
            match addr {
                0x0000..=0x1FFF => {
                    self.cpu_ram[(addr & 0x07FF) as usize] = value;
                },
                0x2000..=0x3FFF => {
                    //PPU registers
                },
                0x4000..=0x4013 | 0x4015..=0x4017 => {
                    //APU Registers
                },
                0x4014 => {
                    //OAM DMA
                },              
                0x4020..=0xFFFF => {
                    self.cart.cpu_write(addr, value);
                },
                _ => ()
            }
        }


        pub fn execute_instruction(&mut self) -> u8 {
            //Fetch the opcode and the next byte
            let opcode = self.cart.cpu_read(self.prg_cnt);


            writeln!(self.log, "{:04X} {:02X}\tA:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}\tCYC:{}",
            self.prg_cnt, opcode, self.acc, self.ind_x, self.ind_y, self.stat, self.stck_pnt, self.total_cycles).unwrap();
            let mut buf = String::new();
            self.log_comp.read_line(&mut buf).unwrap();
            let correctPC = u16::from_str_radix(buf.get(0..4).unwrap(), 16).unwrap();
            if correctPC != self.prg_cnt {
                println!("Failed!");
                return 0;
            }

            self.extra_cycles = 0;
            self.prg_cnt += 1; //CEFD

            match self.instruction_array[opcode as usize] {
                Instruction::ADC(mode) => self.adc(&mode),
                Instruction::AND(mode) => self.and(&mode),
                Instruction::ASL(mode) => self.asl(&mode),
                Instruction::BCC(_mode) | Instruction::BCS(_mode) | 
                Instruction::BVC(_mode) | Instruction::BVS(_mode) | 
                Instruction::BMI(_mode) | Instruction::BNE(_mode) |
                Instruction::BPL(_mode) | 
                Instruction::BEQ(_mode) => self.branch(opcode >> 6, (opcode >> 5) & 1),
                Instruction::BIT(mode) => self.bit(&mode),       
                Instruction::BRK(_mode) => self.brk(),
                Instruction::CLC(_mode) => self.clear(0xFE),
                Instruction::CLD(_mode) => self.clear(0xF7),
                Instruction::CLI(_mode) => self.clear(0xFB),
                Instruction::CLV(_mode) => self.clear(0xBF),
                Instruction::CMP(mode) => self.cmp(&mode),
                Instruction::CPX(mode) => self.cpx(&mode),
                Instruction::CPY(mode) => self.cpy(&mode),
                Instruction::DEC(mode) => self.dec(&mode),
                Instruction::DEX(_mode) => self.dex(),
                Instruction::DEY(_mode) => self.dey(),
                Instruction::EOR(mode) => self.eor(&mode),
                Instruction::INC(mode) => self.inc(&mode),
                Instruction::INX(_mode) => self.inx(),
                Instruction::INY(_mode) => self.iny(),
                Instruction::JMP(mode) => self.jmp(&mode),
                Instruction::JSR(_mode) => self.jsr(),
                Instruction::LDA(mode) => self.lda(&mode),
                Instruction::LDX(mode) => self.ldx(&mode),
                Instruction::LDY(mode) => self.ldy(&mode),
                Instruction::LSR(mode) => self.lsr(&mode),
                Instruction::NOP(_mode) => (),
                Instruction::ORA(mode) => self.ora(&mode),
                Instruction::PHA(_mode) => self.push(self.acc),
                Instruction::PHP(_mode) => self.push(self.stat),
                Instruction::PLA(_mode) => self.pla(),
                Instruction::PLP(_mode) => self.plp(),
                Instruction::ROL(mode) => self.rol(&mode),
                Instruction::ROR(mode) => self.ror(&mode),
                Instruction::RTI(_mode) => self.rti(),
                Instruction::RTS(_mode) => self.rts(),
                Instruction::SBC(mode) => self.sbc(&mode),
                Instruction::SEC(_mode) => self.set(0x01),
                Instruction::SED(_mode) => self.set(0x08),
                Instruction::SEI(_mode) => self.set(0x04),
                Instruction::STA(mode) => self.store(self.acc, &mode),
                Instruction::STX(mode) => self.store(self.ind_x, &mode),
                Instruction::STY(mode) => self.store(self.ind_y, &mode),
                Instruction::TAX(_mode) => self.tax(),
                Instruction::TAY(_mode) => self.tay(),
                Instruction::TSX(_mode) => self.tsx(),
                Instruction::TXA(_mode) => self.txa(),
                Instruction::TXS(_mode) => self.txs(),
                Instruction::TYA(_mode) => self.tya(),
                Instruction::NAI => println!("Unofficial/Unassigned opcode!")
            };
            self.total_cycles += (self.instruction_cycles[opcode as usize] + self.extra_cycles) as u64;
            self.instruction_cycles[opcode as usize] + self.extra_cycles
        }


        //Returns a tuple where 1st element is data to operate on, 2nd is the writeback address
        fn fetch_instruction_data(&mut self, mode: &AddressingMode) -> (u8, u16) {
            let data: u8 = self.cart.cpu_read(self.prg_cnt);
            if let AddressingMode::Accumulator = mode {self.prg_cnt -= 1;} //If addressing mode is accumulator, we need to cancel out the increment on the next line
            self.prg_cnt += 1;
            match mode {
                AddressingMode::Accumulator => (self.acc, 0),
                AddressingMode::Immediate => (data, 0),
                AddressingMode::ZeroPage => (self.cpu_ram[data as usize], data as u16),
                AddressingMode::Absolute => {
                    let addr = (self.cart.cpu_read(self.prg_cnt) as u16) << 8 | data as u16;
                    self.prg_cnt += 1;
                    (self.fetch_from_address(addr), addr) 
                },
                AddressingMode::AbsoluteIndexX => {
                    let addr: u16 = (self.cart.cpu_read(self.prg_cnt) as u16) << 8 | (data + self.ind_x) as u16;
                    self.prg_cnt += 1;
                    //'oops' cycle, but only for read instructions that cross a page
                    if (data + self.ind_x) as u16 > 255 { self.extra_cycles += 1; }
                    (self.fetch_from_address(addr), addr)
                },
                AddressingMode::AbsoluteIndexY => {
                    let addr: u16 = (self.cart.cpu_read(self.prg_cnt) as u16) << 8 | (data + self.ind_y) as u16;
                    self.prg_cnt += 1;
                    //'oops' cycle, but only for read instructions that cross a page
                    if (data + self.ind_y) as u16 > 255 { self.extra_cycles += 1; }
                    (self.fetch_from_address(addr), addr)
                },
                AddressingMode::ZeroPageX => (self.cpu_ram[(data + self.ind_x) as usize], data as u16),
                AddressingMode::ZeroPageY => (self.cpu_ram[(data + self.ind_y) as usize], data as u16),
                AddressingMode::IndirectX => {
                    let addr = (self.cpu_ram[data.wrapping_add(self.ind_x).wrapping_add(1) as usize] as u16) << 8 | self.cpu_ram[data.wrapping_add(self.ind_x) as usize] as u16;
                    (self.fetch_from_address(addr), addr) 
                },
                AddressingMode::IndirectY => {
                    let high: u16 = self.cpu_ram[data.wrapping_add(1) as usize] as u16;
                    let addr = high << 8 | self.cpu_ram[data as usize] as u16;
                    if high + self.ind_y as u16 > 255 { self.extra_cycles += 1; }
                    (self.fetch_from_address(addr.wrapping_add(self.ind_y as u16)), addr)
                },
                _ => (0,0)
            }
        }



        fn adc(&mut self, mode: &AddressingMode) {
            let data: (u8, u16) = self.fetch_instruction_data(mode);
            let sum: u8 = self.acc.wrapping_add(data.0).wrapping_add(self.stat & 1);
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
            self.stat = (self.stat & 0xFE) | ((data.0 & 0x80) >> 7);
            data.0 = data.0 << 1;
            if let AddressingMode::Accumulator = mode {
                self.acc = data.0;
            } else {
                self.writeback(data.1, data.0)
            }
            self.examine_status(data.0);
        }


        fn bit(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.stat = (self.stat & 0x3D) | (data.0 & 0xC0); 
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

            let op: i8 = self.cart.cpu_read(self.prg_cnt) as i8;
            self.prg_cnt += 1;

            if comp == value {
                
                if op < 0 {
                    //negative, sub from pc
                    if ((self.prg_cnt - op.abs() as u16) & 0x0F00) != (self.prg_cnt & 0x0F00) {
                        self.extra_cycles += 1;
                    }
                    self.prg_cnt -= op.abs() as u16;
                } else {
                    //positive, add to pc
                    if ((self.prg_cnt + op as u16) & 0x0F00) != (self.prg_cnt & 0x0F00) {
                        self.extra_cycles += 1;
                    }
                    self.prg_cnt += op as u16;
                }
                self.extra_cycles += 1;

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
            self.prg_cnt += 1;
            self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] = ((self.prg_cnt & 0xFF00) >> 8) as u8;
            self.cpu_ram[(0x00FF + self.stck_pnt as u16) as usize] = (self.prg_cnt & 0xFF) as u8;
            self.cpu_ram[(0x00FE + self.stck_pnt as u16) as usize] = self.stat | 0x30;
            self.stck_pnt -= 3;
            self.prg_cnt = self.cart.cpu_read(0xFFFE) as u16 | (self.cart.cpu_read(0xFFFF) as u16) << 8;
            self.stat |= 0x14;
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
            if (self.acc.wrapping_sub(data.0)) & 0x80 == 0x80 { self.stat |= 0x80 }
        }

        
        fn cpx(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.ind_x >= data.0 { self.stat |= 0x01; }
            if self.ind_x == data.0 { self.stat |= 0x02; }
            if (self.ind_x.wrapping_sub(data.0)) & 0x80 == 0x80 { self.stat |= 0x80 }
        }


        fn cpy(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.stat &= 0x7C;
            if self.ind_y >= data.0 { self.stat |= 0x01; }
            if self.ind_y == data.0 { self.stat |= 0x02; }
            if (self.ind_y.wrapping_sub(data.0)) & 0x80 == 0x80 { self.stat |= 0x80 }
        }


        fn dec(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, data.0.wrapping_sub(1));
            self.examine_status(data.0.wrapping_sub(1));
        }

        
        fn dex(&mut self) {
            self.ind_x = self.ind_x.wrapping_sub(1);
            self.examine_status(self.ind_x);
        }


        fn dey(&mut self) {
            self.ind_y = self.ind_y.wrapping_sub(1);
            self.examine_status(self.ind_y);
        }


        fn eor(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.acc ^= data.0;
            self.examine_status(self.acc);
        }


        fn inc(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            self.writeback(data.1, data.0.wrapping_add(1));
            self.examine_status(data.0.wrapping_add(1));
        }

        fn inx(&mut self) {
            self.ind_x = self.ind_x.wrapping_add(1);
            self.examine_status(self.ind_x);
        }

        fn iny(&mut self) {
            self.ind_y = self.ind_y.wrapping_add(1);
            self.examine_status(self.ind_y);
        }


        //JMP Instruction - Absolute or indirect addressing modes
        fn jmp(&mut self, mode: &AddressingMode) {
            if let AddressingMode::Absolute = mode {
                self.prg_cnt = (self.cart.cpu_read(self.prg_cnt + 1) as u16) << 8 | self.cart.cpu_read(self.prg_cnt) as u16;
            } else {
                let target = (self.cart.cpu_read(self.prg_cnt + 1) as u16) << 8 | 
                    self.cart.cpu_read(self.prg_cnt) as u16;
                
                self.prg_cnt = (self.cart.cpu_read(target + 1) as u16) << 8 | self.cart.cpu_read(target) as u16;
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
            self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize] = ((1 + self.prg_cnt & 0xFF00) >> 8) as u8;
            self.cpu_ram[(0x00FF + self.stck_pnt as u16) as usize] = (1 + self.prg_cnt & 0xFF) as u8;
            self.stck_pnt -= 2;

            self.prg_cnt = (self.cart.cpu_read(self.prg_cnt + 1) as u16) << 8 | self.cart.cpu_read(self.prg_cnt) as u16;
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
            if let AddressingMode::Accumulator = mode {
                self.acc = temp;
            } else {
                self.writeback(data.1, temp);
            }
            self.stat &= 0xFE;
            self.stat |= data.0 & 0x01;
            self.examine_status(temp);
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


        fn pla(&mut self) {
            self.stck_pnt += 1;
            self.acc = self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize];         
            self.examine_status(self.acc);
        }

        fn plp(&mut self) {
            self.stck_pnt += 1;
            self.stat = self.cpu_ram[(0x0100 + self.stck_pnt as u16) as usize];       
        }


        fn rol(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            let temp = (data.0 << 1) | (self.stat & 0x01);
            if let AddressingMode::Accumulator = mode {
                self.acc = temp;
            } else {
                self.writeback(data.1, temp);
            }
            self.stat &= 0xFE;
            self.stat |= (data.0 & 0x80) >> 7;
            self.examine_status(temp);
        }

        fn ror(&mut self, mode: &AddressingMode) {
            let data = self.fetch_instruction_data(mode);
            let temp = (data.0 >> 1) | ((self.stat & 0x01) << 7);
            if let AddressingMode::Accumulator = mode {
                self.acc = temp;
            } else {
                self.writeback(data.1, temp);
            }
            self.stat &= 0xFE;
            self.stat |= data.0 & 0x01;
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
            self.stat = self.cpu_ram[(0x0101 + self.stck_pnt as u16) as usize];
            self.prg_cnt = self.cpu_ram[(0x0102 + self.stck_pnt as u16) as usize] as u16 | (self.cpu_ram[(0x0103 + self.stck_pnt as u16) as usize] as u16) << 8;
            self.stck_pnt += 3;
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
            self.prg_cnt = self.cpu_ram[(0x0101 + self.stck_pnt as u16) as usize] as u16 | (self.cpu_ram[(0x0102 + self.stck_pnt as u16) as usize] as u16) << 8;
            self.stck_pnt += 2;
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

        //OR the mask with the status register
        fn set(&mut self, mask: u8) {
            self.stat |= mask;
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
            self.ind_x = self.stck_pnt;
            self.examine_status(self.ind_x);
        }


        fn txa(&mut self) {
            self.acc = self.ind_x;
            self.examine_status(self.acc);
        }


        fn txs(&mut self) {
            self.stck_pnt = self.ind_x;
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