use std::fs::File;
use std::io::prelude::*;

pub struct GB {
    wram: [u8; 8192],
    vram: [u8; 8192],
    cart: Cartridge,
    regs: [u8; 0x80],
    oam: [u8; 0xA0],
    ime: u8,
    stack: [u8; 0x180],

    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
}

impl GB {
    pub fn new() -> GB {
        return GB {
            wram: [0; 8192],
            vram: [0; 8192],
            cart: Cartridge::new(),
            regs: [0; 0x80],
            oam: [0; 0xA0],
            ime: 0,
            stack: [0; 0x180],

            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0xFFFE,
            pc: 0,
        }
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
            self.ime = val;
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
            return self.ime; //TODO: This might not be correct
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

impl GB {
    pub fn load_application(&mut self, filename: &str) -> bool {
        self.cart.load_application(filename)
    }
}

impl GB {

    pub fn emulate_cycle(&mut self) -> u32 {
        let mut pc_inc: u16 = 0;
        let opcode = (self.mem_read(self.pc), self.mem_read(self.pc+1));
        match opcode {
            // RLC
            (0xCB, 0x00) => { self.shift_r8(&GB::get_b, &GB::rlc, &GB::set_b) }
            (0xCB, 0x01) => { self.shift_r8(&GB::get_c, &GB::rlc, &GB::set_c) }
            (0xCB, 0x02) => { self.shift_r8(&GB::get_d, &GB::rlc, &GB::set_d) }
            (0xCB, 0x03) => { self.shift_r8(&GB::get_e, &GB::rlc, &GB::set_e) }
            (0xCB, 0x04) => { self.shift_r8(&GB::get_h, &GB::rlc, &GB::set_h) }
            (0xCB, 0x05) => { self.shift_r8(&GB::get_l, &GB::rlc, &GB::set_l) }
            (0xCB, 0x06) => { self.shift_mem(&GB::rlc) }
            (0xCB, 0x07) => { self.shift_r8(&GB::get_a, &GB::rlc, &GB::set_a) }
            // RRC
            (0xCB, 0x08) => { self.shift_r8(&GB::get_b, &GB::rrc, &GB::set_b) }
            (0xCB, 0x09) => { self.shift_r8(&GB::get_c, &GB::rrc, &GB::set_c) }
            (0xCB, 0x0A) => { self.shift_r8(&GB::get_d, &GB::rrc, &GB::set_d) }
            (0xCB, 0x0B) => { self.shift_r8(&GB::get_e, &GB::rrc, &GB::set_e) }
            (0xCB, 0x0C) => { self.shift_r8(&GB::get_h, &GB::rrc, &GB::set_h) }
            (0xCB, 0x0D) => { self.shift_r8(&GB::get_l, &GB::rrc, &GB::set_l) }
            (0xCB, 0x0E) => { self.shift_mem(&GB::rrc) }
            (0xCB, 0x0F) => { self.shift_r8(&GB::get_a, &GB::rrc, &GB::set_a) }
            // RL
            (0xCB, 0x10) => { self.shift_r8(&GB::get_b, &GB::rl, &GB::set_b) }
            (0xCB, 0x11) => { self.shift_r8(&GB::get_c, &GB::rl, &GB::set_c) }
            (0xCB, 0x12) => { self.shift_r8(&GB::get_d, &GB::rl, &GB::set_d) }
            (0xCB, 0x13) => { self.shift_r8(&GB::get_e, &GB::rl, &GB::set_e) }
            (0xCB, 0x14) => { self.shift_r8(&GB::get_h, &GB::rl, &GB::set_h) }
            (0xCB, 0x15) => { self.shift_r8(&GB::get_l, &GB::rl, &GB::set_l) }
            (0xCB, 0x16) => { self.shift_mem(&GB::rl) }
            (0xCB, 0x17) => { self.shift_r8(&GB::get_a, &GB::rl, &GB::set_a) }
            // RR
            (0xCB, 0x18) => { self.shift_r8(&GB::get_b, &GB::rr, &GB::set_b) }
            (0xCB, 0x19) => { self.shift_r8(&GB::get_c, &GB::rr, &GB::set_c) }
            (0xCB, 0x1A) => { self.shift_r8(&GB::get_d, &GB::rr, &GB::set_d) }
            (0xCB, 0x1B) => { self.shift_r8(&GB::get_e, &GB::rr, &GB::set_e) }
            (0xCB, 0x1C) => { self.shift_r8(&GB::get_h, &GB::rr, &GB::set_h) }
            (0xCB, 0x1D) => { self.shift_r8(&GB::get_l, &GB::rr, &GB::set_l) }
            (0xCB, 0x1E) => { self.shift_mem(&GB::rr) }
            (0xCB, 0x1F) => { self.shift_r8(&GB::get_a, &GB::rr, &GB::set_a) }
            // SLA
            (0xCB, 0x20) => { self.shift_r8(&GB::get_b, &GB::sla, &GB::set_b) }
            (0xCB, 0x21) => { self.shift_r8(&GB::get_c, &GB::sla, &GB::set_c) }
            (0xCB, 0x22) => { self.shift_r8(&GB::get_d, &GB::sla, &GB::set_d) }
            (0xCB, 0x23) => { self.shift_r8(&GB::get_e, &GB::sla, &GB::set_e) }
            (0xCB, 0x24) => { self.shift_r8(&GB::get_h, &GB::sla, &GB::set_h) }
            (0xCB, 0x25) => { self.shift_r8(&GB::get_l, &GB::sla, &GB::set_l) }
            (0xCB, 0x26) => { self.shift_mem(&GB::sla) }
            (0xCB, 0x27) => { self.shift_r8(&GB::get_a, &GB::sla, &GB::set_a) }
            // SRA
            (0xCB, 0x28) => { self.shift_r8(&GB::get_b, &GB::sra, &GB::set_b) }
            (0xCB, 0x29) => { self.shift_r8(&GB::get_c, &GB::sra, &GB::set_c) }
            (0xCB, 0x2A) => { self.shift_r8(&GB::get_d, &GB::sra, &GB::set_d) }
            (0xCB, 0x2B) => { self.shift_r8(&GB::get_e, &GB::sra, &GB::set_e) }
            (0xCB, 0x2C) => { self.shift_r8(&GB::get_h, &GB::sra, &GB::set_h) }
            (0xCB, 0x2D) => { self.shift_r8(&GB::get_l, &GB::sra, &GB::set_l) }
            (0xCB, 0x2E) => { self.shift_mem(&GB::sra) }
            (0xCB, 0x2F) => { self.shift_r8(&GB::get_a, &GB::sra, &GB::set_a) }
            // SWAP
            (0xCB, 0x30) => { self.shift_r8(&GB::get_b, &GB::swap, &GB::set_b) }
            (0xCB, 0x31) => { self.shift_r8(&GB::get_c, &GB::swap, &GB::set_c) }
            (0xCB, 0x32) => { self.shift_r8(&GB::get_d, &GB::swap, &GB::set_d) }
            (0xCB, 0x33) => { self.shift_r8(&GB::get_e, &GB::swap, &GB::set_e) }
            (0xCB, 0x34) => { self.shift_r8(&GB::get_h, &GB::swap, &GB::set_h) }
            (0xCB, 0x35) => { self.shift_r8(&GB::get_l, &GB::swap, &GB::set_l) }
            (0xCB, 0x36) => { self.shift_mem(&GB::swap) }
            (0xCB, 0x37) => { self.shift_r8(&GB::get_a, &GB::swap, &GB::set_a) }
            // SRL
            (0xCB, 0x38) => { self.shift_r8(&GB::get_b, &GB::srl, &GB::set_b) }
            (0xCB, 0x39) => { self.shift_r8(&GB::get_c, &GB::srl, &GB::set_c) }
            (0xCB, 0x3A) => { self.shift_r8(&GB::get_d, &GB::srl, &GB::set_d) }
            (0xCB, 0x3B) => { self.shift_r8(&GB::get_e, &GB::srl, &GB::set_e) }
            (0xCB, 0x3C) => { self.shift_r8(&GB::get_h, &GB::srl, &GB::set_h) }
            (0xCB, 0x3D) => { self.shift_r8(&GB::get_l, &GB::srl, &GB::set_l) }
            (0xCB, 0x3E) => { self.shift_mem(&GB::srl) }
            (0xCB, 0x3F) => { self.shift_r8(&GB::get_a, &GB::srl, &GB::set_a) }
            // BIT 0
            (0xCB, 0x40) => { self.shift_r8(&GB::get_b, &GB::bit_0, &GB::set_b) }
            (0xCB, 0x41) => { self.shift_r8(&GB::get_c, &GB::bit_0, &GB::set_c) }
            (0xCB, 0x42) => { self.shift_r8(&GB::get_d, &GB::bit_0, &GB::set_d) }
            (0xCB, 0x43) => { self.shift_r8(&GB::get_e, &GB::bit_0, &GB::set_e) }
            (0xCB, 0x44) => { self.shift_r8(&GB::get_h, &GB::bit_0, &GB::set_h) }
            (0xCB, 0x45) => { self.shift_r8(&GB::get_l, &GB::bit_0, &GB::set_l) }
            (0xCB, 0x46) => { self.shift_mem(&GB::bit_0) }
            (0xCB, 0x47) => { self.shift_r8(&GB::get_a, &GB::bit_0, &GB::set_a) }
            // BIT 1
            (0xCB, 0x48) => { self.shift_r8(&GB::get_b, &GB::bit_1, &GB::set_b) }
            (0xCB, 0x49) => { self.shift_r8(&GB::get_c, &GB::bit_1, &GB::set_c) }
            (0xCB, 0x4A) => { self.shift_r8(&GB::get_d, &GB::bit_1, &GB::set_d) }
            (0xCB, 0x4B) => { self.shift_r8(&GB::get_e, &GB::bit_1, &GB::set_e) }
            (0xCB, 0x4C) => { self.shift_r8(&GB::get_h, &GB::bit_1, &GB::set_h) }
            (0xCB, 0x4D) => { self.shift_r8(&GB::get_l, &GB::bit_1, &GB::set_l) }
            (0xCB, 0x4E) => { self.shift_mem(&GB::bit_1) }
            (0xCB, 0x4F) => { self.shift_r8(&GB::get_a, &GB::bit_1, &GB::set_a) }
            // BIT 2
            (0xCB, 0x50) => { self.shift_r8(&GB::get_b, &GB::bit_2, &GB::set_b) }
            (0xCB, 0x51) => { self.shift_r8(&GB::get_c, &GB::bit_2, &GB::set_c) }
            (0xCB, 0x52) => { self.shift_r8(&GB::get_d, &GB::bit_2, &GB::set_d) }
            (0xCB, 0x53) => { self.shift_r8(&GB::get_e, &GB::bit_2, &GB::set_e) }
            (0xCB, 0x54) => { self.shift_r8(&GB::get_h, &GB::bit_2, &GB::set_h) }
            (0xCB, 0x55) => { self.shift_r8(&GB::get_l, &GB::bit_2, &GB::set_l) }
            (0xCB, 0x56) => { self.shift_mem(&GB::bit_2) }
            (0xCB, 0x57) => { self.shift_r8(&GB::get_a, &GB::bit_2, &GB::set_a) }
            // BIT 3
            (0xCB, 0x58) => { self.shift_r8(&GB::get_b, &GB::bit_3, &GB::set_b) }
            (0xCB, 0x59) => { self.shift_r8(&GB::get_c, &GB::bit_3, &GB::set_c) }
            (0xCB, 0x5A) => { self.shift_r8(&GB::get_d, &GB::bit_3, &GB::set_d) }
            (0xCB, 0x5B) => { self.shift_r8(&GB::get_e, &GB::bit_3, &GB::set_e) }
            (0xCB, 0x5C) => { self.shift_r8(&GB::get_h, &GB::bit_3, &GB::set_h) }
            (0xCB, 0x5D) => { self.shift_r8(&GB::get_l, &GB::bit_3, &GB::set_l) }
            (0xCB, 0x5E) => { self.shift_mem(&GB::bit_3) }
            (0xCB, 0x5F) => { self.shift_r8(&GB::get_a, &GB::bit_3, &GB::set_a) }
            // BIT 4
            (0xCB, 0x60) => { self.shift_r8(&GB::get_b, &GB::bit_4, &GB::set_b) }
            (0xCB, 0x61) => { self.shift_r8(&GB::get_c, &GB::bit_4, &GB::set_c) }
            (0xCB, 0x62) => { self.shift_r8(&GB::get_d, &GB::bit_4, &GB::set_d) }
            (0xCB, 0x63) => { self.shift_r8(&GB::get_e, &GB::bit_4, &GB::set_e) }
            (0xCB, 0x64) => { self.shift_r8(&GB::get_h, &GB::bit_4, &GB::set_h) }
            (0xCB, 0x65) => { self.shift_r8(&GB::get_l, &GB::bit_4, &GB::set_l) }
            (0xCB, 0x66) => { self.shift_mem(&GB::bit_4) }
            (0xCB, 0x67) => { self.shift_r8(&GB::get_a, &GB::bit_4, &GB::set_a) }
            // BIT 5
            (0xCB, 0x68) => { self.shift_r8(&GB::get_b, &GB::bit_5, &GB::set_b) }
            (0xCB, 0x69) => { self.shift_r8(&GB::get_c, &GB::bit_5, &GB::set_c) }
            (0xCB, 0x6A) => { self.shift_r8(&GB::get_d, &GB::bit_5, &GB::set_d) }
            (0xCB, 0x6B) => { self.shift_r8(&GB::get_e, &GB::bit_5, &GB::set_e) }
            (0xCB, 0x6C) => { self.shift_r8(&GB::get_h, &GB::bit_5, &GB::set_h) }
            (0xCB, 0x6D) => { self.shift_r8(&GB::get_l, &GB::bit_5, &GB::set_l) }
            (0xCB, 0x6E) => { self.shift_mem(&GB::bit_5) }
            (0xCB, 0x6F) => { self.shift_r8(&GB::get_a, &GB::bit_5, &GB::set_a) }
            // BIT 6
            (0xCB, 0x70) => { self.shift_r8(&GB::get_b, &GB::bit_6, &GB::set_b) }
            (0xCB, 0x71) => { self.shift_r8(&GB::get_c, &GB::bit_6, &GB::set_c) }
            (0xCB, 0x72) => { self.shift_r8(&GB::get_d, &GB::bit_6, &GB::set_d) }
            (0xCB, 0x73) => { self.shift_r8(&GB::get_e, &GB::bit_6, &GB::set_e) }
            (0xCB, 0x74) => { self.shift_r8(&GB::get_h, &GB::bit_6, &GB::set_h) }
            (0xCB, 0x75) => { self.shift_r8(&GB::get_l, &GB::bit_6, &GB::set_l) }
            (0xCB, 0x76) => { self.shift_mem(&GB::bit_6) }
            (0xCB, 0x77) => { self.shift_r8(&GB::get_a, &GB::bit_6, &GB::set_a) }
            // BIT 7
            (0xCB, 0x78) => { self.shift_r8(&GB::get_b, &GB::bit_7, &GB::set_b) }
            (0xCB, 0x79) => { self.shift_r8(&GB::get_c, &GB::bit_7, &GB::set_c) }
            (0xCB, 0x7A) => { self.shift_r8(&GB::get_d, &GB::bit_7, &GB::set_d) }
            (0xCB, 0x7B) => { self.shift_r8(&GB::get_e, &GB::bit_7, &GB::set_e) }
            (0xCB, 0x7C) => { self.shift_r8(&GB::get_h, &GB::bit_7, &GB::set_h) }
            (0xCB, 0x7D) => { self.shift_r8(&GB::get_l, &GB::bit_7, &GB::set_l) }
            (0xCB, 0x7E) => { self.shift_mem(&GB::bit_7) }
            (0xCB, 0x7F) => { self.shift_r8(&GB::get_a, &GB::bit_7, &GB::set_a) }
            // RES 0
            (0xCB, 0x80) => { self.shift_r8(&GB::get_b, &GB::res_0, &GB::set_b) }
            (0xCB, 0x81) => { self.shift_r8(&GB::get_c, &GB::res_0, &GB::set_c) }
            (0xCB, 0x82) => { self.shift_r8(&GB::get_d, &GB::res_0, &GB::set_d) }
            (0xCB, 0x83) => { self.shift_r8(&GB::get_e, &GB::res_0, &GB::set_e) }
            (0xCB, 0x84) => { self.shift_r8(&GB::get_h, &GB::res_0, &GB::set_h) }
            (0xCB, 0x85) => { self.shift_r8(&GB::get_l, &GB::res_0, &GB::set_l) }
            (0xCB, 0x86) => { self.shift_mem(&GB::res_0) }
            (0xCB, 0x87) => { self.shift_r8(&GB::get_a, &GB::res_0, &GB::set_a) }
            // RES 1
            (0xCB, 0x88) => { self.shift_r8(&GB::get_b, &GB::res_1, &GB::set_b) }
            (0xCB, 0x89) => { self.shift_r8(&GB::get_c, &GB::res_1, &GB::set_c) }
            (0xCB, 0x8A) => { self.shift_r8(&GB::get_d, &GB::res_1, &GB::set_d) }
            (0xCB, 0x8B) => { self.shift_r8(&GB::get_e, &GB::res_1, &GB::set_e) }
            (0xCB, 0x8C) => { self.shift_r8(&GB::get_h, &GB::res_1, &GB::set_h) }
            (0xCB, 0x8D) => { self.shift_r8(&GB::get_l, &GB::res_1, &GB::set_l) }
            (0xCB, 0x8E) => { self.shift_mem(&GB::res_1) }
            (0xCB, 0x8F) => { self.shift_r8(&GB::get_a, &GB::res_1, &GB::set_a) }
            // RES 2
            (0xCB, 0x90) => { self.shift_r8(&GB::get_b, &GB::res_2, &GB::set_b) }
            (0xCB, 0x91) => { self.shift_r8(&GB::get_c, &GB::res_2, &GB::set_c) }
            (0xCB, 0x92) => { self.shift_r8(&GB::get_d, &GB::res_2, &GB::set_d) }
            (0xCB, 0x93) => { self.shift_r8(&GB::get_e, &GB::res_2, &GB::set_e) }
            (0xCB, 0x94) => { self.shift_r8(&GB::get_h, &GB::res_2, &GB::set_h) }
            (0xCB, 0x95) => { self.shift_r8(&GB::get_l, &GB::res_2, &GB::set_l) }
            (0xCB, 0x96) => { self.shift_mem(&GB::res_2) }
            (0xCB, 0x97) => { self.shift_r8(&GB::get_a, &GB::res_2, &GB::set_a) }
            // RES 3
            (0xCB, 0x98) => { self.shift_r8(&GB::get_b, &GB::res_3, &GB::set_b) }
            (0xCB, 0x99) => { self.shift_r8(&GB::get_c, &GB::res_3, &GB::set_c) }
            (0xCB, 0x9A) => { self.shift_r8(&GB::get_d, &GB::res_3, &GB::set_d) }
            (0xCB, 0x9B) => { self.shift_r8(&GB::get_e, &GB::res_3, &GB::set_e) }
            (0xCB, 0x9C) => { self.shift_r8(&GB::get_h, &GB::res_3, &GB::set_h) }
            (0xCB, 0x9D) => { self.shift_r8(&GB::get_l, &GB::res_3, &GB::set_l) }
            (0xCB, 0x9E) => { self.shift_mem(&GB::res_3) }
            (0xCB, 0x9F) => { self.shift_r8(&GB::get_a, &GB::res_3, &GB::set_a) }
            // RES 4
            (0xCB, 0xA0) => { self.shift_r8(&GB::get_b, &GB::res_4, &GB::set_b) }
            (0xCB, 0xA1) => { self.shift_r8(&GB::get_c, &GB::res_4, &GB::set_c) }
            (0xCB, 0xA2) => { self.shift_r8(&GB::get_d, &GB::res_4, &GB::set_d) }
            (0xCB, 0xA3) => { self.shift_r8(&GB::get_e, &GB::res_4, &GB::set_e) }
            (0xCB, 0xA4) => { self.shift_r8(&GB::get_h, &GB::res_4, &GB::set_h) }
            (0xCB, 0xA5) => { self.shift_r8(&GB::get_l, &GB::res_4, &GB::set_l) }
            (0xCB, 0xA6) => { self.shift_mem(&GB::res_4) }
            (0xCB, 0xA7) => { self.shift_r8(&GB::get_a, &GB::res_4, &GB::set_a) }
            // RES 5
            (0xCB, 0xA8) => { self.shift_r8(&GB::get_b, &GB::res_5, &GB::set_b) }
            (0xCB, 0xA9) => { self.shift_r8(&GB::get_c, &GB::res_5, &GB::set_c) }
            (0xCB, 0xAA) => { self.shift_r8(&GB::get_d, &GB::res_5, &GB::set_d) }
            (0xCB, 0xAB) => { self.shift_r8(&GB::get_e, &GB::res_5, &GB::set_e) }
            (0xCB, 0xAC) => { self.shift_r8(&GB::get_h, &GB::res_5, &GB::set_h) }
            (0xCB, 0xAD) => { self.shift_r8(&GB::get_l, &GB::res_5, &GB::set_l) }
            (0xCB, 0xAE) => { self.shift_mem(&GB::res_5) }
            (0xCB, 0xAF) => { self.shift_r8(&GB::get_a, &GB::res_5, &GB::set_a) }
            // RES 6
            (0xCB, 0xB0) => { self.shift_r8(&GB::get_b, &GB::res_6, &GB::set_b) }
            (0xCB, 0xB1) => { self.shift_r8(&GB::get_c, &GB::res_6, &GB::set_c) }
            (0xCB, 0xB2) => { self.shift_r8(&GB::get_d, &GB::res_6, &GB::set_d) }
            (0xCB, 0xB3) => { self.shift_r8(&GB::get_e, &GB::res_6, &GB::set_e) }
            (0xCB, 0xB4) => { self.shift_r8(&GB::get_h, &GB::res_6, &GB::set_h) }
            (0xCB, 0xB5) => { self.shift_r8(&GB::get_l, &GB::res_6, &GB::set_l) }
            (0xCB, 0xB6) => { self.shift_mem(&GB::res_6) }
            (0xCB, 0xB7) => { self.shift_r8(&GB::get_a, &GB::res_6, &GB::set_a) }
            // RES 7
            (0xCB, 0xB8) => { self.shift_r8(&GB::get_b, &GB::res_7, &GB::set_b) }
            (0xCB, 0xB9) => { self.shift_r8(&GB::get_c, &GB::res_7, &GB::set_c) }
            (0xCB, 0xBA) => { self.shift_r8(&GB::get_d, &GB::res_7, &GB::set_d) }
            (0xCB, 0xBB) => { self.shift_r8(&GB::get_e, &GB::res_7, &GB::set_e) }
            (0xCB, 0xBC) => { self.shift_r8(&GB::get_h, &GB::res_7, &GB::set_h) }
            (0xCB, 0xBD) => { self.shift_r8(&GB::get_l, &GB::res_7, &GB::set_l) }
            (0xCB, 0xBE) => { self.shift_mem(&GB::res_7) }
            (0xCB, 0xBF) => { self.shift_r8(&GB::get_a, &GB::res_7, &GB::set_a) }
            // SET 0
            (0xCB, 0xC0) => { self.shift_r8(&GB::get_b, &GB::set_0, &GB::set_b) }
            (0xCB, 0xC1) => { self.shift_r8(&GB::get_c, &GB::set_0, &GB::set_c) }
            (0xCB, 0xC2) => { self.shift_r8(&GB::get_d, &GB::set_0, &GB::set_d) }
            (0xCB, 0xC3) => { self.shift_r8(&GB::get_e, &GB::set_0, &GB::set_e) }
            (0xCB, 0xC4) => { self.shift_r8(&GB::get_h, &GB::set_0, &GB::set_h) }
            (0xCB, 0xC5) => { self.shift_r8(&GB::get_l, &GB::set_0, &GB::set_l) }
            (0xCB, 0xC6) => { self.shift_mem(&GB::set_0) }
            (0xCB, 0xC7) => { self.shift_r8(&GB::get_a, &GB::set_0, &GB::set_a) }
            // SET 1
            (0xCB, 0xC8) => { self.shift_r8(&GB::get_b, &GB::set_1, &GB::set_b) }
            (0xCB, 0xC9) => { self.shift_r8(&GB::get_c, &GB::set_1, &GB::set_c) }
            (0xCB, 0xCA) => { self.shift_r8(&GB::get_d, &GB::set_1, &GB::set_d) }
            (0xCB, 0xCB) => { self.shift_r8(&GB::get_e, &GB::set_1, &GB::set_e) }
            (0xCB, 0xCC) => { self.shift_r8(&GB::get_h, &GB::set_1, &GB::set_h) }
            (0xCB, 0xCD) => { self.shift_r8(&GB::get_l, &GB::set_1, &GB::set_l) }
            (0xCB, 0xCE) => { self.shift_mem(&GB::set_1) }
            (0xCB, 0xCF) => { self.shift_r8(&GB::get_a, &GB::set_1, &GB::set_a) }
            // SET 2
            (0xCB, 0xD0) => { self.shift_r8(&GB::get_b, &GB::set_2, &GB::set_b) }
            (0xCB, 0xD1) => { self.shift_r8(&GB::get_c, &GB::set_2, &GB::set_c) }
            (0xCB, 0xD2) => { self.shift_r8(&GB::get_d, &GB::set_2, &GB::set_d) }
            (0xCB, 0xD3) => { self.shift_r8(&GB::get_e, &GB::set_2, &GB::set_e) }
            (0xCB, 0xD4) => { self.shift_r8(&GB::get_h, &GB::set_2, &GB::set_h) }
            (0xCB, 0xD5) => { self.shift_r8(&GB::get_l, &GB::set_2, &GB::set_l) }
            (0xCB, 0xD6) => { self.shift_mem(&GB::set_2) }
            (0xCB, 0xD7) => { self.shift_r8(&GB::get_a, &GB::set_2, &GB::set_a) }
            // SET 3
            (0xCB, 0xD8) => { self.shift_r8(&GB::get_b, &GB::set_3, &GB::set_b) }
            (0xCB, 0xD9) => { self.shift_r8(&GB::get_c, &GB::set_3, &GB::set_c) }
            (0xCB, 0xDA) => { self.shift_r8(&GB::get_d, &GB::set_3, &GB::set_d) }
            (0xCB, 0xDB) => { self.shift_r8(&GB::get_e, &GB::set_3, &GB::set_e) }
            (0xCB, 0xDC) => { self.shift_r8(&GB::get_h, &GB::set_3, &GB::set_h) }
            (0xCB, 0xDD) => { self.shift_r8(&GB::get_l, &GB::set_3, &GB::set_l) }
            (0xCB, 0xDE) => { self.shift_mem(&GB::set_3) }
            (0xCB, 0xDF) => { self.shift_r8(&GB::get_a, &GB::set_3, &GB::set_a) }
            // SET 4
            (0xCB, 0xE0) => { self.shift_r8(&GB::get_b, &GB::set_4, &GB::set_b) }
            (0xCB, 0xE1) => { self.shift_r8(&GB::get_c, &GB::set_4, &GB::set_c) }
            (0xCB, 0xE2) => { self.shift_r8(&GB::get_d, &GB::set_4, &GB::set_d) }
            (0xCB, 0xE3) => { self.shift_r8(&GB::get_e, &GB::set_4, &GB::set_e) }
            (0xCB, 0xE4) => { self.shift_r8(&GB::get_h, &GB::set_4, &GB::set_h) }
            (0xCB, 0xE5) => { self.shift_r8(&GB::get_l, &GB::set_4, &GB::set_l) }
            (0xCB, 0xE6) => { self.shift_mem(&GB::set_4) }
            (0xCB, 0xE7) => { self.shift_r8(&GB::get_a, &GB::set_4, &GB::set_a) }
            // SET 5
            (0xCB, 0xE8) => { self.shift_r8(&GB::get_b, &GB::set_5, &GB::set_b) }
            (0xCB, 0xE9) => { self.shift_r8(&GB::get_c, &GB::set_5, &GB::set_c) }
            (0xCB, 0xEA) => { self.shift_r8(&GB::get_d, &GB::set_5, &GB::set_d) }
            (0xCB, 0xEB) => { self.shift_r8(&GB::get_e, &GB::set_5, &GB::set_e) }
            (0xCB, 0xEC) => { self.shift_r8(&GB::get_h, &GB::set_5, &GB::set_h) }
            (0xCB, 0xED) => { self.shift_r8(&GB::get_l, &GB::set_5, &GB::set_l) }
            (0xCB, 0xEE) => { self.shift_mem(&GB::set_5) }
            (0xCB, 0xEF) => { self.shift_r8(&GB::get_a, &GB::set_5, &GB::set_a) }
            // SET 6
            (0xCB, 0xF0) => { self.shift_r8(&GB::get_b, &GB::set_6, &GB::set_b) }
            (0xCB, 0xF1) => { self.shift_r8(&GB::get_c, &GB::set_6, &GB::set_c) }
            (0xCB, 0xF2) => { self.shift_r8(&GB::get_d, &GB::set_6, &GB::set_d) }
            (0xCB, 0xF3) => { self.shift_r8(&GB::get_e, &GB::set_6, &GB::set_e) }
            (0xCB, 0xF4) => { self.shift_r8(&GB::get_h, &GB::set_6, &GB::set_h) }
            (0xCB, 0xF5) => { self.shift_r8(&GB::get_l, &GB::set_6, &GB::set_l) }
            (0xCB, 0xF6) => { self.shift_mem(&GB::set_6) }
            (0xCB, 0xF7) => { self.shift_r8(&GB::get_a, &GB::set_6, &GB::set_a) }
            // SET 7
            (0xCB, 0xF8) => { self.shift_r8(&GB::get_b, &GB::set_7, &GB::set_b) }
            (0xCB, 0xF9) => { self.shift_r8(&GB::get_c, &GB::set_7, &GB::set_c) }
            (0xCB, 0xFA) => { self.shift_r8(&GB::get_d, &GB::set_7, &GB::set_d) }
            (0xCB, 0xFB) => { self.shift_r8(&GB::get_e, &GB::set_7, &GB::set_e) }
            (0xCB, 0xFC) => { self.shift_r8(&GB::get_h, &GB::set_7, &GB::set_h) }
            (0xCB, 0xFD) => { self.shift_r8(&GB::get_l, &GB::set_7, &GB::set_l) }
            (0xCB, 0xFE) => { self.shift_mem(&GB::set_7) }
            (0xCB, 0xFF) => { self.shift_r8(&GB::get_a, &GB::set_7, &GB::set_a) }

            // NOP
            (0x00, _) => { return 4; }

            // LD r16, d16
            (0x01, _) => { self.ld_bc_d16() }
            (0x11, _) => { self.ld_de_d16() }
            (0x21, _) => { self.ld_hl_d16() }
            (0x31, _) => { self.ld_sp_d16() }
            // LD (r16), A
            (0x02, _) => { self.ld_r16_mem(self.bc) }
            (0x12, _) => { self.ld_r16_mem(self.de) }
            (0x22, _) => { self.ld_hl_mem_inc() }
            (0x32, _) => { self.ld_hl_mem_dec() }
            // LD B, r8
            (0x40, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_b) }
            (0x41, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_c) }
            (0x42, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_c) }
            (0x43, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_e) }
            (0x44, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_h) }
            (0x45, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_l) }
            (0x46, _) => { self.ld_r8_mem_r16(&GB::set_b, self.hl) }
            (0x47, _) => { self.ld_r8_r8(&GB::set_b, &GB::get_a) }
            // LD C, r8
            (0x48, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_b) }
            (0x49, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_c) }
            (0x4A, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_c) }
            (0x4B, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_e) }
            (0x4C, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_h) }
            (0x4D, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_l) }
            (0x4E, _) => { self.ld_r8_mem_r16(&GB::set_c, self.hl) }
            (0x4F, _) => { self.ld_r8_r8(&GB::set_c, &GB::get_a) }
            // LD D, r8
            (0x50, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_b) }
            (0x51, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_c) }
            (0x52, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_c) }
            (0x53, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_e) }
            (0x54, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_h) }
            (0x55, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_l) }
            (0x56, _) => { self.ld_r8_mem_r16(&GB::set_d, self.hl) }
            (0x57, _) => { self.ld_r8_r8(&GB::set_d, &GB::get_a) }
            // LD E, r8
            (0x58, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_b) }
            (0x59, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_c) }
            (0x5A, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_c) }
            (0x5B, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_e) }
            (0x5C, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_h) }
            (0x5D, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_l) }
            (0x5E, _) => { self.ld_r8_mem_r16(&GB::set_e, self.hl) }
            (0x5F, _) => { self.ld_r8_r8(&GB::set_e, &GB::get_a) }
            // LD H, r8
            (0x60, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_b) }
            (0x61, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_c) }
            (0x62, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_c) }
            (0x63, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_e) }
            (0x64, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_h) }
            (0x65, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_l) }
            (0x66, _) => { self.ld_r8_mem_r16(&GB::set_h, self.hl) }
            (0x67, _) => { self.ld_r8_r8(&GB::set_h, &GB::get_a) }
            // LD L, r8
            (0x68, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_b) }
            (0x69, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_c) }
            (0x6A, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_c) }
            (0x6B, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_e) }
            (0x6C, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_h) }
            (0x6D, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_l) }
            (0x6F, _) => { self.ld_r8_r8(&GB::set_l, &GB::get_a) }
            // LD (HL), r8
            (0x70, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_b) }
            (0x71, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_c) }
            (0x72, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_d) }
            (0x73, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_e) }
            (0x74, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_h) }
            (0x75, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_l) }
            (0x76, _) => { panic!("Not implemented yet!") } // HALT
            (0x77, _) => { self.ld_mem_r16_r8(self.hl, &GB::get_a) }
            // LD A, r8
            (0x78, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_b) }
            (0x79, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_c) }
            (0x7A, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_c) }
            (0x7B, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_e) }
            (0x7C, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_h) }
            (0x7D, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_l) }
            (0x66, _) => { self.ld_r8_mem_r16(&GB::set_a, self.hl) }
            (0x7F, _) => { self.ld_r8_r8(&GB::set_a, &GB::get_a) }

            // Arithmetic ops
            // ADD
            (0x80, _) => { let val = self.get_b(); self.add_r8(val) }
            (0x81, _) => { let val = self.get_c(); self.add_r8(val) }
            (0x82, _) => { let val = self.get_d(); self.add_r8(val) }
            (0x83, _) => { let val = self.get_e(); self.add_r8(val) }
            (0x84, _) => { let val = self.get_h(); self.add_r8(val) }
            (0x85, _) => { let val = self.get_l(); self.add_r8(val) }
            (0x86, _) => { let val = self.mem_read(self.hl); self.add_r8(val) }
            (0x87, _) => { let val = self.get_a(); self.add_r8(val) }
            // ADC
            (0x88, _) => { let val = self.get_b(); self.adc_r8(val) }
            (0x89, _) => { let val = self.get_c(); self.adc_r8(val) }
            (0x8A, _) => { let val = self.get_d(); self.adc_r8(val) }
            (0x8B, _) => { let val = self.get_e(); self.adc_r8(val) }
            (0x8C, _) => { let val = self.get_h(); self.adc_r8(val) }
            (0x8D, _) => { let val = self.get_l(); self.adc_r8(val) }
            (0x8E, _) => { let val = self.mem_read(self.hl); self.adc_r8(val) }
            (0x8F, _) => { let val = self.get_a(); self.adc_r8(val) }
            // SUB
            (0x90, _) => { let val = self.get_b(); self.sub_r8(val) }
            (0x91, _) => { let val = self.get_c(); self.sub_r8(val) }
            (0x92, _) => { let val = self.get_d(); self.sub_r8(val) }
            (0x93, _) => { let val = self.get_e(); self.sub_r8(val) }
            (0x94, _) => { let val = self.get_h(); self.sub_r8(val) }
            (0x95, _) => { let val = self.get_l(); self.sub_r8(val) }
            (0x96, _) => { let val = self.mem_read(self.hl); self.sub_r8(val) }
            (0x97, _) => { let val = self.get_a(); self.sub_r8(val) }
            // SBC
            (0x98, _) => { let val = self.get_b(); self.sbc_r8(val) }
            (0x99, _) => { let val = self.get_c(); self.sbc_r8(val) }
            (0x9A, _) => { let val = self.get_d(); self.sbc_r8(val) }
            (0x9B, _) => { let val = self.get_e(); self.sbc_r8(val) }
            (0x9C, _) => { let val = self.get_h(); self.sbc_r8(val) }
            (0x9D, _) => { let val = self.get_l(); self.sbc_r8(val) }
            (0x9E, _) => { let val = self.mem_read(self.hl); self.sbc_r8(val) }
            (0x9F, _) => { let val = self.get_a(); self.sbc_r8(val) }
            // AND
            (0xA0, _) => { let val = self.get_b(); self.and_r8(val) }
            (0xA1, _) => { let val = self.get_c(); self.and_r8(val) }
            (0xA2, _) => { let val = self.get_d(); self.and_r8(val) }
            (0xA3, _) => { let val = self.get_e(); self.and_r8(val) }
            (0xA4, _) => { let val = self.get_h(); self.and_r8(val) }
            (0xA5, _) => { let val = self.get_l(); self.and_r8(val) }
            (0xA6, _) => { let val = self.mem_read(self.hl); self.and_r8(val) }
            (0xA7, _) => { let val = self.get_a(); self.and_r8(val) }
            // XOR
            (0xA8, _) => { let val = self.get_b(); self.xor_r8(val) }
            (0xA9, _) => { let val = self.get_c(); self.xor_r8(val) }
            (0xAA, _) => { let val = self.get_d(); self.xor_r8(val) }
            (0xAB, _) => { let val = self.get_e(); self.xor_r8(val) }
            (0xAC, _) => { let val = self.get_h(); self.xor_r8(val) }
            (0xAD, _) => { let val = self.get_l(); self.xor_r8(val) }
            (0xAE, _) => { let val = self.mem_read(self.hl); self.xor_r8(val) }
            (0xAF, _) => { let val = self.get_a(); self.xor_r8(val) }
            // OR
            (0xB0, _) => { let val = self.get_b(); self.or_r8(val) }
            (0xB1, _) => { let val = self.get_c(); self.or_r8(val) }
            (0xB2, _) => { let val = self.get_d(); self.or_r8(val) }
            (0xB3, _) => { let val = self.get_e(); self.or_r8(val) }
            (0xB4, _) => { let val = self.get_h(); self.or_r8(val) }
            (0xB5, _) => { let val = self.get_l(); self.or_r8(val) }
            (0xB6, _) => { let val = self.mem_read(self.hl); self.or_r8(val) }
            (0xB7, _) => { let val = self.get_a(); self.or_r8(val) }
            // CP
            (0xB8, _) => { let val = self.get_b(); self.cp_r8(val) }
            (0xB9, _) => { let val = self.get_c(); self.cp_r8(val) }
            (0xBA, _) => { let val = self.get_d(); self.cp_r8(val) }
            (0xBB, _) => { let val = self.get_e(); self.cp_r8(val) }
            (0xBC, _) => { let val = self.get_h(); self.cp_r8(val) }
            (0xBD, _) => { let val = self.get_l(); self.cp_r8(val) }
            (0xBE, _) => { let val = self.mem_read(self.hl); self.cp_r8(val) }
            (0xBF, _) => { let val = self.get_a(); self.cp_r8(val) }


            // Stuff gets weird here
            // Assorted LD r8, d8
            (0x06, val) => { self.ld_r8_d8(&GB::set_b, val) }
            (0x16, val) => { self.ld_r8_d8(&GB::set_d, val) }
            (0x26, val) => { self.ld_r8_d8(&GB::set_h, val) }
            (0x36, val) => { self.ld_mem_r16_d8(self.hl, val) }
            (0x0E, val) => { self.ld_r8_d8(&GB::set_c, val) }
            (0x1E, val) => { self.ld_r8_d8(&GB::set_e, val) }
            (0x2E, val) => { self.ld_r8_d8(&GB::set_l, val) }
            (0x3E, val) => { self.ld_r8_d8(&GB::set_a, val) }
            // LD A, (r16)
            (0x0A, _) => { self.ld_r8_mem_r16(&GB::set_a, self.bc) }
            (0x1A, _) => { self.ld_r8_mem_r16(&GB::set_a, self.de) }
            (0x2A, _) => { self.ld_a_mem_hl_inc() }
            (0x3A, _) => { self.ld_a_mem_hl_dec() }
            // LDH
            (0xE0, val) => { self.ldh_mem_a8_r8(val, &GB::get_a) }
            (0xF0, val) => { self.ldh_r8_mem_a8(&GB::set_a, val) }
            // LD C (Like LDH for hi mem)
            (0xE2, val) => { self.ld_mem_r8_r8(&GB::get_c, &GB::get_a) }
            (0xF2, val) => { self.ld_r8_mem_r8(&GB::set_a, &GB::get_c) }
            // LD SP/HL
            (0xF8, val) => { self.ld_hl_sp_plus_a8(val) }
            (0xF9, val) => { self.ld_sp_hl() }
            // Arithmetic d8
            (0xC6, val) => { self.add_r8(val) }
            (0xD6, val) => { self.sub_r8(val) }
            (0xE6, val) => { self.and_r8(val) }
            (0xF6, val) => { self.or_r8(val) }
            (0xCE, val) => { self.adc_r8(val) }
            (0xDE, val) => { self.sbc_r8(val) }
            (0xEE, val) => { self.xor_r8(val) }
            (0xFE, val) => { self.cp_r8(val) }
            // ADD HL, r16
            (0x09, _) => { self.add_hl_bc() }
            (0x19, _) => { self.add_hl_de() }
            (0x29, _) => { self.add_hl_hl() }
            (0x39, _) => { self.add_hl_sp() }

            // DEC and INC r16
            (0x03, _) => { self.inc_bc() }
            (0x13, _) => { self.inc_de() }
            (0x23, _) => { self.inc_hl() }
            (0x33, _) => { self.inc_sp() }
            (0x0B, _) => { self.dec_bc() }
            (0x1B, _) => { self.dec_de() }
            (0x2B, _) => { self.dec_hl() }
            (0x3B, _) => { self.dec_sp() }
            // DEC r8
            (0x05, _) => { self.dec_r8(&GB::set_b, &GB::get_b) }
            (0x15, _) => { self.dec_r8(&GB::set_d, &GB::get_d) }
            (0x25, _) => { self.dec_r8(&GB::set_h, &GB::get_h) }
            (0x35, _) => { self.dec_r8_mem() }
            (0x0D, _) => { self.dec_r8(&GB::set_c, &GB::get_c) }
            (0x1D, _) => { self.dec_r8(&GB::set_e, &GB::get_e) }
            (0x2D, _) => { self.dec_r8(&GB::set_l, &GB::get_l) }
            (0x3D, _) => { self.dec_r8(&GB::set_a, &GB::get_a) }
            // INC r8
            (0x04, _) => { self.inc_r8(&GB::set_b, &GB::get_b) }
            (0x14, _) => { self.inc_r8(&GB::set_d, &GB::get_d) }
            (0x24, _) => { self.inc_r8(&GB::set_h, &GB::get_h) }
            (0x34, _) => { self.inc_r8_mem() }
            (0x0C, _) => { self.inc_r8(&GB::set_c, &GB::get_c) }
            (0x1C, _) => { self.inc_r8(&GB::set_e, &GB::get_e) }
            (0x2C, _) => { self.inc_r8(&GB::set_l, &GB::get_l) }
            (0x3C, _) => { self.inc_r8(&GB::set_a, &GB::get_a) }
            // Interrupts
            (0xF3, _) => { self.di() }
            (0xFB, _) => { self.ei() }
            // JP a16
            (0xC3, _) => { self.jp_a16() }
            (0xE9, _) => { self.jp_hl() }
            // JP CC, a16
            (0xC2, _) => { self.jp_nz() }
            (0xD2, _) => { self.jp_nc() }
            (0xCA, _) => { self.jp_z() }
            (0xDA, _) => { self.jp_c() }
            // JR a8
            (0x18, val) => { self.jr_a8(val as i8) }
            // JR cc, a8
            (0x18, val) => { self.jr_nz_a8(val as i8) }
            (0x20, val) => { self.jr_nc_a8(val as i8) }
            (0x28, val) => { self.jr_z_a8(val as i8) }
            (0x38, val) => { self.jr_c_a8(val as i8) }
            // CALL a16
            (0xCD, _) => { self.call_a16() }
            // CALL cc, a16
            (0xC4, _) => { self.call_nz_a16() }
            (0xD4, _) => { self.call_nc_a16() }
            (0xCC, _) => { self.call_z_a16() }
            (0xDC, _) => { self.call_c_a16() }
            // RET a16
            (0xC9, _) => { self.ret_a16() }
            // RET cc, a16
            (0xC0, _) => { self.ret_nz_a16() }
            (0xD0, _) => { self.ret_nc_a16() }
            (0xC8, _) => { self.ret_z_a16() }
            (0xC8, _) => { self.ret_c_a16() }
            // RETI a16
            (0xD9, _) => { self.reti_a16() }
            // PUSH r16
            (0xC5, _) => { self.push_r16(self.bc) }
            (0xD5, _) => { self.push_r16(self.de) }
            (0xE5, _) => { self.push_r16(self.hl) }
            (0xF5, _) => { self.push_r16(self.af) }
            // RST n
            (0xC7, _) => { self.rst_n8(0x00) }
            (0xD7, _) => { self.rst_n8(0x10) }
            (0xE7, _) => { self.rst_n8(0x20) }
            (0xF7, _) => { self.rst_n8(0x30) }
            (0xCF, _) => { self.rst_n8(0x08) }
            (0xDF, _) => { self.rst_n8(0x18) }
            (0xEF, _) => { self.rst_n8(0x28) }
            (0xFF, _) => { self.rst_n8(0x38) }
            // POP r16
            (0xC1, _) => { self.pop_bc() }
            (0xD1, _) => { self.pop_de() }
            (0xE1, _) => { self.pop_hl() }
            (0xF1, _) => { self.pop_af() }
            // Shift A stuff
            (0x07, _) => { self.shift_r8(&GB::get_b, &GB::rlc, &GB::set_b) }
            (0x0F, _) => { self.shift_r8(&GB::get_b, &GB::rrc, &GB::set_b) }
            (0x17, _) => { self.shift_r8(&GB::get_b, &GB::rl, &GB::set_b) }
            (0x1F, _) => { self.shift_r8(&GB::get_b, &GB::rr, &GB::set_b) }
            (0xF1, _) => { self.pop_af() }
            // Random stuff
            (0x27, _) => { self.daa() }
            (0x37, _) => { self.scf() }
            (0x2F, _) => { self.cpl() }
            (0x3F, _) => { self.ccf() }
            (0x27, _) => { self.daa() }



            (_, _)  => { panic!("Unknown opcode") }
        }
    }

    pub fn shift_r8(&mut self,
                    getter: &Fn(&mut GB) -> u8,
                    f: &Fn(&mut GB, u8) -> u8,
                    setter: &Fn(&mut GB, u8)) -> u32 {
        let mut r = getter(self);
        r = f(self, r);
        setter(self, r);
        return 8;
    }
    pub fn shift_mem(&mut self, f: &Fn(&mut GB, u8) -> u8) -> u32 {
        let addr = self.get_hl();
        let mut r = self.mem_read(addr);
        r = f(self, r);
        self.mem_write(addr, r);
        return 16;
    }

    // Shifting functions
    fn rlc(&mut self, mut r: u8) -> u8 {
        let cy = r >> 7;
        r = (r << 1) | cy;
        self.set_cy(cy);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }
    fn rrc(mut gb: &mut GB, mut r: u8) -> u8 {
        let cy = r & 1;
        r = (r >> 1) | (cy << 7);
        gb.set_cy(cy);
        if r == 0 {
            gb.set_z(1);
        }
        return r;
    }
    fn rl(&mut self, mut r: u8) -> u8 {
        let cy = r >> 7;
        r = (r << 1) | self.get_cy();
        self.set_cy(cy);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }
    fn rr(&mut self, mut r: u8) -> u8 {
        let cy = r & 1;
        r = (r >> 1) | (self.get_cy() << 7);
        self.set_cy(cy);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }
    fn sla(&mut self, mut r: u8) -> u8 {
        let cy = r >> 7;
        r = r << 1;
        self.set_cy(cy);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }
    fn sra(&mut self, mut r: u8) -> u8 {
        let cy = r & 1;
        self.set_cy(cy);
        let sign = r >> 7;
        r = (r >> 1) | (sign << 7);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }
    fn swap(&mut self, mut r: u8) -> u8 {
        let sign = r >> 7;
        r = (r >> 4) | (r << 4);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }
    fn srl(&mut self, mut r: u8) -> u8 {
        let cy = r & 1;
        r = r >> 1;
        self.set_cy(cy);
        if r == 0 {
            self.set_z(1);
        }
        return r;
    }


    //BIT testing/setting functions
    fn bit_0(&mut self, r: u8) -> u8 { return self.bit(r, 0); }
    fn bit_1(&mut self, r: u8) -> u8 { return self.bit(r, 1); }
    fn bit_2(&mut self, r: u8) -> u8 { return self.bit(r, 2); }
    fn bit_3(&mut self, r: u8) -> u8 { return self.bit(r, 3); }
    fn bit_4(&mut self, r: u8) -> u8 { return self.bit(r, 4); }
    fn bit_5(&mut self, r: u8) -> u8 { return self.bit(r, 5); }
    fn bit_6(&mut self, r: u8) -> u8 { return self.bit(r, 6); }
    fn bit_7(&mut self, r: u8) -> u8 { return self.bit(r, 7); }
    fn bit(&mut self, r: u8, i: u8) -> u8 {
        self.set_z((r >> i) & 1);
        return r;
    }
    fn res_0(&mut self, r: u8) -> u8 { return self.res(r, 0); }
    fn res_1(&mut self, r: u8) -> u8 { return self.res(r, 1); }
    fn res_2(&mut self, r: u8) -> u8 { return self.res(r, 2); }
    fn res_3(&mut self, r: u8) -> u8 { return self.res(r, 3); }
    fn res_4(&mut self, r: u8) -> u8 { return self.res(r, 4); }
    fn res_5(&mut self, r: u8) -> u8 { return self.res(r, 5); }
    fn res_6(&mut self, r: u8) -> u8 { return self.res(r, 6); }
    fn res_7(&mut self, r: u8) -> u8 { return self.res(r, 7); }
    fn res(&mut self, r: u8, i: u8) -> u8 {
        return r & !(1 << i);
    }
    fn set_0(&mut self, r: u8) -> u8 { return self.set(r, 0); }
    fn set_1(&mut self, r: u8) -> u8 { return self.set(r, 1); }
    fn set_2(&mut self, r: u8) -> u8 { return self.set(r, 2); }
    fn set_3(&mut self, r: u8) -> u8 { return self.set(r, 3); }
    fn set_4(&mut self, r: u8) -> u8 { return self.set(r, 4); }
    fn set_5(&mut self, r: u8) -> u8 { return self.set(r, 5); }
    fn set_6(&mut self, r: u8) -> u8 { return self.set(r, 6); }
    fn set_7(&mut self, r: u8) -> u8 { return self.set(r, 7); }
    fn set(&mut self, r: u8, i: u8) -> u8 {
        return r | (1 << i);
    }


    //LD ops
    fn ld_bc_d16(&mut self) -> u32 {
        let d16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        self.bc = d16;
        return 12;
    }
    fn ld_de_d16(&mut self) -> u32 {
        let d16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        self.de = d16;
        return 12;
    }
    fn ld_hl_d16(&mut self) -> u32 {
        let d16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        self.hl = d16;
        return 12;
    }
    fn ld_sp_d16(&mut self) -> u32 {
        let d16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        self.sp = d16;
        return 12;
    }
    fn ld_r16_mem(&mut self, addr: u16) -> u32 {
        let a = self.get_a();
        self.mem_write(addr, a);
        return 8;
    }
    fn ld_hl_mem_inc(&mut self) -> u32 {
        let hl = self.hl;
        let a = self.get_a();
        self.mem_write(hl, a);
        self.hl += 1;
        return 8;
    }
    fn ld_hl_mem_dec(&mut self) -> u32 {
        let hl = self.hl;
        let a = self.get_a();
        self.mem_write(hl, a);
        self.hl -= 1;
        return 8;
    }
    fn ld_r8_r8(&mut self, setter: &Fn(&mut GB, u8), getter: &Fn(&mut GB) -> u8) -> u32 {
        let val = getter(self);
        setter(self, val);
        return 4;
    }
    fn ld_mem_r16_r8(&mut self, dest_addr: u16, src_getter: &Fn(&mut GB) -> u8) -> u32 {
        let val = src_getter(self);
        self.mem_write(dest_addr, val);
        return 8;
    }
    fn ld_r8_mem_r16(&mut self, dest_setter: &Fn(&mut GB, u8), src_addr: u16) -> u32 {
        let val = self.mem_read(src_addr);
        dest_setter(self, val);
        return 8;
    }
    fn ld_r8_d8(&mut self, setter: &Fn(&mut GB, u8), val: u8) -> u32 {
        setter(self, val);
        return 8;
    }
    fn ld_mem_r16_d8(&mut self, dest_addr: u16, val: u8) -> u32 {
        self.mem_write(dest_addr, val);
        return 8;
    }
    fn ld_mem_a16_r16(&mut self, dest_addr: u16, val: u16) -> u32 {
        self.mem_write(dest_addr, (val & 0xFF) as u8);
        self.mem_write(dest_addr+1, ((val >> 8) & 0xFF) as u8);
        return 8;
    }
    fn ld_a_mem_hl_inc(&mut self) -> u32 {
        let val = self.mem_read(self.hl);
        self.set_a(val);
        self.hl += 1;
        return 8;
    }
    fn ld_a_mem_hl_dec(&mut self) -> u32 {
        let val = self.mem_read(self.hl);
        self.set_a(val);
        self.hl -= 1;
        return 8;
    }
    fn ldh_mem_a8_r8(&mut self, dest_addr: u8, getter: &Fn(&mut GB) -> u8) -> u32 {
        let val = getter(self);
        let dest = (dest_addr as u16)| 0xFF00;
        self.mem_write(dest, val);
        return 12;
    }
    fn ldh_r8_mem_a8(&mut self, setter: &Fn(&mut GB, u8), src_addr: u8) -> u32 {
        let src = (src_addr as u16)| 0xFF00;
        let val = self.mem_read(src);
        setter(self, val);
        return 12;
    }
    fn ld_mem_r8_r8(&mut self, dest_getter: &Fn(&mut GB) -> u8, src_getter: &Fn(&mut GB) -> u8) -> u32 {
        let val = src_getter(self);
        let mut dest = dest_getter(self) as u16;
        dest = dest| 0xFF00;
        self.mem_write(dest, val);
        return 8;
    }
    fn ld_r8_mem_r8(&mut self, dest_setter: &Fn(&mut GB, u8), src_getter: &Fn(&mut GB) -> u8) -> u32 {
        let mut src = src_getter(self) as u16;
        src = src | 0xFF00;
        let val = self.mem_read(src);
        dest_setter(self, val);
        return 8;
    }
    fn ld_hl_sp_plus_a8(&mut self, val: u8) -> u32 {
        let (result, _) = self.sp.overflowing_add(val as u16);
        self.hl = result;

        // Calculate C
        let (_, c) = self.sp.overflowing_add(val as u16);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // Calculate H
        let (_, h) = (self.sp as u8).overflowing_add(val);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // Set remaining bits
        self.set_z(0);
        self.set_n(0);
        return 12;
    }
    fn ld_sp_hl(&mut self) -> u32 {
        self.sp = self.hl;
        return 8;
    }
    fn ld_mem_a16_a(&mut self) -> u32 {
        let a16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        let val = self.get_a();
        self.mem_write(a16, val);
        return 16;
    }
    fn ld_a_mem_a16(&mut self) -> u32 {
        let a16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        let val = self.mem_read(a16);
        self.set_a(val);
        return 16;
    }

    // Arithmetic
    fn adc_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let (mut result, _) = a.overflowing_add(val);
        let (result, _) = result.overflowing_add(self.get_cy());
        self.set_a(result);

        // Calculate C
        let (v1, c1) = a.overflowing_add(val);
        let (_, c2) = v1.overflowing_add(self.get_cy());
        if c1 || c2{
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // Calculate H
        let (v1, h1) = (a << 4).overflowing_add(val << 4);
        let (_, h2) = v1.overflowing_add(self.get_cy() << 4);
        if h1 || h2 {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        return 4;
    }
    fn add_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let (result, _) = a.overflowing_add(val);
        self.set_a(result);

        // Calculate C
        let (_, c) = a.overflowing_add(val);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // Calculate H
        let (_, h) = (a << 4).overflowing_add(val << 4);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        return 4;
    }
    fn and_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let result = a & val;
        self.set_a(result);

        // Set C
        self.set_cy(0);

        // Set H
        self.set_hc(0);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        return 4;
    }
    fn cp_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let (mut result, _) = a.overflowing_sub(val);
        let (result, _) = result.overflowing_sub(self.get_cy());

        // Calculate C
        let (v1, c1) = a.overflowing_sub(val);
        let (_, c2) = v1.overflowing_sub(self.get_cy());
        if c1 || c2{
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // Calculate H
        let (v1, h1) = (a << 4).overflowing_sub(val << 4);
        let (_, h2) = v1.overflowing_sub(self.get_cy() << 4);
        if h1 || h2 {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn or_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let result = a | val;
        self.set_a(result);

        // Set C
        self.set_cy(0);

        // Set H
        self.set_hc(0);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);
        return 4;
    }
    fn sbc_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let (mut result, _) = a.overflowing_sub(val);
        let (result, _) = result.overflowing_sub(self.get_cy());
        self.set_a(result);

        // Calculate C
        let (v1, c1) = a.overflowing_sub(val);
        let (_, c2) = v1.overflowing_sub(self.get_cy());
        if c1 || c2{
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // Calculate H
        let (v1, h1) = (a << 4).overflowing_sub(val << 4);
        let (_, h2) = v1.overflowing_sub(self.get_cy() << 4);
        if h1 || h2 {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn sub_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let (result, _) = a.overflowing_sub(val);
        self.set_a(result);

        // Calculate C
        let (_, c) = a.overflowing_sub(val);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // Calculate H
        let (_, h) = (a << 4).overflowing_sub(val << 4);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn xor_r8(&mut self, val: u8) -> u32 {
        let a = self.get_a();
        let result = a ^ val;
        self.set_a(result);

        // Set C
        self.set_cy(0);

        // Set H
        self.set_hc(0);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);
        return 4;
    }
    fn add_hl_bc(&mut self) -> u32 {
        let (result, _) = self.hl.overflowing_add(self.bc);

        // Calculate C
        let (_, c) = self.hl.overflowing_add(self.bc);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // // Calculate H
        let (_, h) = (self.hl << 4).overflowing_add(self.bc << 4);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        self.hl = result;

        return 8;
    }
    fn add_hl_de(&mut self) -> u32 {
        let (result, _) = self.hl.overflowing_add(self.de);

        // Calculate C
        let (_, c) = self.hl.overflowing_add(self.de);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // // Calculate H
        let (_, h) = (self.hl << 4).overflowing_add(self.de << 4);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        self.hl = result;
        return 8;
    }
    fn add_hl_hl(&mut self) -> u32 {
        let (result, _) = self.hl.overflowing_add(self.hl);

        // Calculate C
        let (_, c) = self.hl.overflowing_add(self.hl);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // // Calculate H
        let (_, h) = (self.hl << 4).overflowing_add(self.hl << 4);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        self.hl = result;
        return 8;
    }
    fn add_hl_sp(&mut self) -> u32 {
        let (result, _) = self.hl.overflowing_add(self.sp);

        // Calculate C
        let (_, c) = self.hl.overflowing_add(self.sp);
        if c {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }

        // // Calculate H
        let (_, h) = (self.hl << 4).overflowing_add(self.sp << 4);
        if h {
            self.set_hc(1);
        } else {
            self.set_hc(0);
        }

        // // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Set N
        self.set_n(0);

        self.hl = result;
        return 8;
    }
    fn dec_r8(&mut self, setter: &Fn(&mut GB, u8), getter: &Fn(&mut GB) -> u8) -> u32 {
        let mut val = getter(self);
        let (result, _) = val.overflowing_sub(1);
        setter(self, result);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Calculate H
        if (val << 4) == 0 {
            self.set_hc(0);
        } else {
            self.set_hc(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn dec_r8_mem(&mut self) -> u32 {
        let mut val = self.mem_read(self.hl);
        let (result, _) = val.overflowing_sub(1);
        self.mem_write(self.hl, result);
        println!("HL: {:#X}", self.hl);
        println!("Val: {:#X}", val);
        println!("Result: {:#X}", result);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Calculate H
        if (val << 4) == 0 {
            self.set_hc(0);
        } else {
            self.set_hc(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn dec_bc(&mut self) -> u32 {
        let (result, _) = self.bc.overflowing_sub(1);
        self.bc = result;
        return 8;
    }
    fn dec_de(&mut self) -> u32 {
        let (result, _) = self.de.overflowing_sub(1);
        self.de = result;
        return 8;
    }
    fn dec_hl(&mut self) -> u32 {
        let (result, _) = self.hl.overflowing_sub(1);
        self.hl = result;
        return 8;
    }
    fn dec_sp(&mut self) -> u32 {
        let (result, _) = self.sp.overflowing_sub(1);
        self.sp = result;
        return 8;
    }
    fn inc_r8(&mut self, setter: &Fn(&mut GB, u8), getter: &Fn(&mut GB) -> u8) -> u32 {
        let mut val = getter(self);
        let (result, _) = val.overflowing_add(1);
        setter(self, result);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Calculate H
        if (val << 4) == 0 {
            self.set_hc(0);
        } else {
            self.set_hc(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn inc_r8_mem(&mut self) -> u32 {
        let mut val = self.mem_read(self.hl);
        let (result, _) = val.overflowing_add(1);
        self.mem_write(self.hl, result);
        println!("HL: {:#X}", self.hl);
        println!("Val: {:#X}", val);
        println!("Result: {:#X}", result);

        // Calculate Z
        if result == 0 {
            self.set_z(0);
        } else {
            self.set_z(1);
        }

        // Calculate H
        if (val << 4) == 0 {
            self.set_hc(0);
        } else {
            self.set_hc(1);
        }

        // Set N
        self.set_n(1);

        return 4;
    }
    fn inc_bc(&mut self) -> u32 {
        let (result, _) = self.bc.overflowing_add(1);
        self.bc = result;
        return 8;
    }
    fn inc_de(&mut self) -> u32 {
        let (result, _) = self.de.overflowing_add(1);
        self.de = result;
        return 8;
    }
    fn inc_hl(&mut self) -> u32 {
        let (result, _) = self.hl.overflowing_add(1);
        self.hl = result;
        return 8;
    }
    fn inc_sp(&mut self) -> u32 {
        let (result, _) = self.sp.overflowing_add(1);
        self.sp = result;
        return 8;
    }
    fn ei(&mut self) -> u32 {
        self.ime = 1;
        return 4;
    }
    fn di(&mut self) -> u32 {
        self.ime = 0;
        return 4;
    }
    fn jp_a16(&mut self) -> u32 {
        let a16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        self.pc = a16;
        return 16;
    }
    fn jp_hl(&mut self) -> u32 {
        self.pc = self.hl;
        return 16;
    }
    fn jp_nz(&mut self) -> u32 {
        if self.get_z() == 0 {
            return self.jp_a16();
        }
        return 12;
    }
    fn jp_nc(&mut self) -> u32 {
        if self.get_cy() == 0 {
            return self.jp_a16();
        }
        return 12;
    }
    fn jp_z(&mut self) -> u32 {
        if self.get_z() == 1 {
            return self.jp_a16();
        }
        return 12;
    }
    fn jp_c(&mut self) -> u32 {
        if self.get_cy() == 1 {
            return self.jp_a16();
        }
        return 12;
    }
    fn jr_a8(&mut self, val: i8) -> u32 {
        if val >= 0 {
            let (result, _) = self.pc.overflowing_add(val as u16);
            self.pc = result;
        } else {
            let (result, _) = self.pc.overflowing_sub((val & 0x7F) as u16);
            self.pc = result;
        }
        return 16;
    }
    fn jr_nz_a8(&mut self, val: i8) -> u32 {
        if self.get_z() == 0 {
            return self.jr_a8(val);
        }
        return 12;
    }
    fn jr_nc_a8(&mut self, val: i8) -> u32 {
        if self.get_cy() == 0 {
            return self.jr_a8(val);
        }
        return 12;
    }
    fn jr_z_a8(&mut self, val: i8) -> u32 {
        if self.get_z() == 1 {
            return self.jr_a8(val);
        }
        return 12;
    }
    fn jr_c_a8(&mut self, val: i8) -> u32 {
        if self.get_cy() == 1 {
            return self.jr_a8(val);
        }
        return 12;
    }
    fn call_a16(&mut self) -> u32 {
        let a16 = ((self.mem_read(self.pc + 1) as u16) << 8) | (self.mem_read(self.pc + 2) as u16);
        self.mem_write(self.sp, (self.pc >> 8) as u8);
        self.mem_write(self.sp-1, self.pc as u8);
        self.sp = self.sp - 2;
        self.pc = a16;

        return 24;
    }
    fn call_nz_a16(&mut self) -> u32 {
        if self.get_z() == 0 {
            return self.call_a16();
        }
        return 12;
    }
    fn call_nc_a16(&mut self) -> u32 {
        if self.get_cy() == 0 {
            return self.call_a16();
        }
        return 12;
    }
    fn call_z_a16(&mut self) -> u32 {
        if self.get_z() == 1 {
            return self.call_a16();
        }
        return 12;
    }
    fn call_c_a16(&mut self) -> u32 {
        if self.get_cy() == 1 {
            return self.call_a16();
        }
        return 12;
    }

    fn ret_a16(&mut self) -> u32 {
        let lsb = self.mem_read(self.sp + 1);
        let msb = self.mem_read(self.sp + 2);
        self.sp = self.sp + 2;
        self.pc = ((msb as u16) << 8) | (lsb as u16);
        return 16;
    }
    fn ret_nz_a16(&mut self) -> u32 {
        if self.get_z() == 0 {
            return self.ret_a16();
        }
        return 12;
    }
    fn ret_nc_a16(&mut self) -> u32 {
        if self.get_cy() == 0 {
            return self.ret_a16();
        }
        return 12;
    }
    fn ret_z_a16(&mut self) -> u32 {
        if self.get_z() == 1 {
            return self.ret_a16();
        }
        return 12;
    }
    fn ret_c_a16(&mut self) -> u32 {
        if self.get_cy() == 1 {
            return self.ret_a16();
        }
        return 12;
    }
    fn reti_a16(&mut self) -> u32 {
        self.ime = 1;
        return self.ret_a16();
    }
    fn push_r16(&mut self, val: u16) -> u32 {
        let lsb = val as u8;
        let msb = (val >> 8) as u8;
        self.mem_write(self.sp, msb);
        self.mem_write(self.sp-1, lsb);
        self.sp = self.sp - 2;
        return 16;
    }
    fn rst_n8(&mut self, val: u8) -> u32 {
        self.pc = val as u16;
        return 16;
    }
    fn pop_bc(&mut self) -> u32 {
        let msb = self.mem_read(self.sp+2) as u16;
        let lsb = self.mem_read(self.sp+1) as u16;
        self.sp = self.sp + 2;
        self.bc = (msb << 8) | lsb;
        return 12;
    }
    fn pop_de(&mut self) -> u32 {
        let msb = self.mem_read(self.sp+2) as u16;
        let lsb = self.mem_read(self.sp+1) as u16;
        self.sp = self.sp + 2;
        self.de = (msb << 8) | lsb;
        return 12;
    }
    fn pop_hl(&mut self) -> u32 {
        let msb = self.mem_read(self.sp+2) as u16;
        let lsb = self.mem_read(self.sp+1) as u16;
        self.sp = self.sp + 2;
        self.hl = (msb << 8) | lsb;
        return 12;
    }
    fn pop_af(&mut self) -> u32 {
        let msb = self.mem_read(self.sp+2) as u16;
        let lsb = self.mem_read(self.sp+1) as u16;
        self.sp = self.sp + 2;
        self.af = (msb << 8) | lsb;
        return 12;
    }
    fn daa(&mut self) -> u32 {
        let cf = self.get_cy();
        let hf = self.get_hc();
        let nf = self.get_n();
        let mut a = self.get_a();
        if hf == 1 && nf == 0 {
            let (a, result) = a.overflowing_add(0x06);
        } else if cf == 1 && nf == 0 {
            let (a, _) = a.overflowing_add(0x60);
        } else if hf == 1 && nf == 1 {
            let (a, _) = a.overflowing_sub(0x06);
        } else if cf == 1 && nf == 1 {
            let (a, _) = a.overflowing_sub(0x60);
        }
        self.set_a(a);
        return 4;
    }
    fn scf(&mut self) -> u32 {
        self.set_cy(1);
        self.set_n(0);
        self.set_hc(0);
        return 4;
    }
    fn cpl(&mut self) -> u32 {
        let a = self.get_a();
        self.set_a(!a);
        return 4;
    }
    fn ccf(&mut self) -> u32 {
        if self.get_cy() == 0 {
            self.set_cy(1);
        } else {
            self.set_cy(0);
        }
        self.set_n(0);
        self.set_hc(0);
        return 4;
    }
}


// RLC Tests
#[test]
fn rlc_b_carry() {
    let mut gb = GB::new();
    gb.pc = 0x100;
    gb.set_b(0b11001100);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::rlc, &GB::set_b);
    assert_eq!(gb.get_b(), 0b10011001);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rlc_b_no_carry() {
    let mut gb = GB::new();
    gb.set_b(0b00110011);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::rlc, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01100110);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn rlc_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.set_cy(1);
    gb.shift_mem(&GB::rlc);
    assert_eq!(gb.mem_read(addr), 0b10011001);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rlc_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.set_cy(1);
    gb.shift_mem(&GB::rlc);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.get_cy(), 0);
}


// RRC Tests
#[test]
fn rrc_b_carry() {
    let mut gb = GB::new();
    gb.set_b(0b00110011);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::rrc, &GB::set_b);
    assert_eq!(gb.get_b(), 0b10011001);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rrc_b_no_carry() {
    let mut gb = GB::new();
    gb.set_b(0b11001100);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::rrc, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01100110);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn rrc_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.set_cy(0);
    gb.shift_mem(&GB::rrc);
    assert_eq!(gb.mem_read(addr), 0b10011001);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rrc_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.set_cy(1);
    gb.shift_mem(&GB::rrc);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.get_cy(), 0);
}


// RL Tests
#[test]
fn rl_b_carry() {
    let mut gb = GB::new();
    gb.set_b(0b11001100);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::rl, &GB::set_b);
    assert_eq!(gb.get_b(), 0b10011000);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rl_b_no_carry() {
    let mut gb = GB::new();
    gb.set_b(0b00110011);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::rl, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01100111);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn rl_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.set_cy(0);
    gb.shift_mem(&GB::rl);
    assert_eq!(gb.mem_read(addr), 0b10011000);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rl_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.set_cy(1);
    gb.shift_mem(&GB::rl);
    assert_eq!(gb.mem_read(addr), 0b01100111);
    assert_eq!(gb.get_cy(), 0);
}


// RR Tests
#[test]
fn rr_b_carry() {
    let mut gb = GB::new();
    gb.set_b(0b00110011);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::rr, &GB::set_b);
    assert_eq!(gb.get_b(), 0b00011001);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rr_b_no_carry() {
    let mut gb = GB::new();
    gb.set_b(0b11001100);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::rr, &GB::set_b);
    assert_eq!(gb.get_b(), 0b11100110);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn rr_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.set_cy(0);
    gb.shift_mem(&GB::rr);
    assert_eq!(gb.mem_read(addr), 0b00011001);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn rr_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.set_cy(1);
    gb.shift_mem(&GB::rr);
    assert_eq!(gb.mem_read(addr), 0b11100110);
    assert_eq!(gb.get_cy(), 0);
}


// SLA Tests
#[test]
fn sla_b_carry() {
    let mut gb = GB::new();
    gb.set_b(0b11001100);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::sla, &GB::set_b);
    assert_eq!(gb.get_b(), 0b10011000);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn sla_b_no_carry() {
    let mut gb = GB::new();
    gb.set_b(0b00110011);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::sla, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01100110);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn sla_hl_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001100);
    gb.set_cy(0);
    gb.shift_mem(&GB::sla);
    assert_eq!(gb.mem_read(addr), 0b10011000);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn sla_hl_no_carry() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110011);
    gb.set_cy(1);
    gb.shift_mem(&GB::sla);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.get_cy(), 0);
}


