use super::intersectable;
use utilities::multi_math::*;
use utilities::math::RayBase;
use utilities::cmp_util::{CmpFn, cmp};
use cgmath::InnerSpace;

pub type TriangleIntersectionResult<N> = IntersectionResult<N>;

#[derive(Clone, Debug)]
pub struct IntersectionResult<N: MultiNum> {
    pub t: N::Scalar,
    pub beta: N::Scalar,
    pub gamma: N::Scalar
}

impl<N: MultiNum> IntersectionResult<N> {
    pub fn new_no_intersection() -> IntersectionResult<N> {
        IntersectionResult {
            t: N::scalar_inf(),
            beta: N::scalar_inf(),
            gamma: N::scalar_inf()
        }
    }

    pub fn alpha(&self) -> N::Scalar {
        N::scalar_one() - self.beta - self.gamma
    }

    pub fn intersected(&self) -> N::Bool {
        let is_inf = N::scalar_cmp::<cmp::Eq>(self.t, N::scalar_inf());
        N::bool_not(is_inf)
    }
}

// TODO refactor return IntersectionResult & benchmark
#[inline]
pub fn intersect_triangle<N: MultiNum>(
    ray: &RayBase<N>,
    pos0: &MultiVec3<N>,
    edge1: &MultiVec3<N>,
    edge2: &MultiVec3<N>
) -> IntersectionResult<N> {
    let h = ray.direction.value().op_cross(*edge2);
    let a = edge1.op_dot(h);

    let a_is_zero= N::scalar_apprx_eq(a, N::scalar_zero(), N::scalar_big_epsilon());
    if N::all_true(a_is_zero) {
        return IntersectionResult::new_no_intersection()
    }

    let f = N::scalar_one() / a;
    let s = ray.position.op_minus(*pos0);

    let u = f * s.op_dot(h);

    let u_lt_zero = N::scalar_cmp::<cmp::Lt>(u, N::scalar_zero());
    let u_gt_one = N::scalar_cmp::<cmp::Gt>(u, N::scalar_one());
    let u_out_of_bounds = N::bool_or(u_lt_zero, u_gt_one);
    if N::all_true(u_out_of_bounds) {
        return IntersectionResult::new_no_intersection()
    }

    let q = s.op_cross(*edge1);
    let v = f * ray.direction.value().op_dot(q);
    let v_lt_zero = N::scalar_cmp::<cmp::Lt>(v, N::scalar_zero());
    let u_plus_v_gt_one = N::scalar_cmp::<cmp::Gt>(u + v, N::scalar_one());
    let uv_out_of_bounds = N::bool_or(v_lt_zero, u_plus_v_gt_one);
    if N::all_true(uv_out_of_bounds) {
        return IntersectionResult::new_no_intersection()
    }

    let n = edge1.op_cross(*edge2).op_normalized();
    let t = (-s).op_dot(n) / ray.direction.value().op_dot(n);
    let t_lt_range_start = N::scalar_cmp::<cmp::Lt>(t, ray.t_range.start);
    let t_gte_range_end = N::scalar_cmp::<cmp::Gte>(t, ray.t_range.end);
    let t_out_of_bounds = N::bool_or(t_lt_range_start, t_gte_range_end);
    if N::all_true(t_out_of_bounds) {
        return IntersectionResult::new_no_intersection()
    }

    let is_invalid_intersection =
        N::bool_or(t_out_of_bounds,
                   N::bool_or(u_out_of_bounds, uv_out_of_bounds));

    IntersectionResult {
        t: N::scalar_conditional_set(is_invalid_intersection, N::scalar_inf(), t),
        beta: u,
        gamma: v
    }
}
