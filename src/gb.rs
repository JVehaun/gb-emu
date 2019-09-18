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

    pub fn set_a(&mut self, val: u8) { self.af = (self.af & 0x00FF) | ((val as u16) << 8) }
    pub fn set_b(&mut self, val: u8) { self.bc = (self.bc & 0x00FF) | ((val as u16) << 8) }
    pub fn set_c(&mut self, val: u8) { self.bc = (self.bc & 0xFF00) | (val as u16) }
    pub fn set_d(&mut self, val: u8) { self.de = (self.de & 0x00FF) | ((val as u16) << 8) }
    pub fn set_e(&mut self, val: u8) { self.de = (self.de & 0xFF00) | (val as u16) }
    pub fn set_h(&mut self, val: u8) { self.hl = (self.hl & 0x00FF) | ((val as u16) << 8) }
    pub fn set_l(&mut self, val: u8) { self.hl = (self.hl & 0xFF00) | (val as u16) }

    pub fn get_z(&mut self) -> u8 {return ((self.af >> 7) & 0x1) as u8 }
    pub fn get_n(&mut self) -> u8 {return ((self.af >> 6) & 0x1) as u8 }
    pub fn get_hc(&mut self) -> u8 {return ((self.af >> 5) & 0x1) as u8 }
    pub fn get_cy(&mut self) -> u8 {return ((self.af >> 4) & 0x1) as u8 }

    pub fn set_z(&mut self, val: u8) {
        if val == 1 {self.af = self.af | (val as u16) << 7;}
        else {self.af = self.af & !((val as u16) << 7)}
    }
    pub fn set_n(&mut self, val: u8) {
        if val == 1 {self.af = self.af | (val as u16) << 6;}
        else {self.af = self.af & !((val as u16) << 6)}
    }
    pub fn set_hc(&mut self, val: u8) {
        if val == 1 { self.af = self.af | (val as u16) << 5; }
        else { self.af = self.af & !((val as u16) << 5) }
    }
    pub fn set_cy(&mut self, val: u8) {
        if val == 1 {self.af = self.af | (val as u16) << 4;}
        else {self.af = self.af & !((val as u16) << 4)}
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
        let mut opcode = (self.mem_read(self.cpu.pc), self.mem_read(self.cpu.pc+1));
        match opcode {
            (0xCB, 0x00) => { self.rlc_b() }
            (_, _)  => { return 1; }
        }
    }

    fn rlc_b(&mut self) -> u32 {
        let mut r = self.cpu.get_b();
        let cy = r >> 7;
        assert_eq!(cy, 1);
        r = (r << 1) | cy;
        self.cpu.set_cy(cy);
        self.cpu.set_b(r);
        if r == 0 {
            self.cpu.set_z(1);
        }
        return 8;
    }

}

#[test]
fn rlc_b_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b11001100);
    gb.cpu.set_cy(0);
    gb.rlc_b();
    assert_eq!(gb.cpu.get_b(), 0b10011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}

#[test]
fn rlc_b_no_carry() {
    let mut gb = GB::new();
    gb.cpu.set_b(0b00110011);
    gb.cpu.set_cy(1);
    gb.rlc_b();
    assert_eq!(gb.cpu.get_b(), 0b10011001);
    assert_eq!(gb.cpu.get_cy(), 1);
}
