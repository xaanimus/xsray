
extern crate cgmath;

use super::misc::*;

#[derive(Debug)]
struct CameraBasis {
    right: Vec3,
    up: Vec3,
    back: Vec3,
}

impl CameraBasis {
    fn xyz() -> CameraBasis {
        CameraBasis {
            right: Vec3::unit_x(),
            up: Vec3::unit_y(),
            back: Vec3::unit_z(),
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    //basis and direction must always be consistent
    pub position: Vec3,
    basis: CameraBasis,
    direction: Vec3,
    pub plane_width: f32,
    pub plane_height: f32,
    pub plane_distance: f32,
}

impl Camera {
    pub fn new(position: Vec3, direction: Vec3, up: Vec3, plane_width: f32,
           plane_height: f32, plane_distance: f32) -> Camera {
        let mut camera = Camera {
            position: position,
            basis: CameraBasis::xyz(),
            direction: -Vec3::unit_z(),
            plane_width: plane_width,
            plane_height: plane_height,
            plane_distance: plane_distance,
        };
        camera.look_at(&direction, &up);
        camera
    }

    pub fn look_at(&mut self, direction: &Vec3, non_ortho_up: &Vec3) {
        self.basis.back = -*direction;
        self.basis.right = non_ortho_up.cross(self.basis.back);
        self.basis.up = self.basis.back.cross(*non_ortho_up);
        self.direction = *direction;
    }

    pub fn direction(&self) -> &Vec3 {
        &self.direction
    }

    /// shoots out ray corresponding to u and v coordinates.
    /// u and v should both be in the range [0,1] if the ray should be inside the camera's image
    pub fn shoot_ray(&self, u: f32, v: f32) -> Ray {

        let direction = self.direction
            + ((u - 0.5) * self.basis.right)
            + ((v - 0.5) * self.basis.up);
        Ray::new(self.position, direction)
    }
}
