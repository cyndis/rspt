use scene;
use image::RGB;
use nalgebra::vec::*;
use std::{path, io, float, uint};

pub fn load_obj(path: &path::Path, scene: &mut scene::LinearScene) {
    let mut vcache = ~[::std::num::Zero::zero()];
    let rd = io::file_reader(path).unwrap();

    while !rd.eof() {
        let line = rd.read_line();
        if line.len() == 0 { loop }
        match line[0] as char {
            'v' => {
                let cs: ~[&str] = line.split_iter(' ').collect();
                vcache.push(Vec3::new(float::from_str(cs[1]).unwrap(),
                                      float::from_str(cs[2]).unwrap(),
                                      float::from_str(cs[3]).unwrap()));
            }
            'f' => {
                let is: ~[&str] = line.split_iter(' ').collect();
                let tri = scene::Triangle { a: vcache[uint::from_str(is[1]).unwrap()],
                                            b: vcache[uint::from_str(is[2]).unwrap()],
                                            c: vcache[uint::from_str(is[3]).unwrap()] };
                scene.objs.push(scene::Object { inv_transform: ::std::num::One::one(),
                                                shape: tri,
                                                material: scene::Material::diffuse(RGB { r: 0.75, g: 0.75, b: 0.75 }, RGB::black()) });
            },
            _ => fail!("unsupported obj entry")
        }
    }
}
