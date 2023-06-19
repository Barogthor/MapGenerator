use std::cmp::Ordering;

use nalgebra_glm::Vec2;

use crate::float_eq;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum OuterType {
    Left,
    Top,
    Right,
    Bottom,
    TopLeftCorner,
    TopRightCorner,
    BottomRightCorner,
    BottomLeftCorner,
}

#[derive(Debug, Clone)]
pub enum VoronoiVertex {
    Inner(Vec2),
    Outer(OuterType, Vec2),
}

#[derive(Debug)]
pub struct VoronoiRegion {
    pub(crate) site: Vec2,
    pub(crate) vertices: Vec<VoronoiVertex>,
}

impl VoronoiRegion {
    pub fn new(site: Vec2, vertices: Vec<VoronoiVertex>) -> Self {
        Self { site, vertices }
    }

    pub fn site(&self) -> Vec2 {
        self.site
    }

    pub fn vertices(&self) -> &Vec<VoronoiVertex> {
        &self.vertices
    }
}

pub struct VoronoiEdge {
    pub(crate) from: Vec2,
    pub(crate) to: Vec2,
}

#[derive(PartialOrd)]
pub struct VoronoiCorner {
    pub(crate) x: f32,
    pub(crate) y: f32,
}
impl VoronoiCorner {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Vec2> for VoronoiCorner {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl PartialEq for VoronoiCorner {
    fn eq(&self, other: &Self) -> bool {
        float_eq(self.x, other.x, 1e-6) && float_eq(self.y, other.y, 1e-6)
    }
}
impl Eq for VoronoiCorner {}

impl Ord for VoronoiCorner {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if float_eq(self.y, other.y, 1e-6) {
            if self.x > other.x {
                Ordering::Greater
            } else if self.x < other.x {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        } else if self.y > other.y {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}
