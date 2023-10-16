pub mod console {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    use crate::cpu::cpu::Mos6502;
    use crate::cartridge::cartridge::Cartridge;

    pub struct Console<'a> {
        pub cartridge: &'a Cartridge,
        pub cpu: &'a mut Mos6502<'a>
    }


    impl Console<'_> {
        pub fn load_cartridge(&self, file_name: &String) -> Result<String, String> {
            //Open file in read-only mode,
            let mut file = File::open(file_name).map_err(|_| String::from("Could not open file"))?;

            //Read the header
            let mut ines_header: [u8; 16] = [0; 16];
            file.read_exact(&mut ines_header).map_err(|_| String::from("Could not read header"));

            if ines_header[0] != 0x4E || ines_header[1] != 0x45 || ines_header[2] != 0x53 || ines_header[3] != 0x1A {
                return Err(String::from("Bad iNES header"));
            }

            let mut cpu_start: u64 = 16;
            let mut ppu_start: u64 = 16 + ines_header[4] as u64 * 16384;

            //Trainer present in header, ignore it
            if ines_header[6] & 0x04 != 0 {
                cpu_start = 544;
                ppu_start += 528;
            }

            //Usage of byte 7 only needed for mapper 66
            self.cartridge.mapper_num = (ines_header[6] & 0xF0) >> 4 | ines_header[7] & 0xF0;

            file.seek(SeekFrom::Start(cpu_start)).map_err(|_| String::from("Seek failed"))?;

            Result::Ok(String::from("File contents copied"))
        }


        pub fn start_console(&mut self) {
            //Perform a reset on the cpu and ppu
            self.cpu.reset();


            //Execute a single CPU instruction and receive the cycle count
            //Run the PPU for 3 times that many cycles
            loop {
                let cpu_cycles = self.cpu.execute_instruction();
            }
        }
    }
}