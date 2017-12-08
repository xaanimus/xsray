extern crate rand;
extern crate cgmath;

use std::f32;
use utilities::math::*;
use utilities::color::*;
use super::scene::*;
use self::cgmath::Matrix3;

pub struct Integrator {
    pub max_bounces: u32
}

impl Integrator {
    /// returns sum of light contribution to location for each light in the scene
    /// Makes sure only to get light in direction of normal vector
    fn get_total_light_contribution(location: &Vec3, normal: &UnitVec3, scene: &Scene) -> Color3 {
        let mut total_contribution = Color3::zero();
        let light = &scene.lights[0];

        let light_vector = light.position - location;
        let distance_to_light = light_vector.magnitude();

        let light_ray = RayUnit::new_shadow_target(
            *location,
            light.position
        );

        let light_intersection = scene.intersect(&light_ray);
        let light_dot_normal = light_vector.normalize()
            .dot(*normal.vec());

        if light_dot_normal > 0.0 && !light_intersection.intersected() {
            total_contribution += Color3::new(1.0, 1.0, 1.0) * light.intensity
                / distance_to_light.powi(2)
        };
        total_contribution
    }

    pub fn render(&self, ray: RayUnit, scene: &Scene) -> Color3 {
        let mut sum_of_color_samples = Color3::zero(); //sum of all samples
        //the current accumulated amount of light in the longest chain of ray bounces
        let mut color_acc = Color3 {x: 1.0, y: 1.0, z: 1.0};
        let mut current_ray = ray.clone();
        let mut light_dot_normal = 0f32;
        let mut num_of_bounces = 0;

        for _ in 0..self.max_bounces {
            let record = scene.intersect(&current_ray);
            if record.intersected() {
                num_of_bounces += 1;
                //without incoming light irradiance term
                let shaded_no_incoming = record.shader.as_ref()
                    .unwrap().brdf(&record)
                    .mul_element_wise(2.0 * f32::consts::PI); //divide by probability of sample
                color_acc.mul_assign_element_wise(shaded_no_incoming);

                //get light for this bounce
                let light_amount = {
                    let light_irradiance = Integrator::get_total_light_contribution(
                        &record.position,
                        &record.normal.unit(),
                        scene
                    );
                    let light = &scene.lights[0];
                    let light_vec = (light.position - record.position).normalize();
                    let cosine_term = light_vec.dot(record.normal);
                    cosine_term * light_irradiance
                };
                sum_of_color_samples += color_acc.mul_element_wise(light_amount);

                //sample a new ray
                let direction = sample_vector_hemisphere(record.normal.unit());
                light_dot_normal = direction.vec().dot(record.normal);
                current_ray = RayUnit::new_shadow(record.position, direction);
                color_acc.mul_assign_element_wise(light_dot_normal);
            } else {
                break
            }
        }

        println!("n{}", num_of_bounces);

        sum_of_color_samples / self.max_bounces as f32
    }
}

fn sample_vector_hemisphere(normal: UnitVec3) -> UnitVec3 {
    let axis_0 = { //axis that's perpendicular to normal
        let up = Vec3::new(0.0, 1.0, 0.0);
        let x_dir = Vec3::new(1.0, 0.0, 0.0);
        let dot = up.dot(*normal.vec());
        if dot < 0.95 {
            up.cross(*normal.vec()).normalize()
        } else {
            x_dir.cross(*normal.vec()).normalize()
        }
    };
    let axis_1 = axis_0.cross(*normal.vec());

    let (u, v) = rand::random::<(f32, f32)>();
    let r = (1.0 - u*u).sqrt();
    let phi = 2.0 * f32::consts::PI * v;

    let sampled_vec = Vec3::new(
        r * phi.cos(),
        u,
        -r * phi.sin(),
    );

    let rot_matrix = Matrix3 {
        x: axis_0,
        y: *normal.vec(),
        z: axis_1
    };

    (rot_matrix * sampled_vec).unit()
}

#[test]
fn test_sample_vector_hemisphere() {
    let n = Vec3::new(0.0, 1.0, 0.0).unit();
    let s = sample_vector_hemisphere(n);
    println!("{:?}", s);
}


