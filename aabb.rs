use nalgebra::vec::*;
use Ts = nalgebra::traits::transformation::Transform;
use scene;

type Vec3f = Vec3<float>;

pub struct AABB {
    min: Vec3f,
    max: Vec3f
}

impl AABB {
    pub fn from_min_max(min: Vec3f, max: Vec3f) -> AABB {
        AABB {
            min: min, max: max
        }
    }

    pub fn from_origin_extents(origin: Vec3f, extents: Vec3f) -> AABB {
        assert!(extents.x > 0.0 && extents.y > 0.0 && extents.z > 0.0);
        AABB {
            min: origin - extents,
            max: origin + extents
        }
    }

    pub fn transformed(&self, ts: &scene::Transform3d) -> AABB {
        /* FIXME: need to find the new min and max points.. */
        fail!("BUG!");
        AABB {
            min: ts.transform(&self.min),
            max: ts.transform(&self.max)
        }
    }

    pub fn stretch_to(&mut self, other: &AABB) {
        if other.min.x < self.min.x { self.min.x = other.min.x }
        if other.min.y < self.min.y { self.min.y = other.min.y }
        if other.min.z < self.min.z { self.min.z = other.min.z }
        if other.max.x > self.max.x { self.max.x = other.max.x }
        if other.max.y > self.max.y { self.max.y = other.max.y }
        if other.max.z > self.max.z { self.max.z = other.max.z }
    }
}

