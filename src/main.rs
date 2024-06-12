mod interfaces;
mod processor;

use crossterm::terminal::disable_raw_mode;
use interfaces::{Interface, OpenGlInterface, TerminalInterface};

use clap::{Parser, ValueEnum};
use processor::Processor;
use std::boxed::Box;
use std::thread;
use std::time::Instant;
use std::{
    fs,
    fs::{File, OpenOptions},
    io::{self, Write},
    time::Duration,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Terminal,
    OpenGl,
}

impl ToString for Mode {
    fn to_string(&self) -> String {
        return String::from(match self {
            Mode::Terminal => "terminal",
            Mode::OpenGl => "open_gl",
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
    #[arg(short, long, default_value_t = Mode::Terminal)]
    mode: Mode,

    // File to read
    #[arg(short, long, default_value_t = String::new())]
    file: String,

    // Optional debug output file, to write all the instructions the processor runs through
    #[arg(long, default_value_t = String::new())]
    debug_file: String,
}

fn main() {
    let args = Args::parse();
    let mut p = Processor::new();
    let mut interface: Box<dyn Interface> = match args.mode {
        Mode::Terminal => Box::new(TerminalInterface::new()),
        Mode::OpenGl => Box::new(OpenGlInterface::new()),
    };

    let mut file: Option<File> = None;
    if !args.debug_file.is_empty() {
        file = Some(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(args.debug_file.clone())
                .unwrap(),
        );
        writeln!(file.as_ref().unwrap(), "BEGINNING OF OPCODE RECORD:").unwrap();
    }
    if args.file.is_empty() {
        disable_raw_mode().unwrap();
        loop {
            interface.render(&p);
            print!("Enter a command: ");
            io::stdout().flush().unwrap();
            let mut str = String::new();
            io::stdin().read_line(&mut str).unwrap();
            let hex = u16::from_str_radix(str.trim(), 16).unwrap();
            p.execute(hex);
        }
    }
    let data: Vec<u8> = fs::read(args.file.clone()).unwrap();
    p.load_program(data.as_slice());
    let mut dt = Instant::now();
    let mut last_inst: u16 = 0x0;
    loop {
        let pc = p.get_program_counter();
        if !args.debug_file.is_empty() {
            let inst = ((p.get_mem_at(pc) as u16) << 8) + p.get_mem_at(pc + 1) as u16;
            // Don't write the exact same instruction multiple times in a row
            if inst != last_inst {
                let to_write = format!("{:#6X}", inst);
                writeln!(
                    file.as_ref().unwrap(),
                    "{:#6x} {}",
                    pc,
                    to_write[2..].to_string()
                )
                .unwrap();
                last_inst = inst;
            }
        }
        p.step();

        if interface.update(&mut p) {
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
        thread::sleep(Duration::from_micros(10));
    }
    interface.exit();
}
