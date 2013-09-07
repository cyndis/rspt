use std::{io, iterator};

#[deriving(Clone, Eq, Encodable)]
pub struct RGB { r: float, g: float, b: float }

impl RGB {
    pub fn add_v(&self, c: &RGB) -> RGB { RGB { r: self.r + c.r, g: self.g + c.g, b: self.b + c.b } }
    pub fn mul_v(&self, c: &RGB) -> RGB { RGB { r: self.r * c.r, g: self.g * c.g, b: self.b * c.b } }
    pub fn mul_t(&self, c: float) -> RGB { RGB { r: self.r * c, g: self.g * c, b: self.b * c } }

    pub fn black() -> RGB { RGB { r: 0.0, g: 0.0, b: 0.0 }}
    pub fn white() -> RGB { RGB { r: 1.0, g: 1.0, b: 1.0 }}
    pub fn red() -> RGB { RGB { r: 1.0, g: 0.0, b: 0.0 }}
    pub fn blue() -> RGB { RGB { r: 0.0, g: 0.0, b: 1.0 }}
}

pub struct Image {
    data: ~[RGB],
    iters: uint,
    w: uint,
    h: uint
}

fn clamp(x: float) -> float {
    if x > 1.0 { 1.0 }
    else { x }
}

impl Image {
    pub fn new(w: uint, h: uint) -> Image {
        let mut i = Image { data: ~[], iters: 0, w: w, h: h };
        i.data.grow(w*h, &RGB{r: 0.0, g: 0.0, b: 0.0});
        i
    }

    pub fn to_ppm(&self) -> ~str {
        do io::with_str_writer |wr| {
            wr.write_line(fmt!("P3 %? %? 255", self.w, self.h));
            for y in iterator::range(0, self.h) {
                for x in iterator::range(0, self.w) {
                    let color = self.data[y*self.w+x];
                    let cs = [clamp(color.r), clamp(color.g), clamp(color.b)];
                    for &v in cs.iter() {
                        let vs = [(v*255.0).to_int(), 255];
                        wr.write_str(fmt!("%? ", vs.iter().min()));
                    }
                }
            }
        }
    }

    pub fn each_coordinate(&self, f: &fn(x: uint, y: uint) -> bool) {
        for x in iterator::range(0, self.w) {
            for y in iterator::range(0, self.h) {
                if !f(x, y) { return; }
            }
        }
    }

    pub fn blend_into(&self, other: &mut Image, count: uint) {
        assert!(self.w == other.w && self.h == other.h);

        let count = count as float;
        do self.each_coordinate |x, y| {
            let color = &self.data[y*self.w+x];
            let old = other.data[y*self.w+x];
            other.data[y*self.w+x] = RGB {
                r: (old.r * count + color.r) / (count + 1.0),
                g: (old.g * count + color.g) / (count + 1.0),
                b: (old.b * count + color.b) / (count + 1.0)
            };
            true
        }
    }

    pub fn set(&mut self, x: uint, y: uint, c: RGB) {
        self.data[y*self.w+x] = c;
    }
}
