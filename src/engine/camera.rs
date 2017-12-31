extern crate serde;
extern crate cgmath;

use utilities::math::*;
use utilities::codable::*;

#[derive(Debug, Deserialize)]
struct CameraBasis {
    right: UnitVec3,
    up: UnitVec3,
    back: UnitVec3,
}

impl CameraBasis {
    fn xyz() -> CameraBasis {
        CameraBasis {
            right: Vec3::unit_x().unit(),
            up: Vec3::unit_y().unit(),
            back: Vec3::unit_z().unit(),
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    //basis and direction must always be consistent
    pub position: Vec3,
    basis: CameraBasis,
    direction: UnitVec3,
    pub plane_width: f32,
    pub plane_height: f32,
    pub plane_distance: f32,
}

#[derive(Deserialize)]
struct CameraSpec {
    //basis and direction must always be consistent
    pub position: CodableWrapper<Vec3>,
    pub direction: UnitVec3,
    pub up: UnitVec3,
    pub plane_width: f32,
    pub plane_height: f32,
    pub plane_distance: f32,
}

impl_deserialize!(Camera, |deserializer| {
    let spec: CameraSpec = CameraSpec::deserialize(deserializer)?;
    Ok(Camera::new(
        spec.position.get(),
        *spec.direction.value(),
        *spec.up.value(),
        spec.plane_width,
        spec.plane_height,
        spec.plane_distance
    ))
});

impl Camera {
    pub fn new_default() -> Camera {
        Camera {
            position: Vec3::new(0.0, 0.0, 0.0).into(),
            basis: CameraBasis::xyz(),
            direction: -Vec3::unit_z().unit(),
            plane_width: 1.0,
            plane_height: 1.0,
            plane_distance: 1.0
        }
    }

    pub fn new(position: Vec3, direction: Vec3, up: Vec3, plane_width: f32,
           plane_height: f32, plane_distance: f32) -> Camera {
        let mut camera = Camera {
            position: position.into(),
            basis: CameraBasis::xyz(),
            direction: -Vec3::unit_z().unit(),
            plane_width: plane_width,
            plane_height: plane_height,
            plane_distance: plane_distance,
        };
        camera.look_at(&direction, &up);
        camera
    }

    pub fn look_at(&mut self, direction: &Vec3, non_ortho_up: &Vec3) {
        self.direction = direction.unit();
        self.basis.back = -self.direction.clone();
        self.basis.right = non_ortho_up.unit().cross(self.basis.back.clone());
        self.basis.up = self.basis.back.cross(self.basis.right.clone());
    }

    pub fn direction(&self) -> &UnitVec3 {
        &self.direction
    }

    /// shoots out ray corresponding to u and v coordinates.
    /// direction is normalized
    /// u and v should both be in the range [0,1] if the ray should be inside the camera's image
    pub fn shoot_ray(&self, u: f32, v: f32) -> RayUnit {

        let direction = self.direction.value() * self.plane_distance
            + ((u - 0.5) * self.basis.right.value() * self.plane_height)
            + ((v - 0.5) * self.basis.up.value() * self.plane_width);
        RayUnit::new(self.position, direction.unit())
    }
}
