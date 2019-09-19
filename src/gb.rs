use std::fs::File;
use std::io::prelude::*;

pub struct GB {
    wram: [u8; 8192],
    vram: [u8; 8192],
    pub cpu: CPU,
    cart: Cartridge,
    regs: [u8; 0x80],
    oam: [u8; 0xA0],
    ei: u8,
    stack: [u8; 0x180],
}

impl GB {
    pub fn new() -> GB {
        return GB {
            wram: [0; 8192],
            vram: [0; 8192],
            cpu: CPU::new(),
            cart: Cartridge::new(),
            regs: [0; 0x80],
            oam: [0; 0xA0],
            ei: 0,
            stack: [0; 0x180],
        }
    }
}

pub struct CPU {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,

}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0,
            pc: 0,
        };
        return cpu;
    }
    pub fn get_a(&mut self) -> u8 {return ((self.af >> 8) & 0xFF) as u8 }
    pub fn get_b(&mut self) -> u8 {return ((self.bc >> 8) & 0xFF) as u8 }
    pub fn get_c(&mut self) -> u8 {return ((self.bc) & 0xFF) as u8 }
    pub fn get_d(&mut self) -> u8 {return ((self.de >> 8) & 0xFF) as u8 }
    pub fn get_e(&mut self) -> u8 {return ((self.de) & 0xFF) as u8 }
    pub fn get_h(&mut self) -> u8 {return ((self.hl >> 8) & 0xFF) as u8 }
    pub fn get_l(&mut self) -> u8 {return ((self.hl) & 0xFF) as u8 }
    pub fn get_hl(&mut self) -> u16 { return self.hl }

    pub fn set_a(&mut self, val: u8) { self.af = (self.af & 0x00FF) | ((val as u16) << 8) }
    pub fn set_b(&mut self, val: u8) { self.bc = (self.bc & 0x00FF) | ((val as u16) << 8) }
    pub fn set_c(&mut self, val: u8) { self.bc = (self.bc & 0xFF00) | (val as u16) }
    pub fn set_d(&mut self, val: u8) { self.de = (self.de & 0x00FF) | ((val as u16) << 8) }
    pub fn set_e(&mut self, val: u8) { self.de = (self.de & 0xFF00) | (val as u16) }
    pub fn set_h(&mut self, val: u8) { self.hl = (self.hl & 0x00FF) | ((val as u16) << 8) }
    pub fn set_l(&mut self, val: u8) { self.hl = (self.hl & 0xFF00) | (val as u16) }
    pub fn set_hl(&mut self, val: u16) { self.hl = val }

    pub fn get_z(&mut self) -> u8 {return ((self.af >> 7) & 0x1) as u8 }
    pub fn get_n(&mut self) -> u8 {return ((self.af >> 6) & 0x1) as u8 }
    pub fn get_hc(&mut self) -> u8 {return ((self.af >> 5) & 0x1) as u8 }
    pub fn get_cy(&mut self) -> u8 {return ((self.af >> 4) & 0x1) as u8 }

    pub fn set_z(&mut self, val: u8) {
        if val == 1 {self.af = self.af | (val as u16) << 7;}
        else {self.af = self.af & !(1 << 7)}
    }
    pub fn set_n(&mut self, val: u8) {
        if val == 1 {self.af = self.af | (val as u16) << 6;}
        else {self.af = self.af & !(1 << 6)}
    }
    pub fn set_hc(&mut self, val: u8) {
        if val == 1 { self.af = self.af | (val as u16) << 5; }
        else { self.af = self.af & !(1 << 5) }
    }
    pub fn set_cy(&mut self, val: u8) {
        if val == 1 {self.af = self.af | (1 << 4);}
        else {self.af = self.af & !(1 << 4);}
    }
}

pub struct Cartridge {
    rom: [u8; 0x8000],
    ram: [u8; 0x2000],
}

impl Cartridge {
    pub fn new() -> Cartridge {
        let mut cartridge = Cartridge {
            rom: [0; 0x8000],
            ram: [0; 0x2000],
        };
        return cartridge;
    }
}

impl Cartridge {
    pub fn load_application(&mut self, filename: &str) -> bool {
        let mut file = File::open(filename).expect("File error");
        let fsize = file.metadata().unwrap().len();
        println!("{:X}", fsize);

        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("couldn't read file");
        drop(file);

        if (0x8000) >= fsize {
            for i in 0..fsize
            {
                self.rom[i as usize] = buffer[i as usize];
            }
        }
        else {
            panic!("ROM too big for memory");
        }
        return true;
    }
}

