use std::error::Error;

use crate::utils;
use glow::{
    Context, HasContext, NativeBuffer, NativeFramebuffer, NativeProgram, NativeRenderbuffer,
    NativeShader, NativeTexture, NativeVertexArray,
};

macro_rules! check_error {
    ($gl: ident) => {{
        let e = $gl.get_error();
        if e != glow::NO_ERROR {
            panic!("GL error: {:X}", e);
        }
    }};
}

pub unsafe fn setup_framebuffer(gl: &Context) -> NativeFramebuffer {
    const SAMPLES: i32 = 4;
    let fbo = gl.create_framebuffer().unwrap();
    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
    let tex = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D_MULTISAMPLE, Some(tex));
    gl.tex_image_2d_multisample(
        glow::TEXTURE_2D_MULTISAMPLE,
        SAMPLES,
        glow::RGB as i32,
        800,
        600,
        true,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    check_error!(gl);
    gl.framebuffer_texture_2d(
        glow::FRAMEBUFFER,
        glow::COLOR_ATTACHMENT0,
        glow::TEXTURE_2D_MULTISAMPLE,
        Some(tex),
        0,
    );
    check_error!(gl);

    let rbo = gl.create_renderbuffer().unwrap();
    gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
    gl.renderbuffer_storage_multisample(
        glow::RENDERBUFFER,
        SAMPLES,
        glow::DEPTH24_STENCIL8,
        800,
        600,
    );
    gl.bind_renderbuffer(glow::RENDERBUFFER, None);

    gl.framebuffer_renderbuffer(
        glow::FRAMEBUFFER,
        glow::DEPTH_STENCIL_ATTACHMENT,
        glow::RENDERBUFFER,
        Some(rbo),
    );
    if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
        panic!("Unable to attach renderbuffer to framebuffer");
    }
    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
    fbo
}

// Create an openGL program with the vertex and fragment shader provided
pub unsafe fn create_program(gl: &Context, vertex_src: &str, frag_src: &str) -> NativeProgram {
    let program = gl.create_program().unwrap();

    let v = create_shader(gl, program, glow::VERTEX_SHADER, vertex_src).unwrap();
    let f = create_shader(gl, program, glow::FRAGMENT_SHADER, frag_src).unwrap();
    check_error!(gl);
    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!(
            "Unable to link shader: {}",
            gl.get_program_info_log(program)
        );
    }
    delete_shader(gl, program, v);
    delete_shader(gl, program, f);

    program
}
pub unsafe fn create_texture(gl: &Context, size: (usize, usize), data: &[u8]) -> NativeTexture {
    println!("Creating a texture {:?}", size);
    let tex = gl.create_texture().unwrap();
    gl.bind_texture(glow::TEXTURE_2D, Some(tex));
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
    gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MIN_FILTER,
        glow::LINEAR as i32,
    );
    gl.tex_parameter_i32(
        glow::TEXTURE_2D,
        glow::TEXTURE_MAG_FILTER,
        glow::LINEAR as i32,
    );
    check_error!(gl);
    gl.tex_image_2d(
        glow::TEXTURE_2D,
        0,
        glow::RGB as i32,
        size.0 as i32,
        size.1 as i32,
        0,
        glow::RGB,
        glow::UNSIGNED_BYTE,
        Some(data),
    );
    check_error!(gl);
    tex
}

pub unsafe fn create_shader(
    gl: &Context,
    program: NativeProgram,
    shader_type: u32,
    shader_src: &str,
) -> Result<NativeShader, Box<dyn Error>> {
    let shader = gl.create_shader(shader_type)?;
    gl.shader_source(shader, shader_src);
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        return Err(format!(
            "Unable to compile shader {}: {}",
            shader_src,
            gl.get_shader_info_log(shader),
        )
        .into());
    }
    gl.attach_shader(program, shader);
    check_error!(gl);

    Ok(shader)
}
pub unsafe fn delete_shader(gl: &Context, program: NativeProgram, shader: NativeShader) {
    gl.detach_shader(program, shader);
    gl.delete_shader(shader);
}

pub unsafe fn set_uniforms(gl: &Context, program: NativeProgram, dur: f32, rot: (f32, f32, f32)) {
    let (a, b, c) = rot;
    gl.uniform_matrix_3_f32_slice(
        gl.get_uniform_location(program, "rotation").as_ref(),
        false,
        [
            [
                a.cos() * b.cos(),
                a.cos() * b.sin() * c.sin() - a.sin() * c.cos(),
                a.cos() * b.sin() * c.cos() + a.sin() * c.sin(),
            ],
            [
                a.sin() * b.cos(),
                a.sin() * b.sin() * c.sin() + a.cos() * c.cos(),
                a.sin() * b.sin() * c.cos() - a.cos() * c.sin(),
            ],
            [-b.sin(), b.cos() * c.sin(), b.cos() * c.cos()],
        ]
        .as_flattened(),
    );

    let translation = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.5],
        [0.0, 0.0, 0.0, 1.0],
    ];
    gl.uniform_matrix_4_f32_slice(
        gl.get_uniform_location(program, "translation").as_ref(),
        true,
        translation.as_flattened(),
    );

    let s = dur.sin() * 0.5 + 1.0;
    let scale = [[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]];
    gl.uniform_matrix_3_f32_slice(
        gl.get_uniform_location(program, "scale").as_ref(),
        false,
        scale.as_flattened(),
    );

    gl.uniform_3_f32(
        gl.get_uniform_location(program, "lightPosition").as_ref(),
        0.0,
        2.0,
        0.0,
    );
}

const STRIDE: usize = 8;
pub unsafe fn load_mesh(
    gl: &Context,
    data: &[[f32; STRIDE]],
) -> Result<(NativeVertexArray, NativeBuffer), Box<dyn Error>> {
    // Turn raw data of f32 slice into u8 slice to transfer to open gl
    let data = transform_data(data.as_flattened());
    // Create buffer
    let vbo = gl.create_buffer()?;
    check_error!(gl);
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    check_error!(gl);
    // Buffer data
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);
    check_error!(gl);

    let vao = gl.create_vertex_array()?;
    gl.bind_vertex_array(Some(vao));
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(
        0,
        3,
        glow::FLOAT,
        false,
        std::mem::size_of::<f32>() as i32 * STRIDE as i32,
        0,
    );
    check_error!(gl);
    gl.enable_vertex_attrib_array(1);
    gl.vertex_attrib_pointer_f32(
        1,
        2,
        glow::FLOAT,
        false,
        std::mem::size_of::<f32>() as i32 * STRIDE as i32,
        std::mem::size_of::<f32>() as i32 * 6,
    );
    check_error!(gl);
    gl.enable_vertex_attrib_array(2);
    gl.vertex_attrib_pointer_f32(
        2,
        3,
        glow::FLOAT,
        false,
        std::mem::size_of::<f32>() as i32 * STRIDE as i32,
        std::mem::size_of::<f32>() as i32 * 3,
    );

    Ok((vao, vbo))
}
pub unsafe fn load_and_create_texture(gl: &Context, path: &str) -> NativeTexture {
    let (size, data) = utils::read_png_data(path);
    create_texture(gl, size, &data)
}

unsafe fn transform_data(data: &[f32]) -> &[u8] {
    core::slice::from_raw_parts(
        data.as_ptr() as *const u8,
        core::mem::size_of::<f32>() * data.len(),
    )
}
