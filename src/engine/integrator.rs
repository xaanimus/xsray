extern crate rand;
extern crate cgmath;

use std::f32;
use std::rc::Rc;

use utilities::math::*;
use utilities::color::*;
use super::scene::*;
use super::shader::*;
use self::cgmath::Matrix3;

//for now just a point light sampler.
//in the future should sample emissive surfaces
//as well
struct LightSample {
    position: Vec3,
    intensity: Color3,
    sample_probability: f32
}

fn sample_light(scene: &Scene) -> Option<LightSample> {
    if scene.lights.is_empty() { return None }

    let light_i = rand::random::<usize>() % scene.lights.len();
    let light: &Light = &scene.lights[light_i];

    Some(LightSample {
        position: light.position,
        intensity: Color3::new(light.intensity, light.intensity, light.intensity),
        sample_probability: 1. / scene.lights.len() as f32
    })
}


struct PathIntersection {
    shader: Rc<Shader>,
    position: Vec3,
    normal: Vec3
}

struct Path {
    start_position: Vec3,
    intersections: Vec<PathIntersection>,
    light_sample: Option<LightSample>
}

pub trait Integrator {
    fn shade_ray(&self, ray: &RayUnit, scene: &Scene) -> Color3;
    fn shade_camera_point(&self, scene: &Scene, u: f32, v: f32) -> Color3;
}

pub struct PathTracerIntegrator {
    pub max_bounces: u32,
    pub number_samples: u32
}

///returns true if the path will be completely dark using unidirectional path tracing
fn unidirectional_path_has_no_light(path: &Path, scene: &Scene) -> bool {
    if path.intersections.is_empty() || path.light_sample.is_none() { return true }
    let last_intersection: &PathIntersection = path.intersections.last().as_ref().unwrap();
    let light_sample: &LightSample = path.light_sample.as_ref().unwrap();
    let light_intersected = scene
        .intersect_for_obstruction(last_intersection.position, light_sample.position)
        .intersected();
    light_intersected
}

fn shade_path(path: &Path, scene: &Scene, max_bounces: u32) -> Color3 {
    if unidirectional_path_has_no_light(path, scene) { return Color3::zero() }

    let mut accumulated_light = Color3::new(1., 1., 1.);
    let light_sample: &LightSample = path.light_sample.as_ref().unwrap();

    let mut previous_position = path.start_position;
    for i_intersection in 0..path.intersections.len() {
        let intersection: &PathIntersection = &path.intersections[i_intersection];
        let next_position: &Vec3 = if i_intersection == path.intersections.len() - 1 {
            &light_sample.position
        } else {
            &path.intersections[i_intersection + 1].position
        };

        let light_directions = LightDirectionPair {
            outgoing: &(previous_position - intersection.position).unit(),
            incoming: &(next_position - intersection.position).unit()
        };

        let brdf_cos_term = intersection.shader.brdf_cosine_term(
            &intersection.normal.unit(), &light_directions
        );

        let probability_of_incoming_sample = intersection.shader.probability_of_sample(
            &intersection.normal.unit(), &light_directions
        );

        accumulated_light.mul_assign_element_wise(
            brdf_cos_term / probability_of_incoming_sample
        );

        previous_position = intersection.position;
    }
    
    //finally, multiply by light sample
    let light_distance = (light_sample.position - previous_position).magnitude();
    let light_intensity = light_sample.intensity;

    accumulated_light.mul_element_wise(light_intensity) / light_distance.powi(2)
}