impl GB {
    fn mem_write(&mut self, addr: u16, val: u8) {
        if addr <= 0x3FFF {        // ROM Bank
            self.cart.rom[addr as usize] = val;
        } else if addr >= 0x4000 && addr <= 0x7FFF { // ROM Bank 1-n
            self.cart.rom[addr as usize] = val;
        } else if addr >= 0x8000 && addr <= 0x9FFF { // VRAM
            self.vram[(addr - 0x8000) as usize] = val;
        } else if addr >= 0xA000 && addr <= 0xBFFF { // Cart RAM
            self.cart.ram[(addr - 0xA000) as usize] = val;
        } else if addr >= 0xC000 && addr <= 0xDFFF { // Low RAM
            self.wram[(addr - 0xC000) as usize] = val;
        } else if addr >= 0xE000 && addr <= 0xFDFF { // Low RAM Duplicate
            self.wram[(addr - 0xE000) as usize] = val;
        } else if addr >= 0xFE00 && addr <= 0xFE9F { // OAM RAM
            self.oam[(addr - 0xFE00) as usize] = val;
        } else if addr >= 0xFF00 && addr <= 0xFF7F { // I/O Registers
            self.regs[(addr - 0xFF00) as usize] = val;
        } else if addr >= 0xFF80 && addr <= 0xFFFE { // High RAM (Stack)
            self.stack[(addr - 0xFF80) as usize] = val;
        } else if addr == 0xFFFF { // Interrupt Enable
            self.ei = val;
        }
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        if addr <= 0x3FFF {        // ROM Bank
            return self.cart.rom[addr as usize];
        } else if addr >= 0x4000 && addr <= 0x7FFF { // ROM Bank 1-n
            return self.cart.rom[addr as usize];
        } else if addr >= 0x8000 && addr <= 0x9FFF { // VRAM
            return self.vram[(addr - 0x8000) as usize];
        } else if addr >= 0xA000 && addr <= 0xBFFF { // Cart RAM
            return self.cart.ram[(addr - 0xA000) as usize];
        } else if addr >= 0xC000 && addr <= 0xDFFF { // Low RAM
            return self.wram[(addr - 0xC000) as usize];
        } else if addr >= 0xE000 && addr <= 0xFDFF { // Low RAM Duplicate
            return self.wram[(addr - 0xE000) as usize];
        } else if addr >= 0xFE00 && addr <= 0xFE9F { // OAM RAM
            return self.oam[(addr - 0xFE00) as usize];
        } else if addr >= 0xFF00 && addr <= 0xFF7F { // I/O Registers
            return self.regs[(addr - 0xFF00) as usize];
        } else if addr >= 0xFF80 && addr <= 0xFFFE { // High RAM (Stack)
            return self.stack[(addr - 0xFF80) as usize];
        } else if addr == 0xFFFF { // Interrupt Enable
            return self.ei;
        }
        return 0;
    }

    pub fn print_memory(&mut self) {
        for i in 0..0x200/0x10 {
            let mut line = format!("{:#4X}0: ", i);
            for j in 0..0x10/2 {
                let mut address = format!("{:2X}{:2X} ",
                                          self.cart.rom[(i*0x10 + j*2) as usize],
                                          self.cart.rom[(i*0x10 + j*2 + 1) as usize],
                );
                line.push_str(&address);
            }
            println!("{}", line);
        }
    }
}

impl GB {
    pub fn load_application(&mut self, filename: &str) -> bool {
        self.cart.load_application(filename)
    }
}