// SRA Tests
#[test]
fn sra_b_positive() {
    let mut gb = GB::new();
    gb.set_b(0b00110010);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::sra, &GB::set_b);
    assert_eq!(gb.get_b(), 0b00011001);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn sra_b_negative() {
    let mut gb = GB::new();
    gb.set_b(0b11001101);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::sra, &GB::set_b);
    assert_eq!(gb.get_b(), 0b11100110);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn sra_hl_positive() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110010);
    gb.set_cy(1);
    gb.shift_mem(&GB::sra);
    assert_eq!(gb.mem_read(addr), 0b00011001);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn sra_hl_negative() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001101);
    gb.set_cy(0);
    gb.shift_mem(&GB::sra);
    assert_eq!(gb.mem_read(addr), 0b11100110);
    assert_eq!(gb.get_cy(), 1);
}


// SWAP Tests
#[test]
fn swap_b() {
    let mut gb = GB::new();
    gb.set_b(0b10110100);
    gb.shift_r8(&GB::get_b, &GB::swap, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01001011);
}
#[test]
fn swap_hl() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b10110100);
    gb.shift_mem(&GB::swap);
    assert_eq!(gb.mem_read(addr), 0b01001011);
}


// SRL Tests
#[test]
fn srl_b_positive() {
    let mut gb = GB::new();
    gb.set_b(0b00110010);
    gb.set_cy(1);
    gb.shift_r8(&GB::get_b, &GB::srl, &GB::set_b);
    assert_eq!(gb.get_b(), 0b00011001);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn srl_b_negative() {
    let mut gb = GB::new();
    gb.set_b(0b11001101);
    gb.set_cy(0);
    gb.shift_r8(&GB::get_b, &GB::srl, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01100110);
    assert_eq!(gb.get_cy(), 1);
}
#[test]
fn srl_hl_positive() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00110010);
    gb.set_cy(1);
    gb.shift_mem(&GB::srl);
    assert_eq!(gb.mem_read(addr), 0b00011001);
    assert_eq!(gb.get_cy(), 0);
}
#[test]
fn srl_hl_negative() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11001101);
    gb.set_cy(0);
    gb.shift_mem(&GB::srl);
    assert_eq!(gb.mem_read(addr), 0b01100110);
    assert_eq!(gb.get_cy(), 1);
}


