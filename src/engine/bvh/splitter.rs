//!Surface area heuristic

use super::aabb::*;

pub trait HasSurfaceArea {
    fn surface_area(&self) -> f32;
}

pub struct SAHConstants {
    pub cost_traversal: f32,
    pub cost_triangle_intersection: f32
}

fn compute_surface_area<T, FGetSurfaceArea: Fn(&T) -> f32>(
    objects: &[T],
    get_surface_area: &FGetSurfaceArea
) -> f32 {
    objects.iter()
        .map(|obj| get_surface_area(obj))
        .sum()
}

fn surface_area_heuristic<T, FGetSurfaceArea: Fn(&T) -> f32>(
    left_objects: &[T],
    right_objects: &[T],
    sah_constants: &SAHConstants,
    get_surface_area: &FGetSurfaceArea
) -> f32 {
    let left_surface_area = compute_surface_area(left_objects, get_surface_area);
    let right_surface_area = compute_surface_area(right_objects, get_surface_area);
    let total_surface_area = left_surface_area + right_surface_area;
    sah_constants.cost_traversal + sah_constants.cost_triangle_intersection * {
        left_surface_area / total_surface_area * left_objects.len() as f32 +
            right_surface_area / total_surface_area * right_objects.len() as f32
    }
}

pub trait BVHSplitter {
    /// Computes an index where the bvh should be split.
    /// returns 0 if there should be no split
    fn get_spliting_index<T, FGetBoundingBox, FGetSurfaceArea>(
        &self, sorted_objects: &[T],
        get_bbox: &FGetBoundingBox,
        get_surface_area: &FGetSurfaceArea
    ) -> usize
        where FGetBoundingBox: Fn(&T) -> AABoundingBox,
              FGetSurfaceArea: Fn(&T) -> f32;
}

pub struct MedianIndexSplitter {
    pub num_objects_in_leaf: usize
}
impl BVHSplitter for MedianIndexSplitter {
    fn get_spliting_index<T, FGetBoundingBox, FGetSurfaceArea>(
        &self, sorted_objects: &[T],
        get_bbox: &FGetBoundingBox,
        get_surface_area: &FGetSurfaceArea
    ) -> usize
        where FGetBoundingBox: Fn(&T) -> AABoundingBox,
              FGetSurfaceArea: Fn(&T) -> f32
    {
        if sorted_objects.len() <= self.num_objects_in_leaf {
            0
        } else {
            sorted_objects.len() / 2
        }
    }
}

pub struct SAHSubdivideGuessSplitter {
    pub number_of_subdivs: u32,
    pub sah_consts: SAHConstants
}
impl BVHSplitter for SAHSubdivideGuessSplitter {
    fn get_spliting_index<T, FGetBoundingBox, FGetSurfaceArea>(
        &self, sorted_objects: &[T],
        get_bbox: &FGetBoundingBox,
        get_surface_area: &FGetSurfaceArea
    ) -> usize
        where FGetBoundingBox: Fn(&T) -> AABoundingBox,
              FGetSurfaceArea: Fn(&T) -> f32
    {
        if sorted_objects.len() <= 1 {return 0;}

        //the last subdivision may or may not have this size
        let subdivision_size = (sorted_objects.len() as u32 / self.number_of_subdivs).max(1);
        let mut left_size = subdivision_size;
        let mut best_mid_point = 0u32;
        let mut best_cost =
            sorted_objects.len() as f32 * self.sah_consts.cost_triangle_intersection;
        while left_size < sorted_objects.len() as u32 {
            let (left_objects, right_objects) = sorted_objects.split_at(left_size as usize);
            let cost = surface_area_heuristic(
                left_objects, right_objects, &self.sah_consts, get_surface_area);
            if cost < best_cost {
                best_cost = cost;
                best_mid_point = left_size;
            }
            left_size += subdivision_size;
        }

        best_mid_point as usize
    }
}