impl GB {
    pub fn emulate_cycle(&mut self) -> u32 {
        let mut pc_inc: u16 = 0;
        let opcode = (self.mem_read(self.cpu.pc), self.mem_read(self.cpu.pc+1));
        match opcode {
            // RLC
            (0xCB, 0x00) => { GB::shift_b(self, &GB::rlc) }
            (0xCB, 0x01) => { GB::shift_c(self, &GB::rlc) }
            (0xCB, 0x02) => { GB::shift_d(self, &GB::rlc) }
            (0xCB, 0x03) => { GB::shift_e(self, &GB::rlc) }
            (0xCB, 0x04) => { GB::shift_h(self, &GB::rlc) }
            (0xCB, 0x05) => { GB::shift_l(self, &GB::rlc) }
            (0xCB, 0x06) => { GB::shift_mem(self, &GB::rlc) }
            (0xCB, 0x07) => { GB::shift_a(self, &GB::rlc) }
            // RRC
            (0xCB, 0x08) => { GB::shift_b(self, &GB::rrc) }
            (0xCB, 0x09) => { GB::shift_c(self, &GB::rrc) }
            (0xCB, 0x0A) => { GB::shift_d(self, &GB::rrc) }
            (0xCB, 0x0B) => { GB::shift_e(self, &GB::rrc) }
            (0xCB, 0x0C) => { GB::shift_h(self, &GB::rrc) }
            (0xCB, 0x0D) => { GB::shift_l(self, &GB::rrc) }
            (0xCB, 0x0E) => { GB::shift_mem(self, &GB::rrc) }
            (0xCB, 0x0F) => { GB::shift_a(self, &GB::rrc) }
            // RL
            (0xCB, 0x10) => { GB::shift_b(self, &GB::rl) }
            (0xCB, 0x11) => { GB::shift_c(self, &GB::rl) }
            (0xCB, 0x12) => { GB::shift_d(self, &GB::rl) }
            (0xCB, 0x13) => { GB::shift_e(self, &GB::rl) }
            (0xCB, 0x14) => { GB::shift_h(self, &GB::rl) }
            (0xCB, 0x15) => { GB::shift_l(self, &GB::rl) }
            (0xCB, 0x16) => { GB::shift_mem(self, &GB::rl) }
            (0xCB, 0x17) => { GB::shift_a(self, &GB::rl) }
            // RR
            (0xCB, 0x18) => { GB::shift_b(self, &GB::rr) }
            (0xCB, 0x19) => { GB::shift_c(self, &GB::rr) }
            (0xCB, 0x1A) => { GB::shift_d(self, &GB::rr) }
            (0xCB, 0x1B) => { GB::shift_e(self, &GB::rr) }
            (0xCB, 0x1C) => { GB::shift_h(self, &GB::rr) }
            (0xCB, 0x1D) => { GB::shift_l(self, &GB::rr) }
            (0xCB, 0x1E) => { GB::shift_mem(self, &GB::rr) }
            (0xCB, 0x1F) => { GB::shift_a(self, &GB::rr) }
            // SLA
            (0xCB, 0x20) => { GB::shift_b(self, &GB::sla) }
            (0xCB, 0x21) => { GB::shift_c(self, &GB::sla) }
            (0xCB, 0x22) => { GB::shift_d(self, &GB::sla) }
            (0xCB, 0x23) => { GB::shift_e(self, &GB::sla) }
            (0xCB, 0x24) => { GB::shift_h(self, &GB::sla) }
            (0xCB, 0x25) => { GB::shift_l(self, &GB::sla) }
            (0xCB, 0x26) => { GB::shift_mem(self, &GB::sla) }
            (0xCB, 0x27) => { GB::shift_a(self, &GB::sla) }
            // SRA
            (0xCB, 0x28) => { GB::shift_b(self, &GB::sra) }
            (0xCB, 0x29) => { GB::shift_c(self, &GB::sra) }
            (0xCB, 0x2A) => { GB::shift_d(self, &GB::sra) }
            (0xCB, 0x2B) => { GB::shift_e(self, &GB::sra) }
            (0xCB, 0x2C) => { GB::shift_h(self, &GB::sra) }
            (0xCB, 0x2D) => { GB::shift_l(self, &GB::sra) }
            (0xCB, 0x2E) => { GB::shift_mem(self, &GB::sra) }
            (0xCB, 0x2F) => { GB::shift_a(self, &GB::sra) }
            // SWAP
            (0xCB, 0x30) => { GB::shift_b(self, &GB::swap) }
            (0xCB, 0x31) => { GB::shift_c(self, &GB::swap) }
            (0xCB, 0x32) => { GB::shift_d(self, &GB::swap) }
            (0xCB, 0x33) => { GB::shift_e(self, &GB::swap) }
            (0xCB, 0x34) => { GB::shift_h(self, &GB::swap) }
            (0xCB, 0x35) => { GB::shift_l(self, &GB::swap) }
            (0xCB, 0x36) => { GB::shift_mem(self, &GB::swap) }
            (0xCB, 0x37) => { GB::shift_a(self, &GB::swap) }
            // SRL
            (0xCB, 0x38) => { GB::shift_b(self, &GB::srl) }
            (0xCB, 0x39) => { GB::shift_c(self, &GB::srl) }
            (0xCB, 0x3A) => { GB::shift_d(self, &GB::srl) }
            (0xCB, 0x3B) => { GB::shift_e(self, &GB::srl) }
            (0xCB, 0x3C) => { GB::shift_h(self, &GB::srl) }
            (0xCB, 0x3D) => { GB::shift_l(self, &GB::srl) }
            (0xCB, 0x3E) => { GB::shift_mem(self, &GB::srl) }
            (0xCB, 0x3F) => { GB::shift_a(self, &GB::srl) }
            // BIT 0
            (0xCB, 0x40) => { GB::shift_b(self, &GB::bit_0) }
            (0xCB, 0x41) => { GB::shift_c(self, &GB::bit_0) }
            (0xCB, 0x42) => { GB::shift_d(self, &GB::bit_0) }
            (0xCB, 0x43) => { GB::shift_e(self, &GB::bit_0) }
            (0xCB, 0x44) => { GB::shift_h(self, &GB::bit_0) }
            (0xCB, 0x45) => { GB::shift_l(self, &GB::bit_0) }
            (0xCB, 0x46) => { GB::shift_mem(self, &GB::bit_0) }
            (0xCB, 0x47) => { GB::shift_a(self, &GB::bit_0) }
            // BIT 1
            (0xCB, 0x48) => { GB::shift_b(self, &GB::bit_1) }
            (0xCB, 0x49) => { GB::shift_c(self, &GB::bit_1) }
            (0xCB, 0x4A) => { GB::shift_d(self, &GB::bit_1) }
            (0xCB, 0x4B) => { GB::shift_e(self, &GB::bit_1) }
            (0xCB, 0x4C) => { GB::shift_h(self, &GB::bit_1) }
            (0xCB, 0x4D) => { GB::shift_l(self, &GB::bit_1) }
            (0xCB, 0x4E) => { GB::shift_mem(self, &GB::bit_1) }
            (0xCB, 0x4F) => { GB::shift_a(self, &GB::bit_1) }
            // BIT 2
            (0xCB, 0x50) => { GB::shift_b(self, &GB::bit_2) }
            (0xCB, 0x51) => { GB::shift_c(self, &GB::bit_2) }
            (0xCB, 0x52) => { GB::shift_d(self, &GB::bit_2) }
            (0xCB, 0x53) => { GB::shift_e(self, &GB::bit_2) }
            (0xCB, 0x54) => { GB::shift_h(self, &GB::bit_2) }
            (0xCB, 0x55) => { GB::shift_l(self, &GB::bit_2) }
            (0xCB, 0x56) => { GB::shift_mem(self, &GB::bit_2) }
            (0xCB, 0x57) => { GB::shift_a(self, &GB::bit_2) }
            // BIT 3
            (0xCB, 0x58) => { GB::shift_b(self, &GB::bit_3) }
            (0xCB, 0x59) => { GB::shift_c(self, &GB::bit_3) }
            (0xCB, 0x5A) => { GB::shift_d(self, &GB::bit_3) }
            (0xCB, 0x5B) => { GB::shift_e(self, &GB::bit_3) }
            (0xCB, 0x5C) => { GB::shift_h(self, &GB::bit_3) }
            (0xCB, 0x5D) => { GB::shift_l(self, &GB::bit_3) }
            (0xCB, 0x5E) => { GB::shift_mem(self, &GB::bit_3) }
            (0xCB, 0x5F) => { GB::shift_a(self, &GB::bit_3) }
            // BIT 4
            (0xCB, 0x60) => { GB::shift_b(self, &GB::bit_4) }
            (0xCB, 0x61) => { GB::shift_c(self, &GB::bit_4) }
            (0xCB, 0x62) => { GB::shift_d(self, &GB::bit_4) }
            (0xCB, 0x63) => { GB::shift_e(self, &GB::bit_4) }
            (0xCB, 0x64) => { GB::shift_h(self, &GB::bit_4) }
            (0xCB, 0x65) => { GB::shift_l(self, &GB::bit_4) }
            (0xCB, 0x66) => { GB::shift_mem(self, &GB::bit_4) }
            (0xCB, 0x67) => { GB::shift_a(self, &GB::bit_4) }
            // BIT 5
            (0xCB, 0x68) => { GB::shift_b(self, &GB::bit_5) }
            (0xCB, 0x69) => { GB::shift_c(self, &GB::bit_5) }
            (0xCB, 0x6A) => { GB::shift_d(self, &GB::bit_5) }
            (0xCB, 0x6B) => { GB::shift_e(self, &GB::bit_5) }
            (0xCB, 0x6C) => { GB::shift_h(self, &GB::bit_5) }
            (0xCB, 0x6D) => { GB::shift_l(self, &GB::bit_5) }
            (0xCB, 0x6E) => { GB::shift_mem(self, &GB::bit_5) }
            (0xCB, 0x6F) => { GB::shift_a(self, &GB::bit_5) }
            // BIT 6
            (0xCB, 0x70) => { GB::shift_b(self, &GB::bit_6) }
            (0xCB, 0x71) => { GB::shift_c(self, &GB::bit_6) }
            (0xCB, 0x72) => { GB::shift_d(self, &GB::bit_6) }
            (0xCB, 0x73) => { GB::shift_e(self, &GB::bit_6) }
            (0xCB, 0x74) => { GB::shift_h(self, &GB::bit_6) }
            (0xCB, 0x75) => { GB::shift_l(self, &GB::bit_6) }
            (0xCB, 0x76) => { GB::shift_mem(self, &GB::bit_6) }
            (0xCB, 0x77) => { GB::shift_a(self, &GB::bit_6) }
            // BIT 7
            (0xCB, 0x78) => { GB::shift_b(self, &GB::bit_7) }
            (0xCB, 0x79) => { GB::shift_c(self, &GB::bit_7) }
            (0xCB, 0x7A) => { GB::shift_d(self, &GB::bit_7) }
            (0xCB, 0x7B) => { GB::shift_e(self, &GB::bit_7) }
            (0xCB, 0x7C) => { GB::shift_h(self, &GB::bit_7) }
            (0xCB, 0x7D) => { GB::shift_l(self, &GB::bit_7) }
            (0xCB, 0x7E) => { GB::shift_mem(self, &GB::bit_7) }
            (0xCB, 0x7F) => { GB::shift_a(self, &GB::bit_7) }
            (_, _)  => { panic!("Unknown opcode") }
        }
    }

