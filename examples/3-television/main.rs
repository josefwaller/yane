use glow::HasContext;
use sdl2::{keyboard::Scancode, video::GLProfile};
use std::{
    f32::consts::PI,
    fs::File,
    io::Read,
    path::Path,
    thread::sleep,
    time::{Duration, Instant},
};
use yane::{
    app::{Audio, Config, Input},
    core::{Cartridge, Nes, Settings, CPU_CLOCK_SPEED},
};

mod open_gl;
mod utils;

fn main() {
    // Read iNes file
    let ines_contents: Vec<u8> = File::open(
        std::env::args()
            .collect::<Vec<String>>()
            .get(1)
            .expect("Please provide an iNes file to run")
            .clone(),
    )
    .expect("Unable to open file")
    .bytes()
    .map(|b| b.unwrap())
    .collect();
    // Initialise SDL
    let sdl = sdl2::init().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();
    // Set up NES
    let mut nes = Nes::with_cartridge(Cartridge::from_ines(&ines_contents, None));
    let mut input = Input::new();
    let mut audio = Audio::from_sdl_audio(&sdl.audio().unwrap());
    // Spawn OpenGL window
    let window = video
        .window("Example 3 - television", 800, 600)
        .opengl()
        .build()
        .unwrap();
    let _gl_context = window.gl_create_context().unwrap();
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
    };
    // Read TV and screen meshes
    let tv = utils::read_obj(Path::new("examples/3-television/assets/retro_tv.obj"));
    let screen = utils::read_obj(Path::new("examples/3-television/assets/screen.obj"));
    unsafe {
        // Create program
        let program = open_gl::create_program(
            &gl,
            include_str!("./shaders/vertex.glsl"),
            include_str!("./shaders/frag.glsl"),
        );
        // Create framebuffer
        let fbo = open_gl::setup_framebuffer(&gl);
        // Create TV and screen VAO/VBOs
        let (tv_vao, tv_vbo) = open_gl::load_mesh(&gl, &tv).expect("Unable create VAO and VBO");
        let (screen_vao, screen_vbo) = open_gl::load_mesh(&gl, &screen).unwrap();
        // Load textures
        let tex = open_gl::load_and_create_texture(&gl, "examples/3-television/assets/texture.png");
        // Create an empty texture for the screen
        let screen_tex = open_gl::create_texture(&gl, (0, 0), &[]);

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.enable(glow::MULTISAMPLE);
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LEQUAL);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.uniform_1_i32(gl.get_uniform_location(program, "tex").as_ref(), 0 as i32);

        let start_time = Instant::now();
        let mut total_cycles = 0;
        let mut rot = (0.0, PI / 2.0, 0.0);
        while !event_pump.poll_iter().any(|e| match e {
            sdl2::event::Event::Quit { .. } => true,
            _ => false,
        }) {
            // Update TV rotation
            const ROT_SPEED: f32 = 0.05;
            let keys: Vec<Scancode> = event_pump.keyboard_state().pressed_scancodes().collect();
            if keys.contains(&Scancode::Left) {
                rot.1 -= ROT_SPEED;
            } else if keys.contains(&Scancode::Right) {
                rot.1 += ROT_SPEED;
            }
            if keys.contains(&Scancode::Up) {
                rot.0 -= ROT_SPEED;
            } else if keys.contains(&Scancode::Down) {
                rot.0 += ROT_SPEED;
            }
            if keys.contains(&Scancode::Period) {
                rot.2 -= ROT_SPEED;
            } else if keys.contains(&Scancode::Comma) {
                rot.2 += ROT_SPEED;
            }
            // Update NES
            input.update(&mut nes, &event_pump, &mut Config::default());
            audio.update(&mut nes, &Config::default());
            total_cycles += nes.advance_frame(&Settings::default()).unwrap();
            // Set screen texture
            // Since the NES's output is already in RGB values, we can just pass it almost directly to open gl
            gl.bind_texture(glow::TEXTURE_2D, Some(screen_tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                256,
                240,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(nes.ppu.rgb_output().as_flattened().as_flattened()),
            );
            let dur = (Instant::now() - start_time).as_millis() as f32 / 5_000.0;

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // Render TV
            gl.use_program(Some(program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            open_gl::set_uniforms(&gl, program, dur, rot);
            gl.uniform_1_f32(gl.get_uniform_location(program, "ambient").as_ref(), 0.25);
            gl.uniform_1_f32(
                gl.get_uniform_location(program, "lightStrength").as_ref(),
                0.5,
            );
            gl.bind_vertex_array(Some(tv_vao));
            gl.draw_arrays(glow::TRIANGLES, 0, tv.len() as i32);
            // Render screen
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(screen_tex));
            // Set ambient light to more intense to make it appear as though screen is glowing
            gl.uniform_1_f32(gl.get_uniform_location(program, "ambient").as_ref(), 1.0);
            gl.uniform_1_f32(
                gl.get_uniform_location(program, "lightStrength").as_ref(),
                0.0,
            );
            gl.bind_vertex_array(Some(screen_vao));
            gl.draw_arrays(glow::TRIANGLES, 0, screen.len() as i32);

            // Render framebuffer to screen
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fbo));
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            gl.blit_framebuffer(
                0,
                0,
                800,
                600,
                0,
                0,
                800,
                600,
                glow::COLOR_BUFFER_BIT,
                glow::LINEAR,
            );

            window.gl_swap_window();

            // Check if we need to slow down
            let emu_elapsed = Duration::from_nanos(
                (total_cycles as f64 * 1_000_000_000.0 / CPU_CLOCK_SPEED as f64).floor() as u64,
            );
            let actual_elapsed = Instant::now().duration_since(start_time);
            if emu_elapsed > actual_elapsed {
                sleep(emu_elapsed - actual_elapsed);
            }
        }
        // Cleanup
        gl.delete_program(program);
        gl.delete_vertex_array(tv_vao);
        gl.delete_buffer(tv_vbo);
        gl.delete_vertex_array(screen_vao);
        gl.delete_buffer(screen_vbo);
        gl.delete_framebuffer(fbo);
    }
}
