use crate::{
    apu::{PulseRegister, TriangleRegister},
    Controller, Nes,
};
use glow::*;
use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    event::Event::Quit,
    keyboard::Keycode,
};

use std::mem::size_of;

const DUTY_CYCLES: [[f32; 8]; 4] = [
    [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0],
];
#[derive(Clone, Copy)]
struct PulseWave {
    // The pulse register this device is attached to
    register: PulseRegister,
    // How many sampels are taken every second, i.e. how many times callback is called per second
    samples_per_second: i32,
    // The phase of the wave, i.e. the progress through a single wave
    phase: f32,
}
impl AudioCallback for PulseWave {
    type Channel = f32;

    // Ouputs the amplitude
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            // Conditions for register being disabled
            const MAX_VOLUME: f32 = 0.05;
            let duty_cycle = DUTY_CYCLES[self.register.duty as usize];
            // Should be between 0 and 1
            let volume = if self.register.constant_volume {
                self.register.volume as f32 / 0xF as f32
            } else {
                self.register.volume_decay as f32 / 0xF as f32
            };
            // Output no sound if muted one way or another
            if !self.register.enabled
                || self.register.length_counter == 0
                || self.register.timer < 8
                || self.register.sweep_target_period > 0x7FF
            {
                *x = 0.0;
            } else {
                *x = MAX_VOLUME
                    * (1.0
                        - 2.0
                            * duty_cycle[(self.phase * duty_cycle.len() as f32).floor() as usize
                                % duty_cycle.len()])
                    * volume;
            }
            // Advance phase
            let clock = 1_789_000.0;
            let freq = clock / (16.0 * (self.register.timer as f32 + 1.0));
            self.phase = (self.phase + (freq / self.samples_per_second as f32)) % 1.0;
        }
    }
}

#[derive(Clone, Copy)]
struct TriangleWave {
    register: TriangleRegister,
    phase: f32,
    sample_rate: i32,
}
impl AudioCallback for TriangleWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let amp = if self.phase < 0.5 {
                self.phase * 2.0
            } else {
                1.0 - (self.phase - 0.5) * 2.0
            };
            if !self.register.enabled
                || self.register.length == 0
                || self.register.linear_counter == 0
            {
                *x = 0.0;
            } else {
                *x = 0.25 * (2.0 * amp - 1.0);
            }
            let freq = 1_789_000.0 / (32.0 * (self.register.timer + 1) as f32);
            self.phase = (self.phase + (freq / self.sample_rate as f32)) % 1.0;
        }
    }
}

pub struct Gui {
    gl: glow::Context,
    _gl_context: sdl2::video::GLContext,
    window: sdl2::video::Window,
    event_loop: sdl2::EventPump,
    sprite_program: NativeProgram,
    texture_program: NativeProgram,
    tex_vao: NativeVertexArray,
    vao_array: [NativeVertexArray; 64],
    texture_buffer: NativeFramebuffer,
    palette: [[f32; 3]; 0x40],
    background_program: NativeProgram,
    background_vao: NativeVertexArray,
    pulse_devices: [AudioDevice<PulseWave>; 2],
    triangle_device: AudioDevice<TriangleWave>,
}
impl Gui {
    // TODO: Rename
    pub fn new(nes: &Nes) -> Gui {
        let window_width = 256 * 3;
        let window_height = 240 * 3;
        unsafe {
            // Create SDL2 Window
            let sdl = sdl2::init().unwrap();
            // Setup video
            let video = sdl.video().unwrap();
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(3, 3);
            let window = video
                .window("Y.A.N.E", window_width, window_height)
                .opengl()
                .resizable()
                .build()
                .unwrap();
            let gl_context = window.gl_create_context().unwrap();
            let gl =
                glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);

            window.gl_make_current(&gl_context).unwrap();
            let event_loop = sdl.event_pump().unwrap();

            // Setup audio
            let audio = sdl.audio().unwrap();
            let spec = AudioSpecDesired {
                freq: Some(44_100),
                channels: Some(1),
                samples: None,
            };

            let pulse_devices: [AudioDevice<PulseWave>; 2] = core::array::from_fn(|i| {
                let device = audio
                    .open_playback(None, &spec, |spec| PulseWave {
                        register: nes.apu.pulse_registers[i],
                        samples_per_second: spec.freq,
                        phase: 0.0,
                    })
                    .unwrap();
                device.resume();
                device
            });
            let triangle_device = audio
                .open_playback(None, &spec, |spec| TriangleWave {
                    register: nes.apu.triangle_register,
                    sample_rate: spec.freq,
                    phase: 0.0,
                })
                .unwrap();
            triangle_device.resume();

            // Send CHR ROM data
            let data: &[u8] = nes.cartridge.chr_rom.as_slice();
            let chr_rom_tex = gl.create_texture().expect("Unable to create a Texture");
            gl.bind_texture(glow::TEXTURE_1D, Some(chr_rom_tex));
            gl.tex_image_1d(
                glow::TEXTURE_1D,
                0,
                glow::R8 as i32,
                nes.cartridge.chr_rom.len() as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                Some(&data),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_1D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_1D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_1D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );

            // Create program for rendering sprites to texture
            let sprite_program = gl.create_program().expect("Unable to create program");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./shaders/pass_through.vert"),
                &sprite_program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("./shaders/oam.geom"),
                &sprite_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./shaders/tile.frag"),
                &sprite_program,
            );