// BIT Tests
#[test]
fn bit_0_b_on() {
    let mut gb = GB::new();
    gb.set_b(0b00000001);
    gb.set_z(0);
    gb.shift_r8(&GB::get_b, &GB::bit_0, &GB::set_b);
    assert_eq!(gb.get_b(), 0b00000001);
    assert_eq!(gb.get_z(), 1);
}
#[test]
fn bit_0_b_off() {
    let mut gb = GB::new();
    gb.set_b(0b11111110);
    gb.set_z(1);
    gb.shift_r8(&GB::get_b, &GB::bit_0, &GB::set_b);
    assert_eq!(gb.get_b(), 0b11111110);
    assert_eq!(gb.get_z(), 0);
}
#[test]
fn bit_0_hl_on() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00000001);
    gb.set_z(0);
    gb.shift_mem(&GB::bit_0);
    assert_eq!(gb.mem_read(addr), 0b00000001);
    assert_eq!(gb.get_z(), 1);
}
#[test]
fn bit_0_hl_off() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11111110);
    gb.set_z(1);
    gb.shift_mem(&GB::bit_0);
    assert_eq!(gb.mem_read(addr), 0b11111110);
    assert_eq!(gb.get_z(), 0);
}
#[test]
fn bit_6_b_on() {
    let mut gb = GB::new();
    gb.set_b(0b01000000);
    gb.set_z(0);
    gb.shift_r8(&GB::get_b, &GB::bit_6, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01000000);
    assert_eq!(gb.get_z(), 1);
}
#[test]
fn bit_6_b_off() {
    let mut gb = GB::new();
    gb.set_b(0b10111111);
    gb.set_z(1);
    gb.shift_r8(&GB::get_b, &GB::bit_6, &GB::set_b);
    assert_eq!(gb.get_b(), 0b10111111);
    assert_eq!(gb.get_z(), 0);
}
#[test]
fn bit_6_hl_on() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b01000000);
    gb.set_z(0);
    gb.shift_mem(&GB::bit_6);
    assert_eq!(gb.mem_read(addr), 0b01000000);
    assert_eq!(gb.get_z(), 1);
}
#[test]
fn bit_6_hl_off() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b10111111);
    gb.set_z(1);
    gb.shift_mem(&GB::bit_6);
    assert_eq!(gb.mem_read(addr), 0b10111111);
    assert_eq!(gb.get_z(), 0);
}


