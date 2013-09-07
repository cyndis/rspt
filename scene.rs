use nalgebra::vec::*;
use nalgebra::mat::*;
use nalgebra::adaptors::transform::*;
use nalgebra::adaptors::rotmat::*;
use Ts = nalgebra::traits::transformation::Transform;
use image;
use random;
use aabb;

type Vec3f = Vec3<float>;
type Mat4f = Mat4<float>;
pub type Transform3d = Transform<Vec3f, Rotmat<Mat3<float>>>;

pub struct Ray {
    pos: Vec3f,
    dir: Vec3f
}

pub struct LinearScene {
    objs: ~[Object]
}

pub struct Intersection<'self> {
    distance: float,
    object: &'self Object
}

pub trait Scene {
    fn intersect<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>>;
}

impl Scene for LinearScene {
    fn intersect<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>> {
        let mut closest = None;
        for obj in self.objs.iter() {
            let intr = obj.intersect(ray);
            match (intr, closest) {
                (Some(i), None) if i.distance > 0.0 => {
                    closest = Some(i)
                }
                (Some(i), Some(c)) if i.distance > 0.0 && i.distance < c.distance => {
                    closest = Some(i)
                }
                _ => ()
            }
        }
        closest
    }
}

#[deriving(Eq)]
pub enum ReflectanceFunction {
    Diffuse,
    Specular
}

#[deriving(Encodable)]
pub struct ReflectanceDistribution {
    diffuse: float,
    specular: float
}

impl ReflectanceDistribution {
    pub fn sample(&self) -> ReflectanceFunction {
        let r = random::random_real();
        if r <= self.diffuse { return Diffuse }
        if r <= self.diffuse + self.specular { return Specular }
        fail!("non-1.0 reflectance distribution, diffuse %f, specular %f",
              self.diffuse, self.specular);
    }
}

#[deriving(Encodable)]
pub struct Material {
    rfd: ReflectanceDistribution,
    color: image::RGB,
    emission: image::RGB
}

impl Material {
    pub fn diffuse(color: image::RGB, emission: image::RGB) -> Material {
        Material {
            rfd: ReflectanceDistribution { diffuse: 1.0, specular: 0.0 },
            color: color,
            emission: emission
        }
    }
}

pub struct Object {
    inv_transform: Transform3d,
    shape: Shape,
    material: Material
}

impl Object {
    pub fn new(transform: Transform3d, shape: Shape, material: Material) -> Object {
        Object {
            inv_transform: transform.inv_transformation(),
            shape: shape,
            material: material
        }
    }
}

fn minf(a: float, b: float) -> float { if a < b { a } else { b } }
fn maxf(a: float, b: float) -> float { if a < b { b } else { a } }

fn transform_ray(ray: &Ray, inv_transform: &Transform3d) -> Ray {
    let tpos = ray.pos + ray.dir;
    let apos = inv_transform.transform(&ray.pos);
    Ray {
        pos: apos,
        dir: (inv_transform.transform(&tpos) - apos).normalized()
    }
}