            gl.link_program(sprite_program);
            if !gl.get_program_link_status(sprite_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(sprite_program)
                );
            }

            let vao_array = core::array::from_fn(|i| {
                // Our "vertice" is a 1-D vector with the OAM ID in it
                let vertices = [i as i32];
                buffer_data_slice(&gl, &sprite_program, &vertices, 1)
            });

            let background_program = gl.create_program().expect("Unable to create program!");
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./shaders/pass_through.vert"),
                &background_program,
            );
            compile_and_link_shader(
                &gl,
                glow::GEOMETRY_SHADER,
                include_str!("./shaders/background.geom"),
                &background_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./shaders/tile.frag"),
                &background_program,
            );

            gl.link_program(background_program);
            if !gl.get_program_link_status(background_program) {
                panic!(
                    "Couldn't link program: {}",
                    gl.get_program_info_log(background_program)
                );
            }
            let verts: [i32; 32 * 60] = core::array::from_fn(|i| i as i32);
            let background_vao = buffer_data_slice(&gl, &background_program, &verts, 1);

            let texture_program = gl.create_program().unwrap();

            gl.link_program(texture_program);
            gl.use_program(Some(texture_program));
            compile_and_link_shader(
                &gl,
                glow::VERTEX_SHADER,
                include_str!("./shaders/quad_shader.vert"),
                &texture_program,
            );
            compile_and_link_shader(
                &gl,
                glow::FRAGMENT_SHADER,
                include_str!("./shaders/quad_shader.frag"),
                &texture_program,
            );

            let quad_verts: &[f32] = [
                [-1.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
            ]
            .as_flattened();
            let quad_verts_u8 = core::slice::from_raw_parts(
                quad_verts.as_ptr() as *const u8,
                quad_verts.len() * size_of::<f32>(),
            );
            let tex_buf = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(tex_buf));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &quad_verts_u8, glow::STATIC_DRAW);

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);

            gl.link_program(texture_program);
            let tex_vao = vao;

            let texture_buffer = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(texture_buffer));
            let render_texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(render_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                256,
                240,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.framebuffer_texture(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                Some(render_texture),
                0,
            );
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);
            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("Error creating frame buffer");
            }
            // Load pallete
            let palette_data: &[u8] = include_bytes!("./2C02G_wiki.pal");
            let palette: [[f32; 3]; 64] = core::array::from_fn(|i| {
                core::array::from_fn(|j| palette_data[3 * i + j] as f32 / 255.0)
            });

            let e = gl.get_error();
            if e != glow::NO_ERROR {
                panic!("Error generated sometime during initialization: {:X?}", e);
            }
            Gui {
                gl,
                window,
                event_loop,
                // This just needs to stay in scope
                _gl_context: gl_context,
                sprite_program,
                texture_program,
                vao_array,
                tex_vao,
                texture_buffer,
                palette,
                background_program,
                background_vao,
                pulse_devices,
                triangle_device,
            }
        }
    }
    pub fn set_input(&self, nes: &mut Nes) {
        // Update inputs
        let keys: Vec<Keycode> = self
            .event_loop
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();
        // P1
        let controller = Controller {
            up: keys.contains(&Keycode::W),
            left: keys.contains(&Keycode::A),
            right: keys.contains(&Keycode::D),
            down: keys.contains(&Keycode::S),
            a: keys.contains(&Keycode::Q),
            b: keys.contains(&Keycode::E),
            start: keys.contains(&Keycode::R),
            select: keys.contains(&Keycode::F),
        };
        nes.set_input(0, controller);
        // P2
        let controller = Controller {
            up: keys.contains(&Keycode::I),
            left: keys.contains(&Keycode::J),
            right: keys.contains(&Keycode::L),
            down: keys.contains(&Keycode::K),
            a: keys.contains(&Keycode::U),
            b: keys.contains(&Keycode::O),
            start: keys.contains(&Keycode::Y),
            select: keys.contains(&Keycode::H),
        };
        nes.set_input(1, controller);
    }
    pub fn update_audio(&mut self, nes: &Nes) {
        self.pulse_devices
            .iter_mut()
            .enumerate()
            .for_each(|(i, d)| d.lock().register = nes.apu.pulse_registers[i]);
        self.triangle_device.lock().register = nes.apu.triangle_register;
    }
    pub fn render(&mut self, nes: &Nes) -> bool {
        unsafe {
            self.gl.use_program(Some(self.sprite_program));
            self.gl
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.texture_buffer));
            // Set clear color
            let clear_color = self.palette[(nes.ppu.palette_ram[0] & 0x3F) as usize];
            self.gl
                .clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
            self.gl.viewport(0, 0, 256, 240);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            self.render_background(nes);
            if nes.ppu.is_sprite_enabled() {
                self.render_sprites(nes);
            }

            self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            self.gl.use_program(Some(self.texture_program));
            self.gl.viewport(
                0,
                0,
                self.window.size().0 as i32,
                self.window.size().1 as i32,
            );
            self.gl.bind_vertex_array(Some(self.tex_vao));
            self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
        for event in self.event_loop.poll_iter() {
            match event {
                Quit { .. } => {
                    self.pulse_devices.iter_mut().for_each(|t| t.pause());
                    self.triangle_device.pause();
                    return true;
                }
                _ => {}
            }
        }
        self.window.gl_swap_window();

        false
    }
    unsafe fn render_background(&mut self, nes: &Nes) {
        self.gl.use_program(Some(self.background_program));
        self.gl.bind_vertex_array(Some(self.background_vao));
        self.setup_render_uniforms(&self.background_program, nes);
        // Set nametable
        let nametable_uni = self
            .gl
            .get_uniform_location(self.background_program, "nametable");
        // Pack nametable tightly
        let n: Vec<i32> = nes
            .ppu
            .nametable_ram
            .chunks(4)
            .map(|b| {
                ((b[3] as i32) << 24) + ((b[2] as i32) << 16) + ((b[1] as i32) << 8) + b[0] as i32
            })
            .collect();
        self.gl
            .uniform_1_i32_slice(nametable_uni.as_ref(), n.as_slice());
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "backgroundPatternLocation",
            nes.ppu.get_background_pattern_table_addr() as i32,
        );
        set_bool_uniform(
            &self.gl,
            &self.background_program,
            "hideLeftmostBackground",
            nes.ppu.should_hide_leftmost_background(),
        );
        self.gl.draw_arrays(glow::POINTS, 0, 30 * 32);
    }

    unsafe fn render_sprites(&mut self, nes: &Nes) {
        self.gl.use_program(Some(self.sprite_program));
        // Pipe OAM data to GLSL
        let oam_uni = self.gl.get_uniform_location(self.sprite_program, "oamData");
        let oam_data: [u32; 4 * 64] = core::array::from_fn(|i| nes.ppu.oam[i] as u32);
        self.gl.uniform_1_u32_slice(oam_uni.as_ref(), &oam_data);
        self.setup_render_uniforms(&self.sprite_program, nes);
        // Set various flags
        set_bool_uniform(
            &self.gl,
            &self.sprite_program,
            "hide_left_sprites",
            nes.ppu.should_hide_leftmost_sprites(),
        );
        set_bool_uniform(
            &self.gl,
            &self.sprite_program,
            "tallSprites",
            nes.ppu.is_8x16_sprites(),
        );
        set_int_uniform(
            &self.gl,
            &self.sprite_program,
            "spritePatternLocation",
            nes.ppu.get_spr_pattern_table_addr() as i32,
        );
        // Draw sprites as points
        // GLSL Shaders add pixels to form the full 8x8 sprite
        self.vao_array.iter().for_each(|vao| {
            self.gl.bind_vertex_array(Some(*vao));
            self.gl.draw_arrays(glow::POINTS, 0, 1);
        });
    }

    unsafe fn setup_render_uniforms(&self, program: &NativeProgram, nes: &Nes) {
        // Set pallete
        let palette: Vec<i32> = nes.ppu.palette_ram.iter().map(|p| *p as i32).collect();
        let palette_uni = self.gl.get_uniform_location(*program, "palettes");
        self.gl
            .uniform_1_i32_slice(palette_uni.as_ref(), palette.as_slice());
        // Set colors
        let colors = self.palette.as_flattened();
        let color_uni = self.gl.get_uniform_location(*program, "colors");
        self.gl.uniform_3_f32_slice(color_uni.as_ref(), colors);
        // Set tint uniforms
        set_bool_uniform(&self.gl, program, "redTint", nes.ppu.is_red_tint_on());
        set_bool_uniform(&self.gl, program, "blueTint", nes.ppu.is_blue_tint_on());
        set_bool_uniform(&self.gl, program, "greenTint", nes.ppu.is_green_tint_on());
        // Set greyscale mode
        set_bool_uniform(
            &self.gl,
            program,
            "greyscaleMode",
            nes.ppu.is_greyscale_mode_on(),
        );
        // Set scroll
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "scrollX",
            0, //nes.ppu.scroll_x as i32,
        );
        set_int_uniform(
            &self.gl,
            &self.background_program,
            "scrollY",
            0, //nes.ppu.scroll_y as i32,
        );
    }
}

