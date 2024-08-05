use crate::interfaces::Interface;
use crate::processor::Processor;
extern crate crossterm;
extern crate rodio;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use rodio::{source::SineWave, OutputStream, Sink};

use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

const KEY_MAP: [char; 16] = [
    'x', '1', '2', '3', 'q', 'w', 'e', 'a', 's', 'd', 'z', 'c', '4', 'r', 'f', 'v',
];

/**
 * An interface that uses the terminal.
 * Mostly useful for debugging purposes.
 */
pub struct TerminalInterface {
    stdout: Stdout,
    sink: Option<rodio::Sink>,
    // Stream just needs to be kept in scope
    #[allow(dead_code)]
    stream: Option<rodio::OutputStream>,
}

impl TerminalInterface {
    pub fn new() -> TerminalInterface {
        let mut stdout = stdout();
        enable_raw_mode().unwrap();
        stdout.execute(Hide).unwrap();
        let device = OutputStream::try_default().ok();
        let sink = match &device {
            Some(d) => Sink::try_new(&d.1)
                .and_then(|s| {
                    s.set_volume(0.1);
                    s.append(SineWave::new(350.0));
                    s.pause();
                    return Ok(s);
                })
                .ok(),
            None => panic!("No sound!"),
        };
        return TerminalInterface {
            stdout,
            sink,
            stream: device.and_then(|d| Some(d.0)),
        };
    }
}
impl Interface for TerminalInterface {
    fn update(&mut self, p: &mut Processor) -> bool {
        let mut inputs: [bool; 0x10] = core::array::from_fn(|i| p.get_input_state(i));
        if poll(Duration::from_millis(0)).unwrap() {
            match read().unwrap() {
                Event::Key(evt) => {
                    if evt.code == KeyCode::Char('c')
                        && evt.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return true;
                    }
                    // For Terminal, we toggle the keys instead of detecting key up/key down
                    match evt.code {
                        KeyCode::Char(c) => match KEY_MAP.iter().position(|ch| *ch == c) {
                            Some(i) => {
                                inputs[i] = !inputs[i];
                            }
                            None => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
            p.update_inputs(inputs);
        }
        match &self.sink {
            Some(s) => {
                if s.is_paused() && p.get_st() > 0 {
                    s.play();
                } else if !s.is_paused() && p.get_st() == 0 {
                    s.pause();
                }
            }
            None => {}
        }
        return false;
    }
    fn exit(&mut self) {
        self.stdout.execute(Show).unwrap();
        disable_raw_mode().unwrap();
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
