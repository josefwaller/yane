use crate::Nes;
use glow::{
    Context, HasContext, NativeFramebuffer, NativeProgram, NativeTexture, NativeVertexArray,
    Program, VertexArray,
};
use log::*;
use sdl2::{
    video::{GLContext, Window},
    VideoSubsystem,
};

#[macro_export]
macro_rules! check_error {
    ($gl: expr, $str: expr) => {
        #[cfg(debug_assertions)]
        {
            let e = $gl.get_error();
            if e != glow::NO_ERROR {
                panic!(
                    "OpenGL error thrown (getError = {:X}, context = \"{}\")",
                    e, $str
                );
            }
        }
    };
    ($gl: expr) => {
        check_error!($gl, "None provided");
    };
}
#[macro_export]
macro_rules! set_uniform {
    ($gl: expr, $program: expr, $name: expr, $func: ident, $($vals: expr),+) => {
        let loc = $gl.get_uniform_location($program, $name);
        check_error!($gl, format!("Getting uniform location {}", $name));
        $gl.$func(loc.as_ref(), $($vals),*);
        check_error!(
            $gl,
            format!("Setting uniform {} value to 0x{:X?}", $name, ($($vals),*))
        );
    };
}

pub fn create_window(
    video: &VideoSubsystem,
    title: &str,
    window_width: u32,
    window_height: u32,
) -> (Window, GLContext, Context) {
    let window = video
        .window(title, window_width, window_height)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();

    // Init screen and audio
    let gl = unsafe {
        glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
    };

    (window, gl_context, gl)
}
pub unsafe fn compile_and_link_shader(
    gl: &Context,
    shader_type: u32,
    shader_src: &str,
    program: &NativeProgram,
) {
    let shader = gl.create_shader(shader_type).expect("Cannot create shader");
    check_error!(gl);
    gl.shader_source(shader, shader_src);
    check_error!(gl);
    gl.compile_shader(shader);
    check_error!(gl);
    if !gl.get_shader_compile_status(shader) {
        panic!(
            "Failed to compile shader with source {}: {}",
            shader_src,
            gl.get_shader_info_log(shader)
        );
    }
    gl.attach_shader(*program, shader);
    check_error!(gl);
    gl.delete_shader(shader);
    check_error!(gl);
}
pub unsafe fn create_program(
    gl: &Context,
    vertex_src: &'static str,
    frag_src: &'static str,
) -> NativeProgram {
    let program = gl.create_program().unwrap();
    compile_and_link_shader(&gl, glow::VERTEX_SHADER, vertex_src, &program);
    compile_and_link_shader(&gl, glow::FRAGMENT_SHADER, frag_src, &program);
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!(
            "Couldn't link program: {}",
            gl.get_program_info_log(program)
        );
    }
    program
}

/// Set the texture given to the CHR ROM/RAM in the nes given
pub unsafe fn refresh_chr_texture(gl: &Context, chr_tex: NativeTexture, nes: &Nes, chr: Vec<u8>) {
    let texture_data: Vec<u8> = chr
        .chunks(16)
        .map(|sprite_data| {
            (0..(8 * 8)).map(|i| {
                let less_sig = (sprite_data[i / 8] >> (7 - i % 8)) & 0x01;
                let more_sig = (sprite_data[i / 8 + 8] >> (7 - i % 8)) & 0x01;
                2 * more_sig + less_sig
            })
        })
        .flatten()
        .collect();
    gl.bind_texture(glow::TEXTURE_2D, Some(chr_tex));
    check_error!(gl);
    // Generate a texture 8 pixels long to use for the CHR ROM/RAM
    let width = 8;
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::R8 as i32,
        width,
        texture_data.len() as i32 / width,
        0,
        glow::RED,
        glow::UNSIGNED_BYTE,
        Some(&texture_data),
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
}

