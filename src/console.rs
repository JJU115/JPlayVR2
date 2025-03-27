pub mod console {

    use crate::cpu::cpu::Mos6502;

    pub struct Console<'a> {
        pub cpu: &'a mut Mos6502<'a>
    }


    impl Console<'_> {

        pub fn start_console(&mut self) {
            //Perform a reset on the cpu and ppu
            self.cpu.reset(true);

            //Execute a single CPU instruction and receive the cycle count
            //Run the PPU for 3 times that many cycles
            loop {
                let cpu_cycles = self.cpu.execute_instruction();
                if cpu_cycles == 0 {break;}
                self.cpu.ppu.generate_signal(cpu_cycles as u16);
            }
        }
    }
}