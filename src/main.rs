#[cfg(feature = "sdl")]
mod run_app;

fn main() {
    #[cfg(feature = "sdl")]
    run_app::run();
    #[cfg(not(feature = "sdl"))]
    println!("The \"sdl\" feature is not enabled. It must be enabled to run yane as an app.")
}