pub unsafe fn set_int_uniform(gl: &glow::Context, program: &glow::Program, name: &str, value: i32) {
    let location = gl.get_uniform_location(*program, name);
    check_error!(gl);
    gl.uniform_1_i32(location.as_ref(), value);
    check_error!(
        gl,
        format!(
            "Setting int uniform {} at location {:?} to value {}",
            name, location, value
        )
    );
}
pub unsafe fn create_f32_slice_vao(gl: &Context, verts: &[f32], element_size: i32) -> VertexArray {
    let buf = gl.create_buffer().unwrap();
    check_error!(gl);
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
    check_error!(gl);
    let verts_u8 =
        core::slice::from_raw_parts(verts.as_ptr() as *const u8, verts.len() * size_of::<f32>());
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &verts_u8, glow::STATIC_DRAW);
    check_error!(gl);
    // Describe the format of the data
    let vao = gl.create_vertex_array().unwrap();
    check_error!(gl);
    gl.bind_vertex_array(Some(vao));
    gl.enable_vertex_attrib_array(0);
    check_error!(gl);
    gl.vertex_attrib_pointer_f32(
        0,
        element_size,
        glow::FLOAT,
        false,
        element_size * size_of::<f32>() as i32,
        0,
    );
    check_error!(gl);
    vao
}
pub unsafe fn buffer_data_slice(gl: &Context, program: &Program, data: &[i32]) -> VertexArray {
    gl.use_program(Some(*program));
    check_error!(gl);
    let vao = gl
        .create_vertex_array()
        .expect("Could not create VertexArray");
    check_error!(gl);
    gl.bind_vertex_array(Some(vao));
    check_error!(gl);
    // Pipe data
    let data_u8: &[u8] =
        core::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * size_of::<i32>());
    let vbo = gl.create_buffer().unwrap();
    check_error!(gl);
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    check_error!(gl);
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data_u8, glow::STATIC_DRAW);
    check_error!(gl);
    // Describe the format of the data
    gl.enable_vertex_attrib_array(0);
    check_error!(gl);
    gl.vertex_attrib_pointer_i32(0, 1 as i32, glow::INT, 4, 0);
    check_error!(gl);
    // Unbind
    gl.bind_vertex_array(None);
    check_error!(gl);

    vao
}

pub unsafe fn create_screen_texture(
    gl: &Context,
    size: (usize, usize),
) -> (NativeFramebuffer, VertexArray, NativeProgram, NativeTexture) {
    info!("Creating screen buffer with size {:?}", size);
    check_error!(gl);
    let texture_program = gl.create_program().unwrap();
    check_error!(gl);
    gl.link_program(texture_program);
    check_error!(gl);
    gl.use_program(Some(texture_program));
    check_error!(gl);
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
    check_error!(gl);
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(tex_buf));
    check_error!(gl);
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &quad_verts_u8, glow::STATIC_DRAW);
    check_error!(gl);

    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);

    gl.link_program(texture_program);

    let texture_buffer = gl.create_framebuffer().unwrap();
    let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if status != glow::FRAMEBUFFER_COMPLETE {
        panic!("Error creating frame buffer: {:X}", status);
    }
    check_error!(gl);
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(texture_buffer));
    check_error!(gl);
    let render_texture = gl.create_texture().unwrap();
    check_error!(gl);
    gl.bind_texture(glow::TEXTURE_2D, Some(render_texture));
    check_error!(gl);
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGBA as i32,
        size.0 as i32,
        size.1 as i32,
        0,
        glow::RGBA,
        glow::UNSIGNED_BYTE,
        None,
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
    gl.framebuffer_texture(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        Some(render_texture),
        0,
    );
    check_error!(gl);
    gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);
    check_error!(gl);
    let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if status != glow::FRAMEBUFFER_COMPLETE {
        panic!("Error creating frame buffer: {:X}", status);
    }
    let depth_stencil_tex = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D, Some(depth_stencil_tex));
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::DEPTH24_STENCIL8 as i32,
        size.0 as i32,
        size.1 as i32,
        0,
        glow::DEPTH_STENCIL,
        glow::UNSIGNED_INT_24_8,
        None,
    );
    check_error!(gl);
    // Add a stencil and depth attachment
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::DEPTH_STENCIL_ATTACHMENT,
        glow::TEXTURE_2D,
        Some(depth_stencil_tex),
        0,
    );
    check_error!(gl);
    let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
    if status != glow::FRAMEBUFFER_COMPLETE {
        panic!("Error creating frame buffer: {:X}", status);
    }
    (texture_buffer, vao, texture_program, render_texture)
}
