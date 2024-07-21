mod interfaces;
mod processor;

use crossterm::terminal::disable_raw_mode;
use interfaces::{Interface, OpenGlInterface, TerminalInterface};

use clap::{Parser, ValueEnum};
use processor::Processor;
use std::boxed::Box;
use std::time::Instant;
use std::{
    fs,
    fs::{File, OpenOptions},
    io::{self, Write},
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

#[derive(Parser, Debug)]
#[command(name = "Rust CHIP-8 OpenGl")]
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
            print!("Enter a command (or exit to exit): ");
            io::stdout().flush().unwrap();
            let mut str = String::new();
            io::stdin().read_line(&mut str).unwrap();
            if str == String::from("exit\n") {
                interface.exit();
                return;
            }
            let hex = u16::from_str_radix(str.trim(), 16).unwrap();
            p.execute(hex);
        }
    }
    let data: Vec<u8> = fs::read(args.file.clone()).unwrap();
    p.load_program(data.as_slice());
    let mut dt = Instant::now();
    let mut rt = Instant::now();
    let mut last_pc: usize = 0x0000;
    loop {
        let pc = p.get_program_counter();
        if !args.debug_file.is_empty() {
            let inst = ((p.get_mem_at(pc) as u16) << 8) + p.get_mem_at(pc + 1) as u16;
            // Don't write the exact same instruction multiple times in a row
            if pc != last_pc {
                let to_write = format!("{:#6X}", inst);
                writeln!(
                    file.as_ref().unwrap(),
                    "{:#5X} {}",
                    pc,
                    to_write[2..].to_string()
                )
                .unwrap();
                last_pc = pc;
            }
        }
        p.step();

        if interface.update(&mut p) {
            break;
        }

        // Update clock
        if dt.elapsed().as_millis() >= 1000 / 60 {
            p.on_tick();
            dt = Instant::now();
        }

        // Render at 60Hz
        if rt.elapsed().as_millis() >= 60 {
            interface.render(&p);
            rt = Instant::now();
        }
    }
    interface.exit();
}
