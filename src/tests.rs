use std::fs::File;
use std::io::prelude::*;
use crate::gb::GB;

// #[test]
// fn load_app_rom_only() {
//     let mut gb = gb::GB::new();

//     if !gb.load_application(&args[1]){
//         panic!("failed load rom");
//     }

//     let mut file = File::open("tetris.gb").expect("File error");
//     let fsize = file.metadata().unwrap().len();

//     let mut buffer = vec![];
//     file.read_to_end(&mut buffer).expect("couldn't read file");
//     drop(file);

//     if (0x8000) >= fsize {
//         for i in 0..fsize
//         {
//             assert_eq(gb.cart.rom[i as usize], buffer[i as usize]);
//         }
//     }
// }
