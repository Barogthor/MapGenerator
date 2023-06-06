use nalgebra_glm::Vec2;

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
pub struct VoronoiRegionBounded {
    site: Vec2,
    vertices: Vec<VoronoiVertex>
}

impl VoronoiRegionBounded {
    pub fn new(site: Vec2, vertices: Vec<VoronoiVertex>) -> Self {
        Self {
            site,
            vertices,
        }
    }

    pub fn center(&self) -> Vec2 {
        self.site
    }

    pub fn vertices(&self) -> &Vec<VoronoiVertex> {
        &self.vertices
    }
}