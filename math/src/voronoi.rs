use crate::delaunay::{CsTriangulation, VertexType};
use nalgebra_glm::Vec2;
use crate::spade::{FloatTriangulation, InsertionError, Triangulation};
use crate::spade::handles::VoronoiVertex::{Inner, Outer};
use crate::{Boundary};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum OuterType {
    Left, Top, Right, Bottom, TopLeftCorner, TopRightCorner, BottomRightCorner, BottomLeftCorner
}

#[derive(Debug, Clone)]
pub enum VoronoiVertex {
    Inner(Vec2),
    Outer(OuterType, Vec2)
}

#[derive(Debug)]
pub struct VoronoiRegion {
    pub(crate) site: Vec2,
    pub(crate) vertices: Vec<VoronoiVertex>
}

impl VoronoiRegion {
    pub fn new(site: Vec2, vertices: Vec<VoronoiVertex>) -> Self {
        Self {
            site,
            vertices,
        }
    }

    pub fn site(&self) -> Vec2 {
        self.site
    }

    pub fn vertices(&self) -> &Vec<VoronoiVertex> {
        &self.vertices
    }
}
