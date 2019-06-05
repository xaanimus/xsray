use utilities::math::{Range, RayBase};
use utilities::multi_math::{MultiNum, MultiVec3, MultiNum1};
use engine::intersectable::Triangle;
use engine::intersect_util::{TriangleIntersectionResult, IntersectionResult};
use engine::intersect_util;
use utilities::cmp_util::cmp;

// TODO refactor rename to IntersectionResult
trait GenericIntersectionResult {
    fn intersection_t(&self) -> f32;
}

impl GenericIntersectionResult for TriangleIntersectionResult<MultiNum1> {
    fn intersection_t(&self) -> f32 {
        self.t
    }
}

trait Intersector<Primitive, N: MultiNum> {
    type IntersectionOutput: GenericIntersectionResult;

    fn new(bvh_to_parent_idx: &[usize], primitives: &[Primitive]) -> Self;

    /// attepmts to intersect with primitives specified by the given range.
    /// may try to intersect with a few more primitives than those that lie inside
    /// the given range.
    fn intersect(&self, ray: &RayBase<N>, bvh_index_range: Range<usize>) -> Self::IntersectionOutput;
}

struct TriangleIntersector<N: MultiNum> {
    position: Vec<MultiVec3<N>>,
    edge1: Vec<MultiVec3<N>>,
    edge2: Vec<MultiVec3<N>>
}

impl<N: MultiNum> Intersector<Triangle, N> for TriangleIntersector<N> {
    type IntersectionOutput = TriangleIntersectionResult<MultiNum1>;

    fn new(bvh_to_parent_idx: &[usize], primitives: &[Triangle]) -> Self {
        let position =  N::new_vec3_array_from_iter(
            primitives.iter().map(|triangle| triangle.positions[0]),
            0.0f32
        );
        let edge1 = N::new_vec3_array_from_iter(
            primitives.iter().map(|triangle| triangle.positions[1] - triangle.positions[0]),
            0.0f32
        );
        let edge2 = N::new_vec3_array_from_iter(
            primitives.iter().map(|triangle| triangle.positions[2] - triangle.positions[0]),
            0.0f32
        );

        TriangleIntersector {
            position, edge1, edge2
        }
    }

    fn intersect(&self, ray: &RayBase<N>, bvh_index_range: Range<usize>) -> Self::IntersectionOutput {
        let mut closest_intersection =
            TriangleIntersectionResult::<N>::new_no_intersection();
        for bvh_idx in bvh_index_range {
            let candidate_intersection = intersect_util::intersect_triangle::<N>(
                ray,
                &self.position[bvh_idx],
                &self.edge1[bvh_idx],
                &self.edge2[bvh_idx]
            );

            let candidate_lt_closest=
                N::scalar_cmp::<cmp::Lt>(candidate_intersection.t, closest_intersection.t);
            closest_intersection.t = N::scalar_conditional_set(
                candidate_lt_closest, candidate_intersection.t, closest_intersection.t);
            closest_intersection.gamma = N::scalar_conditional_set(
                candidate_lt_closest, candidate_intersection.gamma, closest_intersection.gamma);
            closest_intersection.beta = N::scalar_conditional_set(
                candidate_lt_closest, candidate_intersection.beta , closest_intersection.beta);
        }

        let min_idx = N::scalar_argmin(closest_intersection.t);
        IntersectionResult {
            t: N::get_scalar_arg(closest_intersection.t, min_idx),
            beta: N::get_scalar_arg(closest_intersection.beta, min_idx),
            gamma: N::get_scalar_arg(closest_intersection.gamma, min_idx)
        }
    }
}
