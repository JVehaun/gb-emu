mod gb;
mod tests;

use std::env;

fn main() {
    let mut timer = 0;
    let args: Vec<String> = env::args().collect();
    if args.len() != 2
    {
        println!("syntax: gb_emu [rom_file]");
        return;
    }
    let mut gb = gb::GB::new();
    if !gb.load_application(&args[1]){
        panic!("failed load rom");
    }

    gb.print_memory();

    'main: loop {
        gb.emulate_cycle();
    }
}
