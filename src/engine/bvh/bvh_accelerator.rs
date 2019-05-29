extern crate time;

use super::aabb::*;
use super::splitter::*;

use std::f32;
use std::cmp::Ordering;
use std::ops::Range;
use std::collections::VecDeque;

use utilities::math::*;

#[cfg(all(target_arch = "x86_64", target_feature = "avx"))]
use utilities::simd::{
    intrin,
    SimdFloat8
};

/// Bounded Volume Hierarchy Accelerator
/// Each subtree can be represented by a slice of BVHAcceleratorNode objects
/// the first element in the slice is the root node, with the bounding box for
/// the subtree. It also contains the number of elements in this tree
#[derive(Debug)]
pub struct BVHAccelerator {
    nodes: Vec<BVHTree>
}

#[derive(Debug, Clone)]
//TODO try keeping Nodes and Leaves in different Vecs for better space efficiency
enum BVHTree {
    Node{number_of_nodes: usize, wrapper: AABoundingBox},
    Leaf{start: usize, end: usize, wrapper: AABoundingBox},
}

fn ensure_idx_exists<T: Clone>(vector: &mut Vec<T>, idx: usize, default: T) {
    while idx >= vector.len() {
        vector.push(default.clone())
    }
}

impl BVHAccelerator {
    pub fn new<T: HasAABoundingBox + HasSurfaceArea>(objects: &mut [T]) -> BVHAccelerator {
        let start_time = time::precise_time_s();

        let mut tree = Vec::<BVHTree>::new();
        let num_nodes = BVHAccelerator::build_tree(&mut tree, 0, objects, 0);

        let end_time = time::precise_time_s();
        println!("bvh build time: {}s", end_time - start_time);

        BVHAccelerator {
            nodes: tree,
        }
    }

    /// returns number of nodes in built tree
    fn build_tree<T: HasAABoundingBox + HasSurfaceArea> (
        tree_nodes: &mut Vec<BVHTree>, tree_index: usize,
        objects: &mut [T], start_index: usize
    ) -> usize {
        let objects_bbox = get_aa_bounding_box(objects);

        if objects.len() == 0 {
            return 0;
        }

        //find widest axis
        let dx = objects_bbox.upper.x - objects_bbox.lower.x;
        let dy = objects_bbox.upper.y - objects_bbox.lower.y;
        let dz = objects_bbox.upper.z - objects_bbox.lower.z;
        let x_is_largest = dx >= dy && dx >= dz;
        let y_is_largest = dy >= dx && dy >= dz;

        //sort objects
        if x_is_largest {
            objects.sort_by(|a: &T, b: &T| {
                if a.get_bounding_box_center().x > b.get_bounding_box_center().x {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        } else if y_is_largest {
            objects.sort_by(|a: &T, b: &T| {
                if a.get_bounding_box_center().y > b.get_bounding_box_center().y {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        } else {
            objects.sort_by(|a: &T, b: &T| {
                if a.get_bounding_box_center().z > b.get_bounding_box_center().z {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            })
        };

//        let splitter = MedianIndexSplitter {num_objects_in_leaf: 1};
        let splitter = SAHSubdivideGuessSplitter {
            number_of_subdivs: 50,
            sah_consts: SAHConstants {
                cost_traversal: 2.0,
                cost_triangle_intersection: 0.6
            }
        };
        let m = splitter.get_spliting_index(objects);

        ensure_idx_exists(tree_nodes, tree_index, BVHTree::Node {
            number_of_nodes: 1,
            wrapper: AABoundingBox::new()
        });

        if m == 0 {
            tree_nodes[tree_index] = BVHTree::Leaf {
                start: start_index,
                end: start_index + objects.len(),
                wrapper: objects_bbox
            };
            return 1;
        } else {
            let (left_objects, right_objects) = objects.split_at_mut(m);
            //left sub tree first
            let num_nodes_in_left =
                BVHAccelerator::build_tree(tree_nodes, tree_index + 1, left_objects, start_index);
            //right sub tree
            let num_nodes_in_right =
                BVHAccelerator::build_tree(tree_nodes, tree_index + 1 + num_nodes_in_left,
                                           right_objects, start_index + m);
            let num_nodes = 1 + num_nodes_in_right + num_nodes_in_left;
            tree_nodes[tree_index] = BVHTree::Node {
                number_of_nodes: num_nodes_in_left + num_nodes_in_right + 1,
                wrapper: objects_bbox
            };
            return num_nodes;
        }
    }
}

#[test]
fn test_bvh() {
    #[derive(Clone)]
    struct AABBArea {
        aabb: AABoundingBox,
        area: f32
    }
    impl HasAABoundingBox for AABBArea {
        fn aa_bounding_box_ref(&self) -> &AABoundingBox { &self.aabb }
    }
    impl HasSurfaceArea for AABBArea {
        fn surface_area(&self) -> f32 { self.area }
    }

    let aabbobjs = vec![
        AABBArea {
            aabb: AABoundingBox {
                lower: Vec3::new(0.0, 0.0, 0.0),
                upper: Vec3::new(1.0, 1.0, 1.0)
            },
            area: 1.0
        },
        AABBArea {
            aabb: AABoundingBox {
                lower: Vec3::new(1.0, 0.0, 0.0),
                upper: Vec3::new(2.0, 1.0, 1.0)
            },
            area: 1.0
        },
        AABBArea {
            aabb: AABoundingBox {
                lower: Vec3::new(2.0, 0.0, 0.0),
                upper: Vec3::new(3.0, 1.0, 1.0)
            },
            area: 1.0
        },
    ];

    let accelerator = BVHAccelerator::new(aabbobjs.clone().as_mut_slice());
    println!("{:#?}", accelerator);
}

impl BVHAccelerator {
    /// Intersect with bounded boxes.
    /// appends intersection_indices with indices of intersected boxes
    /// TODO make this function iterative. recursion is expensive
    fn intersect_box_intern(
        &self, ray: &AABBIntersectionRay,
        intersection_indices: &mut Vec<Range<usize>>
    ) {
        use self::BVHTree::*;
        let mut i_node = 0;
        while i_node < self.nodes.len() {
            let node: &BVHTree = &self.nodes[i_node];
            match node {
                &Node {number_of_nodes, ref wrapper} => {
                    if wrapper.intersects_with_bounding_box(ray) {
                        i_node += 1;
                    } else {
                        //skip current subtree
                        i_node += number_of_nodes;
                    }
                },
                &Leaf {start, end, ref wrapper} => {
                    if wrapper.intersects_with_bounding_box(ray) {
                        intersection_indices.push(start..end);
                    }
                    i_node += 1;
                }
            }
        }
    }

    /// Intersects with bounding boxes to find indices of objects that
    /// may intersect with the ray. Due to the way th bvh tree is build, the
    /// returned indices will already be sorted, so the caller can iterate through
    /// the objects in a way that benefits from cache locality
    pub fn intersect_boxes(&self, ray: &RayUnit) -> Vec<Range<usize>> {
        let mut indices = Vec::<Range<usize>>::new();
        let aabb_ray = AABBIntersectionRay::new(ray);
        self.intersect_box_intern(&aabb_ray, &mut indices);
        indices
    }
}
