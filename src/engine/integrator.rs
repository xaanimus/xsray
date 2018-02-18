extern crate rand;
extern crate cgmath;

use std::f32;
use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::DerefMut;

use utilities::math::*;
use utilities::color::*;
use utilities::sampler::Sampler;
use utilities::sampler::PseudorandomSampler;

use super::scene::*;
use super::shader::*;
use self::cgmath::Matrix3;
use self::rand::Rng;
use utilities::sampler::SamplerSpec;
use utilities::sampler::NumberSequenceSampler;

#[derive(Deserialize)]
#[serde(tag = "kind")]
pub enum IntegratorSpec {
    PathTracer {
        max_bounces: u32,
        number_of_samples: u32,
        shade_shadow_rays: Option<bool>,
        #[serde(rename = "sampler")]
        sampler_spec: SamplerSpec
    },
}

impl IntegratorSpec {
    pub fn into_integrator(&self) -> Box<Integrator> {
        use self::IntegratorSpec::*;
        match *self {
            PathTracer {
                max_bounces, number_of_samples,
                shade_shadow_rays, ref sampler_spec
            } => {
                if shade_shadow_rays == Some(true) {
                    println!("shading shadow rays is not yet implemented.");
                }
                Box::new(PathTracerIntegrator {
                    max_bounces,
                    number_of_samples,
                    shade_shadow_rays: shade_shadow_rays.unwrap_or(false),
                    sampler_number_sequence: sampler_spec.to_number_sequence(1000),
                })
            }
        }
    }
}


//for now just a point light sampler.
//in the future should sample emissive surfaces
//as well
struct LightSample {
    position: Vec3,
    intensity: Color3,
    sample_probability: f32
}

fn sample_light<TSpl: Sampler + ?Sized>(scene: &Scene, sampler: &mut TSpl) -> Option<LightSample> {
    if scene.lights.is_empty() { return None }

    let light_i = sampler.get_usize_from_f32(scene.lights.len());
    let light: &Light = &scene.lights[light_i];

    Some(LightSample {
        position: light.position.get(),
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

pub struct UvPixelInfo {
    pub uv_pixel_width: f32,
    pub uv_pixel_height: f32
}

pub trait Integrator: Debug {
    fn shade_ray(&self, ray: &RayUnit, scene: &Scene, sampler: &mut NumberSequenceSampler) -> Color3;
    fn shade_camera_point(&self, scene: &Scene, u: f32, v: f32,
                          render_info: &UvPixelInfo) -> Color3;
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
        //get the current intersection and position of next intersection/light
        let intersection: &PathIntersection = &path.intersections[i_intersection];
        let next_position: &Vec3 = if i_intersection == path.intersections.len() - 1 {
            &light_sample.position
        } else {
            &path.intersections[i_intersection + 1].position
        };

        //create incoming and outgoing light vectors
        let light_directions = LightDirectionPair {
            outgoing: &(previous_position - intersection.position).unit(),
            incoming: &(next_position - intersection.position).unit()
        };

        //shade brdf cos term
        let brdf_cos_term = intersection.shader.brdf_cosine_term(
            &intersection.normal.unit(), &light_directions
        );

        //probability
        let probability_of_incoming_sample = if i_intersection == path.intersections.len() - 1 {
            //if last intersection, use a uniform hemisphere
            1. / 2. * PI
        } else {
            intersection.shader.probability_of_sample(
                &intersection.normal.unit(), &light_directions
            )
        };

        //accumulate
        accumulated_light.mul_assign_element_wise(
            brdf_cos_term / probability_of_incoming_sample
        );

        previous_position = intersection.position;
    }
    
    //finally, multiply by light sample
    //assuming this is a point light
    let light_distance = (light_sample.position - previous_position).magnitude();
    let light_intensity = light_sample.intensity;

    accumulated_light.mul_element_wise(light_intensity) / light_distance.powi(2)
}


//TODO for bdpt path shading
fn shade_path_interconnected(path: &Path) {
}

///traces a path with the last ray being linked to a random light
///An unobstructed path exists between the camera and the last intersection
///in Path.intersections. It is not guaranteed that a ray from last intersection
///to the light sample is unobstructed
///Max bounces is >= 0
fn trace_path(ray: &RayUnit, scene: &Scene, max_bounces: u32, sampler: &mut NumberSequenceSampler) -> Path {
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

        let new_direction = shader.sample_bounce(
            &record.normal.unit(),
            &current_ray.direction.neg(),
            sampler);
        current_ray = RayBase::new_epsilon_offset(record.position, new_direction);
    }

    //if there are intersections, connect path to a light
    if !path.intersections.is_empty() {
        path.light_sample = sample_light(scene, sampler);
    }

    path
}

fn sample_anti_alias_uv<TSpl: Sampler>(
    u:f32, v:f32, pixel_info: &UvPixelInfo,
    sampler: &mut TSpl
) -> (f32, f32) {
    let (rand_0, rand_1) = sampler.get_2d_f32();
    let (offset_u, offset_v) =
        ((rand_0 - 0.5) * pixel_info.uv_pixel_width,
         (rand_1 - 0.5) * pixel_info.uv_pixel_height);
    (u + offset_u, v + offset_v)
}

#[derive(Debug, Clone)]
pub struct PathTracerIntegrator {
    pub max_bounces: u32,
    pub number_of_samples: u32,
    pub shade_shadow_rays: bool, //currently shades shadow rays without weights
    sampler_number_sequence: NumberSequenceSampler
}

impl Integrator for PathTracerIntegrator {
    fn shade_ray(&self, ray: &RayUnit, scene: &Scene, sampler: &mut NumberSequenceSampler) -> Color3 {
        let max_bounces = if self.max_bounces == 0 {
            0
        } else {
            rand::random::<u32>() % self.max_bounces
        };

        let path = trace_path(ray, scene, max_bounces, sampler);
        let color = shade_path(&path, scene, max_bounces);
        color
    }

    fn shade_camera_point(
        &self, scene: &Scene, u: f32, v: f32, pixel_info: &UvPixelInfo
    ) -> Color3 {
        let mut acc = Color3::zero();
        let mut number_sequence = self.sampler_number_sequence.clone();
        for _ in 0..self.number_of_samples {
            let (anti_alias_u, anti_alias_v) =
                sample_anti_alias_uv(u, v, pixel_info, &mut number_sequence);
            let ray = scene.camera.shoot_ray(anti_alias_u, anti_alias_v);
            acc += self.shade_ray(&ray, scene, &mut number_sequence);
        }
        acc / self.number_of_samples as f32
    }
}
