mod gb;

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
        println!("failed load rom");
        return
    }

    gb.print_memory();

    'main: loop {
        gb.emulate_cycle();
    }
}
