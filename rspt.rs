extern mod nalgebra;
extern mod extra;

pub mod image;
pub mod main;
pub mod scene;
pub mod camera;
pub mod random;
pub mod sdlui;
pub mod obj;
pub mod aabb;

#[start]
fn start(argc: int, argv: **u8, crate_map: *u8) -> int {
    std::rt::start_on_main_thread(argc, argv, crate_map, main::entrypoint)
}