    pub fn shift_a(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_a();
        r = f(gb, r);
        gb.cpu.set_a(r);
        return 8;
    }
    pub fn shift_b(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_b();
        r = f(gb, r);
        gb.cpu.set_b(r);
        return 8;
    }
    pub fn shift_c(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_c();
        r = f(gb, r);
        gb.cpu.set_c(r);
        return 8;
    }
    pub fn shift_d(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_d();
        r = f(gb, r);
        gb.cpu.set_d(r);
        return 8;
    }
    pub fn shift_e(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_e();
        r = f(gb, r);
        gb.cpu.set_e(r);
        return 8;
    }
    pub fn shift_h(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_h();
        r = f(gb, r);
        gb.cpu.set_h(r);
        return 8;
    }
    pub fn shift_l(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let mut r = gb.cpu.get_l();
        r = f(gb, r);
        gb.cpu.set_l(r);
        return 8;
    }
    pub fn shift_mem(mut gb: &mut GB, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let addr = gb.cpu.get_hl();
        let mut r = gb.mem_read(addr);
        r = f(gb, r);
        gb.mem_write(addr, r);
        return 16;
    }

    // Shifting functions
    fn rlc(&mut self, mut r: u8) -> u8 {
        let cy = r >> 7;
        r = (r << 1) | cy;
        self.cpu.set_cy(cy);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }
    fn rrc(mut gb: &mut GB, mut r: u8) -> u8 {
        let cy = r & 1;
        r = (r >> 1) | (cy << 7);
        gb.cpu.set_cy(cy);
        if r == 0 {
            gb.cpu.set_z(1);
        }
        return r;
    }
    fn rl(&mut self, mut r: u8) -> u8 {
        let cy = r >> 7;
        r = (r << 1) | self.cpu.get_cy();
        self.cpu.set_cy(cy);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }
    fn rr(&mut self, mut r: u8) -> u8 {
        let cy = r & 1;
        r = (r >> 1) | (self.cpu.get_cy() << 7);
        self.cpu.set_cy(cy);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }
    fn sla(&mut self, mut r: u8) -> u8 {
        let cy = r >> 7;
        r = (r << 1);
        self.cpu.set_cy(cy);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }
    fn sra(&mut self, mut r: u8) -> u8 {
        let cy = r & 1;
        self.cpu.set_cy(cy);
        let sign = r >> 7;
        r = (r >> 1) | (sign << 7);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }
    fn swap(&mut self, mut r: u8) -> u8 {
        let sign = r >> 7;
        r = (r >> 4) | (r << 4);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }
    fn srl(&mut self, mut r: u8) -> u8 {
        let cy = r & 1;
        r = (r >> 1);
        self.cpu.set_cy(cy);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return r;
    }