// RES Tests
#[test]
fn res_0_b() {
    let mut gb = GB::new();
    gb.set_b(0b11111111);
    gb.shift_r8(&GB::get_b, &GB::res_0, &GB::set_b);
    assert_eq!(gb.get_b(), 0b11111110);
}
#[test]
fn res_0_hl() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11111111);
    gb.shift_mem(&GB::res_0);
    assert_eq!(gb.mem_read(addr), 0b11111110);
}
#[test]
fn res_6_b() {
    let mut gb = GB::new();
    gb.set_b(0b11111111);
    gb.shift_r8(&GB::get_b, &GB::res_6, &GB::set_b);
    assert_eq!(gb.get_b(), 0b10111111);
}
#[test]
fn res_6_hl() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b11111111);
    gb.shift_mem(&GB::res_6);
    assert_eq!(gb.mem_read(addr), 0b10111111);
}


// SET Tests
#[test]
fn set_0_b() {
    let mut gb = GB::new();
    gb.set_b(0b00000000);
    gb.shift_r8(&GB::get_b, &GB::set_0, &GB::set_b);
    assert_eq!(gb.get_b(), 0b00000001);
}
#[test]
fn set_0_hl() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00000000);
    gb.shift_mem(&GB::set_0);
    assert_eq!(gb.mem_read(addr), 0b00000001);
}
#[test]
fn set_6_b() {
    let mut gb = GB::new();
    gb.set_b(0b00000000);
    gb.shift_r8(&GB::get_b, &GB::set_6, &GB::set_b);
    assert_eq!(gb.get_b(), 0b01000000);
}
#[test]
fn set_6_hl() {
    let mut gb = GB::new();
    let addr = 0xC000;
    gb.set_hl(addr);
    gb.mem_write(addr, 0b00000000);
    gb.shift_mem(&GB::set_6);
    assert_eq!(gb.mem_read(addr), 0b01000000);
}


