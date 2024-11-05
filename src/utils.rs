use glow::{
    Context, HasContext, NativeFramebuffer, NativeProgram, NativeTexture, Program, VertexArray,
};
use sdl2::{
    video::{GLContext, Window},
    VideoSubsystem,
};

#[macro_export]
macro_rules! check_error {
    ($gl: expr, $str: expr) => {
        let e = $gl.get_error();
        if e != glow::NO_ERROR {
            panic!(
                "OpenGL error thrown (getError = {:X}, context = \"{}\")",
                e, $str
            );
        }
    };
    ($gl: expr) => {
        check_error!($gl, "None provided");
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
    // window
    //     .gl_make_current(&gl_context)
    //     .expect("Unable to make context current");

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

pub unsafe fn create_data_texture(gl: &Context, data: &[u8]) -> NativeTexture {
    println!("Creating texture with length {}", data.len());
    let data_tex = gl.create_texture().expect("Unable to create a Texture");
    check_error!(gl);
    set_data_texture_data(gl, &data_tex, data);
    data_tex
}
pub unsafe fn set_data_texture_data(gl: &Context, texture: &NativeTexture, data: &[u8]) {
    gl.bind_texture(glow::TEXTURE_1D, Some(*texture));
    check_error!(gl);
    gl.tex_image_1d(
        glow::TEXTURE_1D,
        0,
        glow::R8 as i32,
        data.len() as i32,
        0,
        glow::RED,
        glow::UNSIGNED_BYTE,
        Some(&data),
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_1D,
        glow::TEXTURE_MIN_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_1D,
        glow::TEXTURE_MAG_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
    gl.tex_parameter_i32(
        glow::TEXTURE_1D,
        glow::TEXTURE_MIN_FILTER,
        glow::NEAREST as i32,
    );
    check_error!(gl);
}

pub unsafe fn set_bool_uniform(
    gl: &glow::Context,
    program: &glow::Program,
    name: &str,
    value: bool,
) {
    let location = gl.get_uniform_location(*program, name);
    check_error!(gl, format!("Getting uniform {} location", name));
    gl.uniform_1_i32(location.as_ref(), if value { 1 } else { 0 });
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
) -> (NativeFramebuffer, VertexArray, NativeProgram) {
    println!("Creating screen buffer with size {:?}", size);
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
    (texture_buffer, vao, texture_program)
}
