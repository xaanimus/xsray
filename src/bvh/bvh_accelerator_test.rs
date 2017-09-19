use super::bvh_accelerator::*;
use super::math::*;

#[test]
fn it_intersects_with_bounding_box() {
    let bb = AABoundingBox{lower: Vec3::zero(), upper: Vec3::new(1.0, 1.0, 1.0)};

    let ray_that_intersects = RayUnit::new(Vec3::new(-1.0, -1.0, -1.0),
                                           Vec3::new(1.0, 1.0, 1.0).unit());
    assert!(bb.intersects_with_bounding_box(&ray_that_intersects));

    let ray_that_doesnt_intersect =
        RayUnit::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(1.0, 1.0, 1.0).unit());
    assert!(!bb.intersects_with_bounding_box(&ray_that_doesnt_intersect));
}

#[test]
fn test_bounding_box() {
    let make_as_thing = 3;
}