    //BIT testing functions
    fn bit_0(&mut self, mut r: u8) -> u8 { return self.bit(r, 0); }
    fn bit_1(&mut self, mut r: u8) -> u8 { return self.bit(r, 1); }
    fn bit_2(&mut self, mut r: u8) -> u8 { return self.bit(r, 2); }
    fn bit_3(&mut self, mut r: u8) -> u8 { return self.bit(r, 3); }
    fn bit_4(&mut self, mut r: u8) -> u8 { return self.bit(r, 4); }
    fn bit_5(&mut self, mut r: u8) -> u8 { return self.bit(r, 5); }
    fn bit_6(&mut self, mut r: u8) -> u8 { return self.bit(r, 6); }
    fn bit_7(&mut self, mut r: u8) -> u8 { return self.bit(r, 7); }

    fn bit(&mut self, mut r: u8, i: u8) -> u8 {
        println!("{:b} >> {} {:b}", r, i, (r >> i) & 1 );
        self.cpu.set_z((r >> i) & 1);
        return r;
    }
}


// RLC Tests
#[test]
fn rlc_b_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001100);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::rlc);
    assert_eq!(gb.cpu.get_b(), 0b10011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rlc_b_no_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110011);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::rlc);
    assert_eq!(gb.cpu.get_b(), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn rlc_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::rlc);
    assert_eq!(gb.mem_read(addr), 0b10011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rlc_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::rlc);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}


