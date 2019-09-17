use std::fs::File;
use std::io::prelude::*;

pub struct VM {
    wram: [u8; 8192],
    vram: [u8; 8192],
    cpu: CPU,
    cart: Cartridge,
    regs: [u8; 0x7F],
    ei: u8,
}

pub struct CPU {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
}

pub struct Cartridge {
    rom: [u8; 8192],
    ram: [u8; 8192],
}

impl VM {

    fn mem_read(&mut self, addr: u16) -> u8 {
        if addr <= 0x3FFF {        // ROM Bank
            return self.cart.rom[addr as usize];
        } else if addr <= 0x7FFF { // ROM Bank 1-n
            return self.cart.rom[addr as usize];
        } else if addr <= 0x9FFF { // VRAM
            return self.vram[(addr - 0x8000) as usize];
        } else if addr <= 0xBFFF { // Cart RAM
            return self.cart.ram[addr as usize];
        } else if addr <= 0xDFFF { // Low RAM
            panic!("This mem shouldn't be accessed");
        } else if addr <= 0xFE9F { // OAM RAM
            panic!("This mem shouldn't be accessed");
        } else if addr <= 0xFF7F { // I/O Registers
            return self.regs[(addr - 0xFF00) as usize];
        } else if addr <= 0xFFFE { // High RAM
            panic!("This mem shouldn't be accessed");
        } else if addr == 0xFFFF { // Interrupt Enable
            return self.ei;
        }

    }

}