//fn _shade_path(path: &Path, scene: &Scene, max_bounces: u32) -> Color3 {
//    if path.intersections.is_empty() { return Color3::zero() }
//
//    let light_sample = &path.light_sample;
//    let mut accumulated_light = Color3::new(1., 1., 1.);
//
//    let mut previous_position = path.start_position;
//    for i_intersection in 0..path.intersections.len() {
//        let intersection = &path.intersections[i_intersection];
//        let outgoing_direction = (previous_position - intersection.position).unit();
//
//        //if there is another after this one, acculumate the color for this intersection
//        if let Some(next_intersection) = path.intersections.get(i_intersection + 1) {
//            let incoming_vector = next_intersection.position - intersection.position;
//            let incoming_direction = incoming_vector.unit();
//            let brdf_cos_term = intersection.shader.brdf_cosine_term(
//                &intersection.normal.unit(), &outgoing_direction, &incoming_direction);
//
//            let probability_of_incoming_direction_inverse = 1.0 /
//                intersection.shader.probability_of_sample(
//                    &intersection.normal.unit(), &incoming_direction, &outgoing_direction);
//
//            let attenuation = 1.; // incoming_vector.magnitude().powi(2);
//            accumulated_light.mul_assign_element_wise(
//                brdf_cos_term.mul_element_wise(attenuation) *
//                    probability_of_incoming_direction_inverse
//            );
//        } else if let &Some(ref light) = light_sample {
//            let light_intersection = scene.intersect_for_obstruction(
//                intersection.position,
//                light.position);
//            if !light_intersection.intersected() {
//                let light_vector = light.position - intersection.position;
//                let light_distance = light_vector.magnitude();
//                let incoming_direction = light_vector.unit();
//                let incoming_intensity = light.intensity / light_distance.powi(2);
//                let brdf_cos_term = intersection.shader.brdf_cosine_term(
//                    &intersection.normal.unit(), &outgoing_direction, &incoming_direction);
//                let probability_of_incoming_direction_inverse = 1.0 /
//                    intersection.shader.probability_of_sample(
//                        &intersection.normal.unit(), &incoming_direction, &outgoing_direction);
//                accumulated_light = accumulated_light
//                    .mul_element_wise(brdf_cos_term)
//                    .mul_element_wise(incoming_intensity) / light.sample_probability *
//                    probability_of_incoming_direction_inverse;
//                return accumulated_light
//            }
//        }
//
//        previous_position = intersection.position;
//    }
//
//    Color3::zero()
//
//    //consider changing to max_bounces
//    //sum_of_samples / path.intersections.len() as f32
//    //let num_ray_samples = max_bounces as f32 + 1.;
//    //sum_of_samples / num_ray_samples as f32
//}

//TODO for bdpt path shading
fn shade_path_interconnected(path: &Path) {
}

///traces a path with the last ray being linked to a random light
///An unobstructed path exists between the camera and the last intersection
///in Path.intersections. It is not guaranteed that a ray from last intersection
///to the light sample is unobstructed
///Max bounces is >= 0
fn compute_path(ray: &RayUnit, scene: &Scene, max_bounces: u32) -> Path {
    let mut path = Path {
        start_position: ray.position,
        intersections: Vec::new(),
        light_sample: None
    };

    let mut current_ray = ray.clone();
    for _ in 0..(max_bounces + 1) {
        let record = scene.intersect(&current_ray);
        if !record.intersected() {
            break;
        }

        let shader = record.shader.clone()
            .unwrap_or(Rc::new(default_shader()));
        let intersection = PathIntersection {
            shader: shader.clone(),
            position: record.position,
            normal: record.normal
        };

        path.intersections.push(intersection);

        let new_direction = shader.sample_bounce(&record.normal.unit(),
                                                 &current_ray.direction.neg());
        current_ray = RayBase::new_epsilon_offset(record.position, new_direction);
    }

    //if there are intersections, connect path to a light
    if !path.intersections.is_empty() {
        path.light_sample = sample_light(scene);
    }

    path
}

impl Integrator for PathTracerIntegrator {
    fn shade_ray(&self, ray: &RayUnit, scene: &Scene) -> Color3 {
        let max_bounces = rand::random::<u32>() % self.max_bounces;
        let path = compute_path(ray, scene, max_bounces);
        let color = shade_path(&path, scene, max_bounces);
        color
    }

    fn shade_camera_point(&self, scene: &Scene, u: f32, v: f32) -> Color3 {
        let ray = scene.camera.shoot_ray(u,v);
        (0..self.number_samples)
            .fold(Color3::zero(), |acc, _| acc + self.shade_ray(&ray, scene)) /
            self.number_samples as f32
    }
}
