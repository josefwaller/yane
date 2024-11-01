use glow::{Context, HasContext, NativeProgram, NativeTexture, Program, VertexArray};
use sdl2::{
    video::{GLContext, Window},
    VideoSubsystem,
};

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

pub unsafe fn create_data_texture(gl: &Context, data: &[u8]) -> NativeTexture {
    let data_tex = gl.create_texture().expect("Unable to create a Texture");
    gl.bind_texture(glow::TEXTURE_1D, Some(data_tex));
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
    data_tex
}

pub unsafe fn set_bool_uniform(
    gl: &glow::Context,
    program: &glow::Program,
    name: &str,
    value: bool,
) {
    let location = gl.get_uniform_location(*program, name);
    gl.uniform_1_i32(location.as_ref(), if value { 1 } else { 0 });
}
pub unsafe fn set_int_uniform(gl: &glow::Context, program: &glow::Program, name: &str, value: i32) {
    let location = gl.get_uniform_location(*program, name);
    gl.uniform_1_i32(location.as_ref(), value);
}
pub unsafe fn buffer_data_slice(gl: &Context, program: &Program, data: &[i32]) -> VertexArray {
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