// LD Tests
#[test]
fn ld_bc_d16_test() {
    let mut gb = GB::new();
    let pc = gb.pc;
    gb.mem_write(pc+1, 0xDE);
    gb.mem_write(pc+2, 0xAD);
    gb.bc = 0x0000;
    gb.ld_bc_d16();
    assert_eq!(gb.bc, 0xDEAD);
}
#[test]
fn ld_de_d16_test() {
    let mut gb = GB::new();
    let pc = gb.pc;
    gb.mem_write(pc+1, 0xDE);
    gb.mem_write(pc+2, 0xAD);
    gb.de = 0x0000;
    gb.ld_de_d16();
    assert_eq!(gb.de, 0xDEAD);
}
#[test]
fn ld_hl_d16_test() {
    let mut gb = GB::new();
    let pc = gb.pc;
    gb.mem_write(pc+1, 0xDE);
    gb.mem_write(pc+2, 0xAD);
    gb.hl = 0x0000;
    gb.ld_hl_d16();
    assert_eq!(gb.hl, 0xDEAD);
}
#[test]
fn ld_sp_d16_test() {
    let mut gb = GB::new();
    let pc = gb.pc;
    gb.mem_write(pc+1, 0xDE);
    gb.mem_write(pc+2, 0xAD);
    gb.sp = 0x0000;
    gb.ld_sp_d16();
    assert_eq!(gb.sp, 0xDEAD);
}
#[test]
fn ld_bc_mem_test() {
    let mut gb = GB::new();
    gb.bc = 0xC000;
    gb.set_a(0xAF);
    gb.mem_write(gb.bc, 0x00);
    gb.ld_r16_mem(gb.bc);
    assert_eq!(gb.mem_read(gb.bc), 0xAF);
}
#[test]
fn ld_de_mem_test() {
    let mut gb = GB::new();
    gb.de = 0xC000;
    gb.set_a(0xAF);
    gb.mem_write(gb.de, 0x00);
    gb.ld_r16_mem(gb.de);
    assert_eq!(gb.mem_read(gb.de), 0xAF);
}
#[test]
fn ld_hl_mem_inc_test() {
    let mut gb = GB::new();
    gb.hl = 0xC000;
    gb.set_a(0xAF);
    gb.mem_write(gb.hl, 0x00);
    gb.ld_hl_mem_inc();
    assert_eq!(gb.hl, 0xC001);
    assert_eq!(gb.mem_read(gb.hl-1), 0xAF);
}
#[test]
fn ld_hl_mem_dec_test() {
    let mut gb = GB::new();
    gb.hl = 0xC000;
    gb.set_a(0xAF);
    gb.mem_write(gb.hl, 0x00);
    gb.ld_hl_mem_dec();
    assert_eq!(gb.hl, 0xBFFF);
    assert_eq!(gb.mem_read(gb.hl+1), 0xAF);
}
#[test]
fn ld_b_c_test() {
    let mut gb = GB::new();
    gb.set_b(0xAF);
    gb.set_c(0xDE);
    gb.ld_r8_r8(&GB::set_b, &GB::get_c);
    assert_eq!(gb.get_b(), 0xDE);
}
#[test]
fn ld_mem_hl_b_test() {
    let mut gb = GB::new();
    gb.set_b(0xAF);
    gb.mem_write(gb.hl, 0x00);
    gb.ld_mem_r16_r8(gb.hl, &GB::get_b);
    assert_eq!(gb.mem_read(gb.hl), 0xAF);
}
#[test]
fn ld_b_mem_hl_test() {
    let mut gb = GB::new();
    gb.set_b(0x00);
    gb.mem_write(gb.hl, 0xAF);
    gb.ld_r8_mem_r16(&GB::set_b, gb.hl);
    assert_eq!(gb.get_b(), 0xAF);
}
#[test]
fn ld_b_d8_test() {
    let mut gb = GB::new();
    gb.set_b(0x00);
    gb.ld_r8_d8(&GB::set_b, 0xAF);
    assert_eq!(gb.get_b(), 0xAF);
}
#[test]
fn ld_mem_hl_d8_test() {
    let mut gb = GB::new();
    gb.mem_write(gb.hl, 0x00);
    gb.ld_mem_r16_d8(gb.hl, 0xAF);
    assert_eq!(gb.mem_read(gb.hl), 0xAF);
}
#[test]
fn ld_mem_a16_sp_test() {
    let mut gb = GB::new();
    gb.mem_write(0xC000, 0x00);
    gb.mem_write(0xC000, 0x00);
    gb.sp = 0xDEAD;
    gb.ld_mem_a16_r16(0xC000, gb.sp);
    assert_eq!(gb.mem_read(0xC000), 0xAD);
    assert_eq!(gb.mem_read(0xC001), 0xDE);
}
#[test]
fn ld_a_mem_bc_test() {
    let mut gb = GB::new();
    gb.set_a(0x00);
    gb.mem_write(gb.bc, 0xFF);
    gb.ld_r8_mem_r16(&GB::set_a, gb.bc);
    assert_eq!(gb.get_a(), 0xFF);
}
#[test]
fn ld_a_mem_hl_inc_test() {
    let mut gb = GB::new();
    gb.set_a(0x00);
    gb.hl = 0x1F;
    gb.mem_write(gb.hl, 0xFE);
    gb.ld_a_mem_hl_inc();
    assert_eq!(gb.get_a(), 0xFE);
    assert_eq!(gb.hl, 0x20);
}
#[test]
fn ld_a_mem_hl_dec_test() {
    let mut gb = GB::new();
    gb.set_a(0x00);
    gb.hl = 0x1F;
    gb.mem_write(gb.hl, 0xFF);
    gb.ld_a_mem_hl_dec();
    assert_eq!(gb.get_a(), 0xFF);
    assert_eq!(gb.hl, 0x1E);
}
#[test]
fn ldh_mem_a8_r8_test() {
    let mut gb = GB::new();
    gb.mem_write(0xFF20, 0x00);
    gb.set_a(0x11);
    gb.ldh_mem_a8_r8(0x20, &GB::get_a);
    assert_eq!(gb.mem_read(0xFF20), 0x11);
}
#[test]
fn ldh_r8_mem_a8_test() {
    let mut gb = GB::new();
    gb.mem_write(0xFF20, 0x11);
    gb.set_a(0x00);
    gb.ldh_r8_mem_a8(&GB::set_a, 0x20);
    assert_eq!(gb.get_a(), 0x11);
}
#[test]
fn ld_mem_r8_r8_test() {
    let mut gb = GB::new();
    gb.mem_write(0xFF20, 0x00);
    gb.set_a(0x11);
    gb.set_c(0x20);
    gb.ld_mem_r8_r8(&GB::get_c, &GB::get_a);
    assert_eq!(gb.mem_read(0xFF20), 0x11);
}
#[test]
fn ld_r8_mem_r8_test() {
    let mut gb = GB::new();
    gb.mem_write(0xFF20, 0x11);
    gb.set_a(0x00);
    gb.set_c(0x20);
    gb.ld_r8_mem_r8(&GB::set_a, &GB::get_c);
    assert_eq!(gb.get_a(), 0x11);
}
#[test]
fn ld_hl_sp_plus_r8_test() {
    let mut gb = GB::new();
    gb.hl = 0;
    gb.sp = 0x0010;
    let val = 0xFF;
    gb.ld_hl_sp_plus_a8(val);
    assert_eq!(gb.hl, 0x10F);
}
#[test]
fn ld_sp_hl_test() {
    let mut gb = GB::new();
    gb.sp = 0;
    gb.hl = 0xDEAD;
    gb.ld_sp_hl();
    assert_eq!(gb.sp, 0xDEAD);
}
#[test]
fn ld_mem_a16_a_test() {
    let mut gb = GB::new();
    let addr = 0xDEAD;
    gb.mem_write(gb.pc+1, 0xDE);
    gb.mem_write(gb.pc+2, 0xAD);
    let val = 0x11;
    gb.set_a(val);
    gb.ld_mem_a16_a();
    assert_eq!(gb.mem_read(addr), val);
}
#[test]
fn ld_a_mem_a16_test() {
    let mut gb = GB::new();
    let addr = 0xDEAD;
    gb.mem_write(gb.pc+1, 0xDE);
    gb.mem_write(gb.pc+2, 0xAD);
    let val = 0x11;
    gb.set_a(0x00);
    gb.mem_write(addr, val);
    gb.ld_a_mem_a16();
    assert_eq!(gb.get_a(), val);
}

