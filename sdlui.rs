use std::libc;
use std::vec;
use image::RGB;
use main::RenderOptions;
use std::unstable;

struct SDL_Window;
struct SDL_Surface {
    flags: u32,
    pixf: *(),
    w: libc::c_int,
    h: libc::c_int,
    pitch: libc::c_int,
    pixels: *u8
}
struct SDL_Event {
    etype: u32,
    padding: [u8, ..56]
}

#[link_args="-lSDL2"]
extern {
    fn SDL_Init(what: u32) -> libc::c_int;
    fn SDL_CreateWindow(name: *i8, x: libc::c_int, y: libc::c_int, w: libc::c_int, h: libc::c_int,
                        flags: u32) -> *SDL_Window;
    fn SDL_GetWindowSurface(w: *SDL_Window) -> *SDL_Surface;
    fn SDL_UpdateWindowSurface(w: *SDL_Window) -> libc::c_int;
    fn SDL_LockSurface(s: *SDL_Surface);
    fn SDL_UnlockSurface(s: *SDL_Surface);
    fn SDL_PollEvent(ev: *SDL_Event) -> libc::c_int;
}

pub struct UI {
    width: uint, height: uint,
    w: *SDL_Window
}

fn f2i(x: float) -> u32 {
    let mut x = x;
    if x > 1.0 { x = 1.0 }
    (x * 255.0) as u32
}

impl UI {
    #[fixed_stack_segment]
    pub fn new(opts: &RenderOptions) -> UI {
        unsafe {
            SDL_Init(0x20);
            UI {
                w: {
                    do "rspt".with_c_str() |cstr| {
                        SDL_CreateWindow(cstr, -1, -1, opts.width as i32, opts.height as i32, 0)
                    }
                },
                width: opts.width,
                height: opts.height
            }
        }
    }

    #[fixed_stack_segment]
    pub fn paint(&mut self, data: &[RGB]) -> bool {
        unsafe {
            let surface = SDL_GetWindowSurface(self.w);
            SDL_LockSurface(surface);
            let pixels = (*surface).pixels as *mut u32;
            do vec::raw::mut_buf_as_slice(pixels, self.width*self.height) |pixs| {
                for (i, &c) in data.iter().enumerate() {
                    pixs[i] = f2i(c.b) | (f2i(c.g) << 8) | (f2i(c.r) << 16);
                }
            }
            SDL_UnlockSurface(surface);
            SDL_UpdateWindowSurface(self.w);

            let ev: SDL_Event = unstable::intrinsics::init();
            while SDL_PollEvent(&ev) == 1 {
                if [0x100, 0x300].contains(&ev.etype) { return false }
            }
        }
        true
    }
}