// RRC Tests
#[test]
fn rrc_b_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110011);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::rrc);
    assert_eq!(gb.cpu.get_b(), 0b10011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rrc_b_no_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001100);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::rrc);
    assert_eq!(gb.cpu.get_b(), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn rrc_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.cpu.set_cy(0);
    GB::shift_mem(&mut gb, &GB::rrc);
    assert_eq!(gb.mem_read(addr), 0b10011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rrc_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::rrc);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}


// RL Tests
#[test]
fn rl_b_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001100);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::rl);
    assert_eq!(gb.cpu.get_b(), 0b10011000);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rl_b_no_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110011);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::rl);
    assert_eq!(gb.cpu.get_b(), 0b01100111);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn rl_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.cpu.set_cy(0);
    GB::shift_mem(&mut gb, &GB::rl);
    assert_eq!(gb.mem_read(addr), 0b10011000);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rl_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::rl);
    assert_eq!(gb.mem_read(addr), 0b01100111);
    assert_eq!(gb.cpu.get_cy(), 0);
}


// RR Tests
#[test]
fn rr_b_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110011);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::rr);
    assert_eq!(gb.cpu.get_b(), 0b00011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rr_b_no_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001100);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::rr);
    assert_eq!(gb.cpu.get_b(), 0b11100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn rr_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.cpu.set_cy(0);
    GB::shift_mem(&mut gb, &GB::rr);
    assert_eq!(gb.mem_read(addr), 0b00011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn rr_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::rr);
    assert_eq!(gb.mem_read(addr), 0b11100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}


// SLA Tests
#[test]
fn sla_b_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001100);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::sla);
    assert_eq!(gb.cpu.get_b(), 0b10011000);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn sla_b_no_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110011);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::sla);
    assert_eq!(gb.cpu.get_b(), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn sla_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.cpu.set_cy(0);
    GB::shift_mem(&mut gb, &GB::sla);
    assert_eq!(gb.mem_read(addr), 0b10011000);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn sla_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::sla);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 0);
}


