use nalgebra::vec::*;
use scene;

type Vec3f = Vec3<float>;

struct Cached {
    localat: Vec3f,
    hori: Vec3f,
    vert: Vec3f
}

pub struct Camera {
    // passed
    position: Vec3f,
    lookat: Vec3f,
    fov: float,
    aspect: float,

    // calculated
    cache: Option<Cached>
}

impl Camera {
    pub fn new(position: Vec3f, lookat: Vec3f, fov: float, aspect: float) -> Camera {
        let mut c = Camera {
            position: position,
            lookat: lookat,
            fov: fov,
            aspect: aspect,
            cache: None
        };
        c.calculate();
        c
    }

    fn calculate(&mut self) {
        let localat = (self.lookat - self.position).normalized();
        let up = Vec3::y();
        let hori = up.cross(&localat);
        let vert = hori.cross(&localat);
        let h = (0.5 * self.fov).tan();

        self.cache = Some(Cached {
            localat: localat,
            hori: hori * h,
            vert: vert * (1.0 / self.aspect) * h
        })
    }

    pub fn make_ray(&self, x: float, y: float) -> scene::Ray {
        let c = self.cache.get_ref();
        scene::Ray {
            pos: self.position,
            dir: (c.localat +
                  c.hori * (2.0 * x - 1.0) +
                  c.vert * (2.0 * y - 1.0)
                 ).normalized()
        }
    }
}
