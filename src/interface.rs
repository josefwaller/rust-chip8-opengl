extern crate gl;
extern crate glfw;

use crate::processor::Processor;
//use beryllium::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLuint};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use std::{
    array,
    ffi::CString,
    io::{stdout, Stdout, Write},
    mem,
    os::raw::c_void,
    ptr,
    time::{Duration, Instant},
};

pub trait Interface {
    //  fn new() -> Self;
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

impl TerminalInterface {
    pub fn new() -> TerminalInterface {
        let mut stdout = stdout();
        enable_raw_mode().unwrap();
        stdout.execute(Hide).unwrap();

        return TerminalInterface {
            stdout: stdout,
            input_dt: Instant::now(),
        };
    }
}
impl Interface for TerminalInterface {
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

pub struct OpenGlInterface {
    glfw: Glfw,
    events: GlfwReceiver<(f64, WindowEvent)>,
    window: PWindow,
    color: [f32; 3],
    cbo: GLuint,
}
const VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
out vec3 pos;
out vec3 vColor;
in vec3 color;

void main() {
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    pos = aPos;
    vColor = color;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 330 core
out vec4 color;
in vec3 vColor;

void main() {
    color = vec4(vColor, 1.0f);
}
"#;

impl OpenGlInterface {
    pub fn new() -> OpenGlInterface {
        // glfw: initialize and configure
        // ------------------------------
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        // glfw window creation
        // --------------------
        let (mut window, events) = glfw
            .create_window(800, 600, "LearnOpenGL", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        window.focus();
        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        // Load up vertexes
        // Define "window" boundaries
        const X_MIN: f32 = -0.75;
        const X_MAX: f32 = 0.75;
        const Y_MIN: f32 = -0.75;
        const Y_MAX: f32 = 0.75;
        // Generate 4 vertices for each pixel (2 triangles)
        let vertices: [f32; 3 * 4 * 64 * 32] = array::from_fn(|i| {
            const PIXEL: [[f32; 3]; 4] = [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ];
            let x = (i / (3 * 4)) % 64;
            let y = (i / (3 * 4)) / 64;
            let pixel_i = i % 12;
            let vertex = PIXEL[pixel_i / 3];
            return match i % 3 {
                0 => X_MIN + (x as f32 + vertex[0]) / 64.0 * (X_MAX - X_MIN),
                1 => Y_MIN + (y as f32 + vertex[1]) / 32.0 * (Y_MAX - Y_MIN),
                2 => 0 as f32,
                _ => 0 as f32,
            };
        });

        // Generate indices linking those 4 vertices into 2 triangles
        let indices: [i32; 6 * 64 * 32] = array::from_fn(|i| {
            let pixel = i / 6;
            return ((4 * pixel) + [0, 1, 2, 0, 2, 3][i % 6]) as i32;
        });

        // Default color is just white
        let colors: [f32; 3 * 4 * 65 * 33] = array::from_fn(|i| {
            return 1.0 as f32;
        });

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let mut cbo = 0;
        unsafe {
            let vertex_shader = compile_shader(VERTEX_SHADER, gl::VERTEX_SHADER);
            let fragment_shader = compile_shader(FRAGMENT_SHADER, gl::FRAGMENT_SHADER);

            let program = create_program(vec![vertex_shader, fragment_shader]);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::GenBuffers(1, &mut cbo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as isize,
                &vertices[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                3 * mem::size_of::<GLfloat>() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<GLfloat>()) as isize,
                &indices[0] as *const i32 as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, cbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (colors.len() * mem::size_of::<GLfloat>()) as isize,
                &colors[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
            let name = "color";
            let c_name = CString::new(name.as_bytes()).unwrap();
            let color_idx = gl::GetAttribLocation(program, c_name.as_ptr()) as u32;
            gl::EnableVertexAttribArray(color_idx);
            gl::VertexAttribPointer(color_idx, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(program);
            gl::BindVertexArray(vao);

            gl::Disable(gl::DITHER);
            gl::Disable(gl::LINE_SMOOTH);
            gl::Disable(gl::POLYGON_SMOOTH);
            gl::Hint(gl::POLYGON_SMOOTH_HINT, gl::DONT_CARE);
            gl::Hint(gl::LINE_SMOOTH_HINT, gl::DONT_CARE);
        }

        OpenGlInterface {
            glfw,
            events,
            window,
            color: [1.0, 0.0, 0.0],
            cbo: cbo,
        }
    }
}

impl Interface for OpenGlInterface {
    fn exit(&mut self) {}
    fn update_inputs(&mut self, p: &mut Processor) -> bool {
        // Check for events
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // make sure the viewport matches the new window dimensions; note that width and
                    // height will be significantly larger than specified on retina displays.
                    unsafe { gl::Viewport(0, 0, width, height) }
                }
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    self.window.set_should_close(true);
                }
                _ => {}
            }
        }
        return self.window.should_close();
    }
    fn render(&mut self, p: &Processor) {
        // render
        // ------
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Calculate colors of each vertex
            // 3 floats per color * 4 colors/vertices per pixel
            let colors: [f32; 3 * 4 * 64 * 32] = array::from_fn(|i| {
                let x = (i / 3 / 4) % 64;
                let y = (i / 3 / 4) / 64;
                // Our screen starts at the top left
                return if p.get_pixel_at(x as u8, 64 - y as u8) {
                    1
                } else {
                    0
                } as f32;
            });

            // Refresh color buffer
            gl::BindBuffer(gl::ARRAY_BUFFER, self.cbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (colors.len() * mem::size_of::<GLfloat>()) as isize,
                &colors[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );

            // Draw pixels
            gl::DrawElements(
                gl::TRIANGLES,
                2 * 3 * 64 * 32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }

        // Refresh page
        self.window.swap_buffers();
    }
}

unsafe fn compile_shader(shader_src: &str, shader_type: u32) -> u32 {
    let shader = gl::CreateShader(shader_type);
    let c_pointer = CString::new(shader_src.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_pointer.as_ptr(), ptr::null());
    gl::CompileShader(shader);
    // Check for success
    let mut success = gl::FALSE as GLint;
    let mut info_log = Vec::with_capacity(512);
    info_log.set_len(512 - 1);
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if success != gl::TRUE as GLint {
        gl::GetShaderInfoLog(
            shader,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        println!("Error compiling shader! {:?}", String::from_utf8(info_log));
    }

    return shader;
}

unsafe fn create_program(shaders: Vec<u32>) -> u32 {
    let program = gl::CreateProgram();
    println!("{:?}", shaders);
    gl::AttachShader(program, shaders[0]);
    gl::AttachShader(program, shaders[1]);
    // for shader in shaders {
    //     gl::AttachShader(program, shader);
    // }
    gl::LinkProgram(program);

    // Check for success
    let mut success = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

    if success != gl::TRUE as GLint {
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetProgramInfoLog(
            program,
            512,
            ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
        println!(
            "Error creating program! {}",
            String::from_utf8(info_log).unwrap()
        );
    }

    return program;
}