// ADD Commands
#[test]
fn adc_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0xF0);
    gb.set_cy(0x1);
    gb.adc_r8(0x0F);
    assert_eq!(gb.get_a(), 0x00);
    assert_eq!(gb.get_cy(), 0x01);
}
#[test]
fn add_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0xF0);
    gb.set_cy(0x1);
    gb.add_r8(0x0F);
    assert_eq!(gb.get_a(), 0xFF);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn add_hl_bc_test() {
    let mut gb = GB::new();
    gb.bc = 0x1111;
    gb.hl = 0x2222;
    gb.add_hl_bc();
    assert_eq!(gb.hl, 0x3333);
}
#[test]
fn and_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0b10011001);
    gb.set_cy(0x1);
    gb.and_r8(0b11110000);
    assert_eq!(gb.get_a(), 0b10010000);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn cp_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0xF0);
    gb.set_cy(0x1);
    gb.cp_r8(0x0F);
    assert_eq!(gb.get_a(), 0xF0);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn or_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0b10011001);
    gb.set_cy(0x1);
    gb.or_r8(0b11110000);
    assert_eq!(gb.get_a(), 0b11111001);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn sbc_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0xF0);
    gb.set_cy(0x1);
    gb.sbc_r8(0x0F);
    assert_eq!(gb.get_a(), 0xE0);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn sub_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0xF0);
    gb.set_cy(0x1);
    gb.sub_r8(0x0F);
    assert_eq!(gb.get_a(), 0xE1);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn xor_r8_test() {
    let mut gb = GB::new();
    gb.set_a(0b10011001);
    gb.set_cy(0x1);
    gb.xor_r8(0b11110000);
    assert_eq!(gb.get_a(), 0b01101001);
    assert_eq!(gb.get_cy(), 0x00);
}
#[test]
fn dec_r8_test() {
    let mut gb = GB::new();
    gb.set_b(0x10);
    gb.set_hc(1);
    gb.dec_r8(&GB::set_b, &GB::get_b);
    assert_eq!(gb.get_b(), 0x0F);
    assert_eq!(gb.get_hc(), 0x00);
}
#[test]
fn inc_r8_test() {
    let mut gb = GB::new();
    gb.set_b(0x0F);
    gb.set_hc(0);
    gb.inc_r8(&GB::set_b, &GB::get_b);
    assert_eq!(gb.get_b(), 0x10);
    assert_eq!(gb.get_hc(), 0x01);
}
#[test]
fn dec_r8_mem_test() {
    let mut gb = GB::new();
    gb.mem_write(gb.hl, 0x10);
    gb.set_hc(1);
    gb.dec_r8_mem();
    assert_eq!(gb.mem_read(gb.hl), 0x0F);
    assert_eq!(gb.get_hc(), 0x00);
}
#[test]
fn inc_r8_mem_test() {
    let mut gb = GB::new();
    gb.mem_write(gb.hl, 0x0F);
    gb.set_hc(0);
    gb.inc_r8_mem();
    assert_eq!(gb.mem_read(gb.hl), 0x10);
    assert_eq!(gb.get_hc(), 0x01);
}
#[test]
fn dec_bc_test() {
    let mut gb = GB::new();
    gb.bc = (0x1111);
    gb.dec_bc();
    assert_eq!(gb.bc, 0x1110);
}
#[test]
fn inc_bc_test() {
    let mut gb = GB::new();
    gb.bc = (0x1111);
    gb.inc_bc();
    assert_eq!(gb.bc, 0x1112);
}
#[test]
fn ei_test() {
    let mut gb = GB::new();
    gb.ime = 0;
    gb.ei();
    assert_eq!(gb.ime, 1);
}
#[test]
fn di_test() {
    let mut gb = GB::new();
    gb.ime = 1;
    gb.di();
    assert_eq!(gb.ime, 0);
}
#[test]
fn jp_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.jp_a16();
    assert_eq!(gb.pc, 0xDEAD);
}
#[test]
fn jp_hl_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.hl = 0xDEAD;
    gb.jp_hl();
    assert_eq!(gb.pc, 0xDEAD);
}
#[test]
fn jp_nz_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_z(1);
    gb.jp_nz();
    assert_eq!(gb.pc, 0x0000);
    gb.set_z(0);
    gb.jp_nz();
    assert_eq!(gb.pc, 0xDEAD);
}
#[test]
fn jp_nc_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_cy(1);
    gb.jp_nc();
    assert_eq!(gb.pc, 0x0000);
    gb.set_cy(0);
    gb.jp_nc();
    assert_eq!(gb.pc, 0xDEAD);
}
#[test]
fn jp_z_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_z(0);
    gb.jp_z();
    assert_eq!(gb.pc, 0x0000);
    gb.set_z(1);
    gb.jp_z();
    assert_eq!(gb.pc, 0xDEAD);
}
#[test]
fn jp_c_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_cy(0);
    gb.jp_c();
    assert_eq!(gb.pc, 0x0000);
    gb.set_cy(1);
    gb.jp_c();
    assert_eq!(gb.pc, 0xDEAD);
}
#[test]
fn jr_a8_neg_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.jr_a8(0xFF_u8 as i8);
    assert_eq!(gb.pc, 0xFFFF - 0x7E);
}
#[test]
fn jr_a8_pos_test() {
    let mut gb = GB::new();
    gb.pc = 0xF000;
    gb.jr_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0xF01F);
}
#[test]
fn jr_nz_a8_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.set_z(1);
    gb.jr_nz_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x0000);
    gb.set_z(0);
    gb.jr_nz_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x1F);
}
#[test]
fn jr_nc_a8_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.set_cy(1);
    gb.jr_nc_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x0000);
    gb.set_cy(0);
    gb.jr_nc_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x1F);
}
#[test]
fn jr_z_a8_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.set_z(0);
    gb.jr_z_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x0000);
    gb.set_z(1);
    gb.jr_z_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x1F);
}
#[test]
fn jr_c_a8_test() {
    let mut gb = GB::new();
    gb.pc = 0x0000;
    gb.set_cy(0);
    gb.jr_c_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x0000);
    gb.set_cy(1);
    gb.jr_c_a8(0x1F_u8 as i8);
    assert_eq!(gb.pc, 0x1F);
}
#[test]
fn call_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFE;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.call_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFC);
    assert_eq!(gb.mem_read(gb.sp+1), 0x10);
    assert_eq!(gb.mem_read(gb.sp+2), 0x11);
}
#[test]
fn call_nz_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFE;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_z(1);
    gb.call_nz_a16();
    assert_eq!(gb.pc, 0x1110);
    gb.set_z(0);
    gb.call_nz_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFC);
    assert_eq!(gb.mem_read(gb.sp+1), 0x10);
    assert_eq!(gb.mem_read(gb.sp+2), 0x11);
}
#[test]
fn call_nc_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFE;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_cy(1);
    gb.call_nc_a16();
    assert_eq!(gb.pc, 0x1110);
    gb.set_cy(0);
    gb.call_nc_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFC);
    assert_eq!(gb.mem_read(gb.sp+1), 0x10);
    assert_eq!(gb.mem_read(gb.sp+2), 0x11);
}
#[test]
fn call_z_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFE;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_z(0);
    gb.call_z_a16();
    assert_eq!(gb.pc, 0x1110);
    gb.set_z(1);
    gb.call_z_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFC);
    assert_eq!(gb.mem_read(gb.sp+1), 0x10);
    assert_eq!(gb.mem_read(gb.sp+2), 0x11);
}
#[test]
fn call_c_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFE;
    gb.mem_write(gb.pc + 1, 0xDE);
    gb.mem_write(gb.pc + 2, 0xAD);
    gb.set_cy(0);
    gb.call_c_a16();
    assert_eq!(gb.pc, 0x1110);
    gb.set_cy(1);
    gb.call_c_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFC);
    assert_eq!(gb.mem_read(gb.sp+1), 0x10);
    assert_eq!(gb.mem_read(gb.sp+2), 0x11);
}
#[test]
fn ret_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFC;
    gb.mem_write(gb.sp + 1, 0xAD);
    gb.mem_write(gb.sp + 2, 0xDE);
    gb.ret_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFE);
}
#[test]
fn ret_nz_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFC;
    gb.mem_write(gb.sp + 1, 0xAD);
    gb.mem_write(gb.sp + 2, 0xDE);
    gb.set_z(1);
    gb.ret_nz_a16();
    assert_eq!(gb.pc, 0x1110);
    assert_eq!(gb.sp, 0xFFFC);
    gb.set_z(0);
    gb.ret_nz_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFE);
}
#[test]
fn ret_nc_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFC;
    gb.mem_write(gb.sp + 1, 0xAD);
    gb.mem_write(gb.sp + 2, 0xDE);
    gb.set_cy(1);
    gb.ret_nc_a16();
    assert_eq!(gb.pc, 0x1110);
    assert_eq!(gb.sp, 0xFFFC);
    gb.set_cy(0);
    gb.ret_nc_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFE);
}
#[test]
fn ret_z_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFC;
    gb.mem_write(gb.sp + 1, 0xAD);
    gb.mem_write(gb.sp + 2, 0xDE);
    gb.set_z(0);
    gb.ret_z_a16();
    assert_eq!(gb.pc, 0x1110);
    assert_eq!(gb.sp, 0xFFFC);
    gb.set_z(1);
    gb.ret_z_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFE);
}
#[test]
fn ret_c_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFC;
    gb.mem_write(gb.sp + 1, 0xAD);
    gb.mem_write(gb.sp + 2, 0xDE);
    gb.set_cy(0);
    gb.ret_c_a16();
    assert_eq!(gb.pc, 0x1110);
    assert_eq!(gb.sp, 0xFFFC);
    gb.set_cy(1);
    gb.ret_c_a16();
    assert_eq!(gb.pc, 0xDEAD);
    assert_eq!(gb.sp, 0xFFFE);
}
#[test]
fn reti_a16_test() {
    let mut gb = GB::new();
    gb.pc = 0x1110;
    gb.sp = 0xFFFC;
    gb.ime = 0;
    gb.reti_a16();
    assert_eq!(gb.ime, 1);
}
#[test]
fn push_a16_test() {
    let mut gb = GB::new();
    gb.bc = 0x1110;
    gb.sp = 0xFFFE;
    gb.push_r16(gb.bc);
    assert_eq!(gb.sp, 0xFFFC);
    assert_eq!(gb.mem_read(gb.sp+1), 0x10);
    assert_eq!(gb.mem_read(gb.sp+2), 0x11);
}
#[test]
fn pop_a16_test() {
    let mut gb = GB::new();
    gb.push_r16(0xDEAD);
    gb.bc = 0x0000;
    gb.pop_bc();
    assert_eq!(gb.sp, 0xFFFE);
    assert_eq!(gb.bc, 0xDEAD);
}
#[test]
fn rst_n8_test() {
    let mut gb = GB::new();
    gb.pc = 0xFFFF;
    gb.rst_n8(0x10);
    assert_eq!(gb.pc, 0x10);
}
#[test]
fn cpl_test() {
    let mut gb = GB::new();
    gb.set_a(0b11001010);
    gb.cpl();
    assert_eq!(gb.get_a(), 0b00110101);
}
#[test]
fn daa_cf_add_test() {
    let mut gb = GB::new();
    gb.set_a(0x90 + 0x10);
    gb.set_n(0);
    gb.daa();
    assert_eq!(gb.get_a(), 0x00);
}
#[test]
fn daa_hf_add_test() {
    let mut gb = GB::new();
    gb.set_a(0x09 + 0x01);
    gb.set_hc(1);
    gb.set_n(0);
    gb.daa();
    assert_eq!(gb.get_a(), 0x10);
}
#[test]
fn daa_cy_sub_test() {
    // TODO: Implement this
}
#[test]
fn daa_hf_sub_test() {
    let mut gb = GB::new();
    gb.set_a(0x10 - 0x09);
    gb.set_n(1);
    gb.set_hc(1);
    gb.daa();
    assert_eq!(gb.get_a(), 0x01);
}
