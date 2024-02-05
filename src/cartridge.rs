pub mod cartridge {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use crate::nrom::nrom::Nrom;

    pub trait Mapper {
        fn ppu_read(&self) -> usize;
        fn cpu_read(&self, addr: u16) -> usize;
        fn cpu_write(&self, addr: u16, value: u8);
        fn ppu_write(&self);
    }

    pub struct Cartridge {
        pub mapper: Box<dyn Mapper>,
        pub prg_rom: Vec<u8>,
        pub chr_rom: Vec<u8>,
    }


    impl Cartridge {

        pub fn load_rom(file_name: &String) -> Result<Cartridge, String> {
            //Open file in read-only mode,
            let mut file = File::open(file_name).map_err(|_| String::from("Could not open file"))?;

            //Read the header
            let mut ines_header: [u8; 16] = [0; 16];
            file.read_exact(&mut ines_header).map_err(|_| String::from("Could not read header"))?;

            if ines_header[0] != 0x4E || ines_header[1] != 0x45 || ines_header[2] != 0x53 || ines_header[3] != 0x1A {
                return Err(String::from("Bad iNES header"));
            }

            let mut cart = Cartridge {
                mapper: match (ines_header[6] & 0xF0) >> 4 | ines_header[7] & 0xF0 {
                    0 => Box::new(Nrom::new(ines_header[4] == 1)),
                    _ => Box::new(Nrom::new(ines_header[4] == 1))
                },
                prg_rom: vec![0],
                chr_rom: vec![0]
            };

            //Total size is in bytes
            cart.prg_rom.resize((ines_header[4] as u64 * 16384) as usize, 0);
            cart.chr_rom.resize((ines_header[5] as u64 * 8192) as usize, 0);

            //Start of the PRG data, taking the trainer into account if present
            let cpu_start: u64 = 16 + if ines_header[6] & 0x04 != 0 {512} else {0}; 

            file.seek(SeekFrom::Start(cpu_start)).map_err(|_| String::from("Seek to PRG failed"))?;
            file.read_exact(&mut cart.prg_rom).map_err(|_| String::from("Read PRG failed"))?;
            file.seek(SeekFrom::Current(0)).map_err(|_| String::from("Seek to CHR failed"))?;
            file.read_exact(&mut cart.chr_rom).map_err(|_| String::from("Read CHR failed"))?;
            
            Result::Ok(cart)
        }

        /*
            Since all instructions require fetching two bytes, can just call this function once 
            from cpu to get those two bytes rather than having two separate calls to cpu_read
        */
        pub fn cpu_read(&self, addr: u16) -> u8 {
            if addr > 0x7FFF {
                self.prg_rom[self.mapper.cpu_read(addr)]
            } else {
                0
            }
            
        }

        pub fn cpu_write(&self, addr: u16, value: u8) {

        }

        pub fn ppu_read(&self) {

        }
    }
    




}