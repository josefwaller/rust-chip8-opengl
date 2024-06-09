extern crate crossterm;
mod interface;
mod processor;

use crate::interface::{Interface, TerminalInterface};
use clap::{Parser, ValueEnum};
use processor::Processor;
use std::time::Instant;
use std::{
    fs,
    io::{self},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Ui {
    Terminal,
    OpenGl,
}

impl ToString for Ui {
    fn to_string(&self) -> String {
        return String::from(match self {
            Ui::Terminal => "terminal",
            Ui::OpenGl => "open_gl",
        });
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "Rust CHIP-8 Simulator")]
#[command(version = "0.1")]
#[command(about = "Simulate running CHIP-8 programs", long_about = None)]
struct Args {
    // UI to use
    // Either terminal (default) or opengl
    #[arg(short, long, default_value_t = Ui::Terminal)]
    mode: Ui,

    // File to read
    #[arg(short, long, default_value_t = String::new())]
    file: String,
}

fn main() {
    let args = Args::parse();
    let mut p = Processor::new();
    let mut interface = TerminalInterface::new();
    if args.file.is_empty() {
        loop {
            interface.render(&p);
            println!("Enter a command: ");
            let mut str = String::new();
            io::stdin().read_line(&mut str).unwrap();
            let hex = u16::from_str_radix(str.trim(), 16).unwrap();
            p.execute(hex);
        }
    }
    let data: Vec<u8> = fs::read(args.file.clone()).unwrap();
    p.load_program(data.as_slice());
    let mut dt = Instant::now();
    loop {
        let pc = p.get_program_counter();
        p.step();

        if interface.update_inputs(&mut p) {
            break;
        }

        // For in console version, this is more reliable
        if dt.elapsed().as_millis() >= 1000 / 60 {
            p.on_tick();
            dt = Instant::now();
        }
        // Only rerender if we ran a render command
        let just_ran = p.get_mem_at(pc);
        if just_ran & 0xF0 == 0xD0 || just_ran == 0x00 {
            interface.render(&p);
        }
    }
    interface.exit();
}
