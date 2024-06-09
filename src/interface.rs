use crate::processor::Processor;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use std::{
    io::{stdout, Stdout, Write},
    time::{Duration, Instant},
};

pub trait Interface {
    fn new() -> Self;
    // Update the inputs in the processor
    // Return true if the program should exit, false otherwise
    fn update_inputs(&mut self, p: &mut Processor) -> bool;
    fn render(&mut self, p: &Processor);
    fn exit(&mut self);
}

const KEY_MAP: [char; 16] = [
    'x', '1', '2', '3', 'q', 'w', 'e', 'a', 's', 'd', 'z', 'c', '4', 'r', 'f', 'v',
];

pub struct TerminalInterface {
    stdout: Stdout,
    input_dt: Instant,
}

impl Interface for TerminalInterface {
    fn new() -> TerminalInterface {
        let mut stdout = stdout();
        enable_raw_mode().unwrap();
        stdout.execute(Hide).unwrap();

        return TerminalInterface {
            stdout: stdout,
            input_dt: Instant::now(),
        };
    }

    fn update_inputs(&mut self, p: &mut Processor) -> bool {
        if self.input_dt.elapsed().as_micros() > 50 {
            let mut inputs = [false; 0x10];
            if poll(Duration::from_millis(1)).unwrap() {
                match read().unwrap() {
                    Event::Key(evt) => {
                        if evt.code == KeyCode::Char('c')
                            && evt.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            return true;
                        }
                        match evt.code {
                            KeyCode::Char(c) => match KEY_MAP.iter().position(|ch| *ch == c) {
                                Some(i) => inputs[i] = true,
                                None => {}
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            p.update_inputs(inputs);
            self.input_dt = Instant::now();
        }
        return false;
    }
    fn exit(&mut self) {
        disable_raw_mode().unwrap();
        self.stdout.execute(Show).unwrap();
    }

    fn render(&mut self, p: &Processor) {
        self.stdout.execute(Clear(ClearType::All)).unwrap();
        self.stdout.execute(MoveTo(0, 0)).unwrap();
        // Create a buffer for the actual screen for speed reasons
        let mut buf = [[' ' as u8; 2 * 64]; 32];
        for y in 0..32 {
            for x in 0..64 {
                if p.get_pixel_at(x as u8, y as u8) {
                    buf[y][2 * x] = '[' as u8;
                    buf[y][2 * x + 1] = ']' as u8;
                };
            }
        }
        let eol = ['\r' as u8, '\n' as u8];
        for row in buf {
            self.stdout.write(&row).unwrap();
            self.stdout.write(&eol).unwrap();
        }
        // Print debug information
        self.stdout.execute(MoveTo(0, 33)).unwrap();
        print!("  PC  |  I   |");
        (0..=0xF).for_each(|r| print!("  V{:x}  |", r));
        self.stdout.execute(MoveTo(0, 34)).unwrap();
        print!("{:#6X}|{:#6X}|", p.get_program_counter(), p.get_i());
        (0..=0xF).for_each(|r| print!(" {:#4X} |", p.get_register_value(r)));
        self.stdout.execute(MoveTo(0, 35)).unwrap();
        print!("  DT  |  ST  ");
        (0..=0xF).for_each(|i| print!("|  I{:X}  ", i));
        self.stdout.execute(MoveTo(0, 36)).unwrap();
        print!(" {:#4X?} | {:#4X?} ", p.get_dt(), p.get_st());
        (0..=0xF).for_each(|i| print!("|  {}   ", if p.get_input_state(i) { 'T' } else { 'F' }));
        self.stdout.execute(MoveTo(0, 37)).unwrap();
        self.stdout.flush().unwrap();
    }
}
