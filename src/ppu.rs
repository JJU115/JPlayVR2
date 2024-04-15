pub mod ppu {
    use crate::cartridge::cartridge::Cartridge;

    pub struct Ricoh2c02<'a> {
        pub cart: &'a Cartridge,

        vram: Vec<u8>,
        primary_oam: Vec<u8>,
        secondary_oam: Vec<u8>,
        ppudata_buffer: u8,

        current_scanline: i16,
        scanline_cycle: u16,
        is_odd_cycle: bool,

        ppu_ctrl: u8,
        ppu_status: u8,
        ppu_mask: u8,
        oam_addr: u8,
        oam_data: u8,
        //ppu_scroll: u8,
        ppu_addr: u8,
        //ppu_data: u8,

        vram_addr: u16, //v
        temp_vram_addr: u16, //t
        fine_x_scroll: u8, //x
        write_toggle: bool, //w

        nmi_occurred: bool,
        nmi_output: bool,
        supress_nmi: bool,
        gen_nmi: bool,
    }



    impl Ricoh2c02<'_> {

        pub fn new(c: &Cartridge) -> Ricoh2c02 {
            Ricoh2c02 { 
                cart: c,

                vram: vec![0; 2048],
                primary_oam: vec![0; 256],
                secondary_oam: vec![0; 64],
                ppudata_buffer: 0,

                current_scanline: 261,
                scanline_cycle: 0,
                is_odd_cycle: false,

                ppu_ctrl: 0,
                ppu_status: 0xA0,
                ppu_mask: 0,
                oam_addr: 0,
                oam_data: 0,
                //ppu_scroll: 0,
                ppu_addr: 0,
                //ppu_data: 0,

                vram_addr: 0,
                temp_vram_addr: 0,
                fine_x_scroll: 0,
                write_toggle: false,

            /*
                Start of vertical blanking: Set NMI_occurred in PPU to true.
                End of vertical blanking, sometime in pre-render scanline: Set NMI_occurred to false.
                Read PPUSTATUS: Return old status of NMI_occurred in bit 7, then set NMI_occurred to false.
                Write to PPUCTRL: Set NMI_output to bit 7. 
            */
                nmi_occurred: false,
                nmi_output: false,
                supress_nmi: false,
                gen_nmi: false,
             }
        }

        pub fn reset(&mut self) {
            self.ppu_ctrl = 0;
            self.ppu_mask = 0;
            self.write_toggle = false;
            self.ppudata_buffer = 0;
            self.is_odd_cycle = false;
        }


        pub fn register_read(&mut self, register_index: u8) -> u8 {
            match register_index {
                0 | 1 | 3 | 5 | 6 => 0, //Should return open bus
                2 => { //PPUSTATUS
                    let reg_value = if self.nmi_occurred {self.ppu_status | 0x80} else {self.ppu_status};
                    self.supress_nmi = (self.current_scanline == 242) && (self.scanline_cycle < 3);
                    self.nmi_occurred = false;
                    self.ppu_status &= 0x7F;
                    self.write_toggle = false;
                    reg_value
                },
                4 => { //OAMDATA
                    //Pull value from OAM at address in OAMADDR
                    self.primary_oam[self.oam_addr as usize]
                },
                7 => { //PPUDATA when v is before palettes
                    //Read from vram from the address specified in PPUADDR, then increment PPUADDR
                    //Utilize the internal read buffer
                    let buffer_val: u8 = self.ppudata_buffer;
                    self.ppudata_buffer = self.vram[self.vram_addr as usize];
                    self.vram_addr += if self.ppu_ctrl & 0x04 == 0 {1} else {32};
                    buffer_val
                },
                _ => 0
            }
        }


        //Writes to the PPUCTRL, PPUMASK, PPUADDR, PPUSCROLL are ignored if earlier than ~29658 CPU clocks after reset
        pub fn register_write(&mut self, register_index: u8, value: u8, cycles_passed: u16) {
            match register_index {
                //If currently in vertical blank and PPUSTATUS has vblank flag is set, 
                //changing bit 7 here from 0 to 1 generates an NMI
                0 if cycles_passed > 29658 => {
                    self.gen_nmi = self.ppu_ctrl & 0x80 == 0 && 
                                        value & 0x80 != 0 &&
                                        self.ppu_status & 0x80 != 0 &&
                                        self.current_scanline > 240;
                    self.nmi_output = value & 0x80 > 0;
                    self.temp_vram_addr &= 0x73FF;
                    self.temp_vram_addr |= (value as u16 & 0x03) << 10;
                    self.ppu_ctrl = value;                
                }
                //PPUMASK, rendering of sprites/backgrounds enabled and disabled here
                1 if cycles_passed > 29658 => {self.ppu_mask = value;}
                //OAMDADDR, set to 0 during each of ticks 257–320 of the pre-render and visible scanlines
                3 => {self.oam_addr = value;}
                //OAMDATA, best to ignore writes during the rendering period
                4 if self.current_scanline > 239 => {
                    self.oam_data = value;
                    self.primary_oam[self.oam_addr as usize] = value;
                    self.oam_addr += 1;
                }
                //PPUSCROLL - write toggle is false
                5 if cycles_passed > 29658 && !self.write_toggle => {
                    self.temp_vram_addr &= 0xFFE0;
                    self.temp_vram_addr |= value as u16 & 0x1F;
                    self.fine_x_scroll = value & 0x07;
                    self.write_toggle = true;
                },
                //PPUSCROLL - second write
                5 if self.write_toggle => {
                    self.temp_vram_addr &= 0x0C1F;
                    self.temp_vram_addr |= (value as u16 & 0x07) << 12; 
                    self.temp_vram_addr |= (value as u16 & 0xF1) << 2;
                    self.write_toggle = false;
                }
                //PPUADDR - First write, toggle is false
                6 if cycles_passed > 29658 && !self.write_toggle => {
                    self.temp_vram_addr &= 0x00FF;
                    self.temp_vram_addr |= (value as u16 & 0x3F) << 8;
                    self.write_toggle = true; 
                },
                //PPUADDR - Second write, toggle is true
                6 if self.write_toggle => {
                    self.temp_vram_addr &= 0xFF00;
                    self.temp_vram_addr |= value as u16;
                    self.vram_addr = self.temp_vram_addr;
                    self.write_toggle = false;
                },
                //PPUDATA
                7 => {
                    self.vram[self.ppu_addr as usize] = value;
                    self.vram_addr += if self.ppu_ctrl & 0x04 == 0 {1} else {32};
                }
                _ => ()
            }
        }

        //Based on the internal current cycle, perform one of several actions
        pub fn generate_signal(&mut self, cycles_to_run: u8) {
            //PPU generates 262 scanlines per frame
            //Each scanline takes 341 PPU cycles, one pixel produced per cycle
            //Cycles 0-340 -- Pre-render scanline, this is one cycle shorter on odd frames
            //240 Visible scanlines 0-239
            //Post render scanline 240
            //20 VBlank scanlines 241-261
            //At the start of vertical blanking, set nmi_occurred to true
            //After vertical blanking, sometime during pre-render, set nmi_occurred to false

            match self.current_scanline {
                261 => self.pre_render_scanline(),
                0..=239 => self.visible_scanline(cycles_to_run),
                240 => (),
                241..=260 => self.vertical_blanking(),
                _ => println!("Error: {} is not a valid scanline", self.current_scanline)
            }

            


            self.scanline_cycle += cycles_to_run as u16;
                       
            if self.scanline_cycle > 340 {
                self.current_scanline += 1;
                if self.current_scanline > 261 {
                    self.current_scanline = 0;
                }
                self.scanline_cycle -= 341;
            } 
        }

        
        //Only for pre-filling the registers for the first two tiles of first visible scanline
        //Although same memory accesses as regular scanline is still technically made
        fn pre_render_scanline(&mut self) {

        }



        fn visible_scanline(&mut self, cycles_to_run: u8) {
            let tiles_loaded = (1..=cycles_to_run).fold(0,|acc, curr| 
                if (self.scanline_cycle + curr as u16) % 9 == 0 {acc+1} else {acc});

            match self.scanline_cycle {
            //Visible portion of scanline, cycles 1-256. 
            //Cycle 0 is idle, shift registers are reloaded on cycle 257 but we ignore this
                0..=256 => {
                    //For tiles_loaded times, render a tile
                },
            //Tile data for next scanline's sprites. 8 sprites, 4 fetches each, 2 cycles per fetch = 64 
                257..=320 => {
                    //If scanline_cycle + cycles_to_run exceeds 320, perform the fetches, ignore otherwise 
                },
            //First 2 tiles of next scanline fetched here
                321..=336 => {
                    //Same as cycles 257..320
                },
            //2 unused nametable bytes fetched, these plus first nametable fetch of next scanline are used
            //by MMC5 mapper to clock a scanline counter
                337..=340 => {
                    //If not imnplementing MMC5 then do nothing, we'll see...
                },
                _ => ()
            }
        }


        //20 scanlines worth of idling, aside from some flag setting
        fn vertical_blanking(&mut self) {

        }
    }

}