unsafe fn compile_and_link_shader(
    gl: &Context,
    shader_type: u32,
    shader_src: &str,
    program: &NativeProgram,
) {
    let shader = gl.create_shader(shader_type).expect("Cannot create shader");
    gl.shader_source(shader, shader_src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        panic!(
            "Failed to compile shader with source {}: {}",
            shader_src,
            gl.get_shader_info_log(shader)
        );
    }
    gl.attach_shader(*program, shader);
    gl.delete_shader(shader);
}

unsafe fn set_bool_uniform(gl: &glow::Context, program: &glow::Program, name: &str, value: bool) {
    let location = gl.get_uniform_location(*program, name);
    gl.uniform_1_i32(location.as_ref(), if value { 1 } else { 0 });
}
unsafe fn set_int_uniform(gl: &glow::Context, program: &glow::Program, name: &str, value: i32) {
    let location = gl.get_uniform_location(*program, name);
    gl.uniform_1_i32(location.as_ref(), value);
}
unsafe fn buffer_data_slice(
    gl: &Context,
    program: &Program,
    data: &[i32],
    count: i32,
) -> VertexArray {
    gl.use_program(Some(*program));
    let vao = gl
        .create_vertex_array()
        .expect("Could not create VertexArray");
    gl.bind_vertex_array(Some(vao));
    // Pipe data
    let data_u8: &[u8] =
        core::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<i32>());
    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_u8, glow::STATIC_DRAW);
    // Describe the format of the data
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_i32(0, 1 as i32, glow::INT, 4, 0);
    // Unbind
    gl.bind_vertex_array(None);

    vao
}
