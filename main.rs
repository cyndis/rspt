use std::{io, iterator, path};
use image::{Image, RGB};
use scene;
use camera;
use nalgebra::vec::*;
use nalgebra::mat::*;
use nalgebra::adaptors::rotmat::Rotmat;
use random;
use extra::arc;
use std::{task, comm};
use sdlui::UI;
use std::num::One;
use obj;
use aabb;

use extra::serialize::*;
use extra::json;

pub struct RenderOptions {
    width: uint,
    height: uint
}

fn trace_ray<S: scene::Scene>(ray: scene::Ray, scene: &S, depth: uint)
    -> RGB
{
    let maybe_intr = scene.intersect(&ray);
    let intr = match maybe_intr {
        None => return RGB::black(),
        Some(_) => maybe_intr.unwrap()
    };

    // russian roulette
    let mut color = intr.object.material.color;
    let refls = [color.r, color.g, color.b];
    let max_refl_comp = *refls.iter().max().unwrap();
    if depth > 5 || max_refl_comp == 0.0 {
        if random::random_real() < max_refl_comp {
            color = color.mul_t(1.0 / max_refl_comp);
        } else {
            return intr.object.material.emission;
        }
    }

    let rf = intr.object.material.rfd.sample();

    match rf {
        scene::Diffuse => {
            let hit_pt = ray.pos + ray.dir * intr.distance;
            let normal = intr.object.normal_at(hit_pt);

            let mut new_dir = random::random_vec();
            if new_dir.dot(&normal) < 0.0 {
                new_dir = -new_dir;
            }

            let new_ray = scene::Ray { pos: hit_pt + new_dir * 0.001, dir: new_dir };

            return intr.object.material.emission.add_v(
                &color.mul_v(&trace_ray(new_ray, scene, depth+1)));
        },
        scene::Specular => {
            let hit_pt = ray.pos + ray.dir * intr.distance;
            let normal = intr.object.normal_at(hit_pt);
            let new_dir = ray.dir - normal * 2.0 * normal.dot(&ray.dir);
            let new_ray = scene::Ray {
                pos: hit_pt + new_dir * 0.001,
                dir: new_dir
            };
            return intr.object.material.emission.add_v(
                &color.mul_v(&trace_ray(new_ray, scene, depth+1)));
        }
    }
}

fn trace_pixel<S: scene::Scene>(x: float, y: float, camera: &camera::Camera, scene: &S)
    -> RGB
{
    let ray = camera.make_ray(x, y);
    trace_ray(ray, scene, 0)
}

fn trace_image<S: scene::Scene>(opts: &RenderOptions, camera: &camera::Camera, scene: &S)
    -> Image
{
    let mut i = Image::new(opts.width, opts.height);
    for x in iterator::range(0, opts.width) {
        for y in iterator::range(0, opts.height) {
            let jitter_x = (random::random_real() - 0.5) / (opts.width as float);
            let jitter_y = (random::random_real() - 0.5) / (opts.width as float);
            let color = trace_pixel(x as float / (opts.width as float) + jitter_x,
                                    y as float / (opts.height as float) + jitter_y,
                                    camera, scene);
            i.set(x, y, color);
        }
    }
    i
}

fn id() -> scene::Transform3d {
    One::one()
}

pub fn entrypoint() {
    let opts = RenderOptions {
        width: 240,
        height: 180
    };
    let mut scene = scene::LinearScene {
        objs: ~[
            scene::Object::new(id().translated(&Vec3::new(0.0, -1002.0, 0.0)),
                               scene::Sphere { radius: 1000.0 },
                               scene::Material { rfd: scene::ReflectanceDistribution { diffuse: 1.0, specular: 0.0 },
                                                 color: RGB { r: 0.3, g: 0.3, b: 0.3 }, emission: RGB::black() }),
            scene::Object::new(id().translated(&Vec3::new(0.0, 0.0, -200.0)),
                               scene::Box { aabb: aabb::AABB { min: Vec3::new(-100.0, -100.0, 0.0), max: Vec3::new(100.0, 100.0, 0.1) } },
                               scene::Material::diffuse(RGB::black(), RGB { r: 10.0, g: 10.0, b: 10.0 })),
            scene::Object::new(id().rotated(&Vec3::new(0.0, -2.0, 0.0)).translated(&Vec3::new( 1.5, -2.0, 0.0)),
                               scene::Box { aabb: aabb::AABB { min: Vec3::new(-0.5, 0.0, -0.5),
                                                               max: Vec3::new( 0.5, 1.0,  0.5) } },
                               scene::Material { rfd: scene::ReflectanceDistribution {
                                                          diffuse: 0.2, specular: 0.8
                                                      },
                                                 color: RGB::red(), emission: RGB::black() }),
            scene::Object::new(id().translated(&Vec3::new(-1.5, -1.0, 0.0)),
                               scene::Sphere { radius: 1.0 },
                               scene::Material { rfd: scene::ReflectanceDistribution {
                                                          diffuse: 0.3, specular: 0.7
                                                      },
                                                 color: RGB::white(), emission: RGB::black() }),
            scene::Object::new(id().translated(&Vec3::new(0.0, -2.0, 10.0)),
                               scene::Box { aabb: aabb::AABB { min: Vec3::new(-15.0, 0.0, 0.0), max: Vec3::new(15.0, 30.0, 0.1) } },
                               scene::Material { rfd: scene::ReflectanceDistribution { diffuse: 0.1, specular: 0.9 }, color: RGB::white(), emission: RGB::black() }),
                               /*
            scene::Object::new(id().translated(&Vec3::new(-2.0, 0.0, 0.0)),
                               scene::Triangle { a: Vec3::new(-1.0, 0.0, 0.0), b: Vec3::new(0.0, 1.0, 0.0), c: Vec3::new(1.0, 0.0, 0.0) },
                               scene::Material::diffuse(RGB::blue(), RGB::black()))
                               */
        ]
    };

//    obj::load_obj(&path::Path("dragon.obj"), &mut scene);

    let camera = camera::Camera::new(Vec3::new(-2.0, 2.5, -3.0),
                                     Vec3::new(0.0, 0.0,  0.0),
                                     1.57, (opts.width as float)/(opts.height as float));

    let mut done = 0u;

    let mut image = Image::new(opts.width, opts.height);

    let mut ui = UI::new(&opts);

    let scene_rc = arc::Arc::new(scene);
    let camera_rc = arc::Arc::new(camera);

    let mut tasks_running = 0u;
    let (data_port, data_chan) = comm::stream();
    let data_chan = comm::SharedChan::new(data_chan);
    loop {
        while tasks_running < 8 {
            let my_chan = data_chan.clone();
            let (my_scene, my_camera) = (scene_rc.clone(), camera_rc.clone());
            tasks_running += 1;
            do task::spawn_sched(task::SingleThreaded) {
                let frame = trace_image(&opts, my_camera.get(), my_scene.get());
                my_chan.send(frame);
            }
        }

        let frame = data_port.recv();
        frame.blend_into(&mut image, done);
        done += 1;
        tasks_running -= 1;
        printf!("\r%5u frames done", done);
        if done % 10 == 0 && !ui.paint(image.data) {
            break;
        }
    }
    println("");

    let fp = io::file_writer(&path::Path("output.ppm"), [io::Create]).unwrap();
    fp.write_str(image.to_ppm());
}

