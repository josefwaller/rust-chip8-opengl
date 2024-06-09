extern crate crossterm;
mod processor;

use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    QueueableCommand,
};
use processor::Processor;
use std::{
    env, fs,
    io::{self, stdout, Write},
};
use std::{
    io::Stdout,
    time::{Duration, Instant},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut p = Processor::new();
    let mut stdout = stdout();
    if args.len() < 2 {
        loop {
            render_scene(&p, &mut stdout);
            println!("Enter a command: ");
            let mut str = String::new();
            io::stdin().read_line(&mut str).unwrap();
            let hex = u16::from_str_radix(str.trim(), 16).unwrap();
            p.execute(hex);
        }
    }
    // TODO: Print a nicer message here
    let data: Vec<u8> = fs::read(args[1].clone()).unwrap();
    p.load_program(data.as_slice());
    enable_raw_mode().unwrap();
    const KEY_MAP: [char; 16] = [
        'x', '1', '2', '3', 'q', 'w', 'e', 'a', 's', 'd', 'z', 'c', '4', 'r', 'f', 'v',
    ];
    let mut input_dt = Instant::now();
    let mut dt = Instant::now();
    loop {
        let pc = p.get_program_counter();
        p.step();

        if input_dt.elapsed().as_micros() > 50 {
            let mut inputs = [false; 0x10];
            if poll(Duration::from_millis(1)).unwrap() {
                match read().unwrap() {
                    Event::Key(evt) => {
                        if evt.code == KeyCode::Char('c')
                            && evt.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            break;
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
            input_dt = Instant::now();
        }
        // For in console version, this is more reliable
        if dt.elapsed().as_millis() >= 1000 / 60 {
            p.on_tick();
            dt = Instant::now();
        }
        let just_ran = p.get_mem_at(pc);
        if just_ran & 0xF0 == 0xD0 || just_ran == 0x00 {
            render_scene(&p, &mut stdout);
        }
    }
    disable_raw_mode().unwrap();
}

fn render_scene(p: &Processor, stdout: &mut Stdout) {
    stdout.queue(Clear(ClearType::All)).unwrap();
    stdout.queue(MoveTo(0, 0)).unwrap();
    //let mut prev_black = true;
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
        stdout.write(&row).unwrap();
        stdout.write(&eol).unwrap();
    }
    stdout.queue(MoveTo(0, 33)).unwrap();
    (0..=0xF).for_each(|r| print!("V{:x} = {:#2X} ", r, p.get_register_value(r)));
    stdout.queue(MoveTo(0, 34)).unwrap();
    println!(
        "PC = {:X?}, I = {:X?}, DT = {:X?}, ST = {:X?}",
        p.get_program_counter(),
        p.get_i(),
        p.get_dt(),
        p.get_st()
    );
    stdout.queue(MoveTo(0, 35)).unwrap();
    (0..=0xF).for_each(|i| {
        print!(
            "I{:x} = {} ",
            i,
            if p.get_input_state(i) { 'T' } else { 'F' }
        )
    });
    println!("");
}
