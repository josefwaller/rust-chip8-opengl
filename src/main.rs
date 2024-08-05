mod errors;
mod interfaces;
mod processor;

use interfaces::Interface;
#[cfg(feature = "open-gl")]
use interfaces::OpenGlInterface;
#[cfg(feature = "terminal")]
use interfaces::TerminalInterface;

use clap::{Parser, ValueEnum};
use processor::Processor;
use std::boxed::Box;
use std::thread;
use std::time::{Duration, Instant};
use std::{
    fs,
    fs::{File, OpenOptions},
    io::Write,
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
#[command(version = "1.1.3")]
#[command(about = "Simulate running CHIP-8 programs", long_about = None)]
struct Args {
    // UI to use
    // Either terminal (default) or opengl
    #[arg(short, long, default_value_t = Mode::Terminal)]
    mode: Mode,

    // File to read
    #[arg(short, long)]
    file: String,

    // Optional debug output file, to write all the instructions the processor runs through
    #[arg(long, default_value_t = String::new())]
    debug_file: String,
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    let args = Args::parse();
    let mut p = Processor::new();
    #[cfg(all(not(feature = "terminal"), not(feature = "open-gl")))]
    panic!("No features enabled, enable one during compilation to use an interface");
    let mut interface: Box<dyn Interface> = match args.mode {
        #[cfg(not(feature = "terminal"))]
        Mode::Terminal => panic!("'terminal' feature needs to be enabled to use TerminalInterface"),
        #[cfg(feature = "terminal")]
        Mode::Terminal => Box::new(TerminalInterface::new()),
        #[cfg(not(feature = "open-gl"))]
        Mode::OpenGl => panic!("'open-gl' feature needs to be enabled to use OpenGlInterface"),
        #[cfg(feature = "open-gl")]
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
    let data: Vec<u8> = fs::read(args.file.clone()).unwrap();
    p.load_program(data.as_slice());
    let mut dt = Instant::now();
    let mut rt = Instant::now();
    let mut ct = Instant::now();
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
        p.step().unwrap();

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
        // Clock speed in Hz
        let clock_speed = 1000;
        thread::sleep(Duration::from_millis(1000 / clock_speed).saturating_sub(ct.elapsed()));
        ct = Instant::now();
    }
    interface.exit();
}
