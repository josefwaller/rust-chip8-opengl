use crate::interfaces::Interface;
use crate::processor::Processor;

extern crate rodio;

use rodio::{source::SineWave, OutputStream, Sink};

use gl::types::{GLchar, GLfloat, GLint, GLsizei, GLuint};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use std::{array, ffi::CString, mem, os::raw::c_void, ptr};

pub struct OpenGlInterface {
    glfw: Glfw,
    events: GlfwReceiver<(f64, WindowEvent)>,
    window: PWindow,
    cbo: GLuint,
    input_states: [bool; 0x10],
    sink: Option<Sink>,
    // Stream just needs to be kept in scope
    #[allow(dead_code)]
    stream: Option<OutputStream>,
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

/**
 * Interface that uses OpenGL to render the processor.
 */
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
            .create_window(800, 600, "CHIP-8", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        window.focus();
        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        // Load up vertexes
        // Define "window" boundaries
        const X_MIN: f32 = -1.0;
        const X_MAX: f32 = 1.0;
        const Y_MIN: f32 = -1.0;
        const Y_MAX: f32 = 1.0;
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

        // Default color is just black
        let colors: [f32; 3 * 4 * 64 * 32] = array::from_fn(|_| {
            return 0.0 as f32;
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

        OpenGlInterface {
            glfw,
            events,
            window,
            cbo: cbo,
            input_states: [false; 0x10],
            sink,
            stream: device.and_then(|d| Some(d.0)),
        }
    }
}

impl Interface for OpenGlInterface {
    fn exit(&mut self) {}
    fn update(&mut self, p: &mut Processor) -> bool {
        let key_map = [
            glfw::Key::X,
            glfw::Key::Num1,
            glfw::Key::Num2,
            glfw::Key::Num3,
            glfw::Key::Q,
            glfw::Key::W,
            glfw::Key::E,
            glfw::Key::A,
            glfw::Key::S,
            glfw::Key::D,
            glfw::Key::Z,
            glfw::Key::C,
            glfw::Key::Num4,
            glfw::Key::R,
            glfw::Key::F,
            glfw::Key::V,
        ];
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
                glfw::WindowEvent::Key(key, _, action, _) => {
                    match key_map.iter().position(|k| *k == key) {
                        Some(i) => {
                            self.input_states[i] = match action {
                                glfw::Action::Press => true,
                                glfw::Action::Release => false,
                                _ => self.input_states[i],
                            };
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        p.update_inputs(self.input_states);
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
        return self.window.should_close();
    }
    fn render(&mut self, p: &Processor) {
        // render
        // ------
        unsafe {
            // Calculate colors of each vertex
            // 3 floats per color * 4 colors/vertices per pixel
            let mut colors: [f32; 12 * 32 * 64] = [0.0; 12 * 32 * 64];
            for y in 0..32 as usize {
                for x in 0..64 as usize {
                    let c = if p.get_pixel_at(x as u8, 63 - y as u8) {
                        1
                    } else {
                        0
                    } as f32;
                    colors[(12 * (x + 64 * y))..(12 * (x + 64 * y + 1))].copy_from_slice(&[c; 12]);
                }
            }

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
        panic!("Error compiling shader! {:?}", String::from_utf8(info_log));
    }

    return shader;
}

unsafe fn create_program(shaders: Vec<u32>) -> u32 {
    let program = gl::CreateProgram();
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
        panic!(
            "Error creating program! {}",
            String::from_utf8(info_log).unwrap()
        );
    }

    return program;
}