// SRA Tests
#[test]
fn sra_b_positive() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110010);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::sra);
    assert_eq!(gb.cpu.get_b(), 0b00011001);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn sra_b_negative() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001101);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::sra);
    assert_eq!(gb.cpu.get_b(), 0b11100110);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn sra_hl_positive() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110010);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::sra);
    assert_eq!(gb.mem_read(addr), 0b00011001);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn sra_hl_negative() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001101);
    gb.cpu.set_cy(0);
    GB::shift_mem(&mut gb, &GB::sra);
    assert_eq!(gb.mem_read(addr), 0b11100110);
    assert_eq!(gb.cpu.get_cy(), 1);
}


// SWAP Tests
#[test]
fn swap_b() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b10110100);
    GB::shift_b(&mut gb, &GB::swap);
    assert_eq!(gb.cpu.get_b(), 0b01001011);
}
#[test]
fn swap_hl() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b10110100);
    GB::shift_mem(&mut gb, &GB::swap);
    assert_eq!(gb.mem_read(addr), 0b01001011);
}


// SRL Tests
#[test]
fn srl_b_positive() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110010);
    gb.cpu.set_cy(1);
    GB::shift_b(&mut gb, &GB::srl);
    assert_eq!(gb.cpu.get_b(), 0b00011001);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn srl_b_negative() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001101);
    gb.cpu.set_cy(0);
    GB::shift_b(&mut gb, &GB::srl);
    assert_eq!(gb.cpu.get_b(), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 1);
}
#[test]
fn srl_hl_positive() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00110010);
    gb.cpu.set_cy(1);
    GB::shift_mem(&mut gb, &GB::srl);
    assert_eq!(gb.mem_read(addr), 0b00011001);
    assert_eq!(gb.cpu.get_cy(), 0);
}
#[test]
fn srl_hl_negative() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11001101);
    gb.cpu.set_cy(0);
    GB::shift_mem(&mut gb, &GB::srl);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.cpu.get_cy(), 1);
}


// BIT Tests
#[test]
fn bit_0_b_on() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00000001);
    gb.cpu.set_z(0);
    GB::shift_b(&mut gb, &GB::bit_0);
    assert_eq!(gb.cpu.get_b(), 0b00000001);
    assert_eq!(gb.cpu.get_z(), 1);
}
#[test]
fn bit_0_b_off() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11111110);
    gb.cpu.set_z(1);
    GB::shift_b(&mut gb, &GB::bit_0);
    assert_eq!(gb.cpu.get_b(), 0b11111110);
    assert_eq!(gb.cpu.get_z(), 0);
}
#[test]
fn bit_0_hl_on() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b00000001);
    gb.cpu.set_z(0);
    GB::shift_mem(&mut gb, &GB::bit_0);
    assert_eq!(gb.mem_read(addr), 0b00000001);
    assert_eq!(gb.cpu.get_z(), 1);
}
#[test]
fn bit_0_hl_off() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b11111110);
    gb.cpu.set_z(1);
    GB::shift_mem(&mut gb, &GB::bit_0);
    assert_eq!(gb.mem_read(addr), 0b11111110);
    assert_eq!(gb.cpu.get_z(), 0);
}
#[test]
fn bit_6_b_on() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b01000000);
    gb.cpu.set_z(0);
    GB::shift_b(&mut gb, &GB::bit_6);
    assert_eq!(gb.cpu.get_b(), 0b01000000);
    assert_eq!(gb.cpu.get_z(), 1);
}
#[test]
fn bit_6_b_off() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b10111111);
    gb.cpu.set_z(1);
    GB::shift_b(&mut gb, &GB::bit_6);
    assert_eq!(gb.cpu.get_b(), 0b10111111);
    assert_eq!(gb.cpu.get_z(), 0);
}
#[test]
fn bit_6_hl_on() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b01000000);
    gb.cpu.set_z(0);
    GB::shift_mem(&mut gb, &GB::bit_6);
    assert_eq!(gb.mem_read(addr), 0b01000000);
    assert_eq!(gb.cpu.get_z(), 1);
}
#[test]
fn bit_6_hl_off() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.cpu.set_hl(addr);
    gb.mem_write(addr, 0b10111111);
    gb.cpu.set_z(1);
    GB::shift_mem(&mut gb, &GB::bit_6);
    assert_eq!(gb.mem_read(addr), 0b10111111);
    assert_eq!(gb.cpu.get_z(), 0);
}
