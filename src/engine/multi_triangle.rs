use super::intersectable;
use utilities::multi_math::*;

struct MultiTriangle<N: MultiNum> {
    position0: N::Vector3,
    edge1: N::Vector3,
    edge2: N::Vector3
}