impl Object {
    pub fn intersect<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>> {
        match self.shape {
            Sphere { radius } => {
                let ray = transform_ray(ray, &self.inv_transform);
                let a = ray.dir.dot(&ray.dir);
                let b = ray.dir.dot(&ray.pos) * 2.0;
                let c = ray.pos.dot(&ray.pos) - radius * radius;
                let d = b * b - 4.0 * a * c;

                if d < 0.0 {
                    return None;
                }

                let a2inv = 1.0 / (2.0 * a);
                let ts = [(-b + d.sqrt()) * a2inv,
                          (-b - d.sqrt()) * a2inv];

                Some(
                    Intersection {
                        object: self,
                        distance: *ts.iter().min().unwrap()
                    }
                )
            },
            Box { aabb: aabb::AABB { min, max } } => {
                let ray = transform_ray(ray, &self.inv_transform);
                let ray_dir_inv = Vec3::new(1.0 / ray.dir.x, 1.0 / ray.dir.y, 1.0 / ray.dir.z);
                let t1 = Vec3::new((min.x - ray.pos.x) * ray_dir_inv.x,
                                   (min.y - ray.pos.y) * ray_dir_inv.y,
                                   (min.z - ray.pos.z) * ray_dir_inv.z);
                let t2 = Vec3::new((max.x - ray.pos.x) * ray_dir_inv.x,
                                   (max.y - ray.pos.y) * ray_dir_inv.y,
                                   (max.z - ray.pos.z) * ray_dir_inv.z);

                let tmin = maxf(minf(t1.z, t2.z), maxf(minf(t1.y, t2.y), minf(t1.x, t2.x)));
                let tmax = minf(maxf(t1.z, t2.z), minf(maxf(t1.y, t2.y), maxf(t1.x, t2.x)));

                if tmax >= maxf(0.0, tmin) {
                    Some(Intersection { object: self, distance: tmin })
                } else {
                    None
                }
            },
            Triangle { a, b, c } => {
                let ray = transform_ray(ray, &self.inv_transform);

                let e1 = b - a;
                let e2 = c - a;

                let h = ray.dir.cross(&e2);
                let aa = e1.dot(&h);

                if aa.approx_eq(&0.0) { return None }

                let f = 1.0/aa;
                let s = ray.pos - a;
                let u = f * s.dot(&h);

                if u < 0.0 || u > 1.0 { return None }

                let q = s.cross(&e1);
                let v = f * ray.dir.dot(&q);

                if v < 0.0 || u + v > 1.0 { return None }

                let t = f * e2.dot(&q);

                Some(Intersection { object: self, distance: t })
            }
        }
    }

    pub fn normal_at(&self, surface_pt: Vec3f) -> Vec3f {
        match self.shape {
            Sphere { _ } => {
                self.inv_transform.transform(&surface_pt).normalized()
            },
            Box { aabb: aabb::AABB { min, max } } => {
                let c1 = self.inv_transform.transform(&surface_pt) - min;
                let c2 = self.inv_transform.transform(&surface_pt) - max;

                     if c1.x.approx_eq(&0.0) { return Vec3::new(-1.0, 0.0, 0.0) }
                else if c1.y.approx_eq(&0.0) { return Vec3::new( 0.0,-1.0, 0.0) }
                else if c1.z.approx_eq(&0.0) { return Vec3::new( 0.0, 0.0,-1.0) }
                else if c2.x.approx_eq(&0.0) { return Vec3::new( 1.0, 0.0, 0.0) }
                else if c2.y.approx_eq(&0.0) { return Vec3::new( 0.0, 1.0, 0.0) }
                else if c2.z.approx_eq(&0.0) { return Vec3::new( 0.0, 0.0, 1.0) }
                else { fail!(~"impossible") }
            },
            Triangle { a, b, c } => {
                (b - a).cross(&(c - a))
            }
        }
    }

    pub fn bounding_box(&self) -> aabb::AABB {
        match self.shape {
            Sphere { radius } => {
                aabb::AABB::from_origin_extents(Vec3::new(0.0, 0.0, 0.0),
                                                Vec3::new(radius, radius, radius))
                           .transformed(&self.inv_transform.inv_transformation())
            },
            Box { aabb } => aabb.transformed(&self.inv_transform.inv_transformation()),
            Triangle { a, b, c } => {
                let xs = [a.x, b.x, c.x];
                let ys = [a.y, b.y, c.y];
                let zs = [a.z, b.z, c.z];
                aabb::AABB::from_min_max(Vec3::new(*xs.iter().min().unwrap(), *ys.iter().min().unwrap(), *zs.iter().min().unwrap()),
                                         Vec3::new(*xs.iter().max().unwrap(), *ys.iter().max().unwrap(), *zs.iter().max().unwrap()))
                           .transformed(&self.inv_transform.inv_transformation())
            }
        }
    }
}

pub enum Shape {
    Sphere { radius: float },
    Box { aabb: aabb::AABB },
    Triangle { a: Vec3f, b: Vec3f, c: Vec3f }
}
