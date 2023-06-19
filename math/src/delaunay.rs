use std::ops::Deref;

use nalgebra_glm::Vec2;
use spade::{ConstrainedDelaunayTriangulation, DelaunayTriangulation, HasPosition, Point2};

pub struct VertexType {
    pub position: Vec2,
    pub radius: f32,
}

impl VertexType {
    pub fn new(x: f32, y: f32) -> Self {
        const DEFAULT_CIRCLE_RADIUS: f32 = 2.0;

        Self {
            position: Vec2::new(x, y),
            radius: DEFAULT_CIRCLE_RADIUS,
        }
    }
}

impl From<Vec2> for VertexType {
    fn from(value: Vec2) -> Self {
        const DEFAULT_CIRCLE_RADIUS: f32 = 2.0;
        Self {
            position: value,
            radius: DEFAULT_CIRCLE_RADIUS,
        }
    }
}

impl HasPosition for VertexType {
    type Scalar = f32;

    fn position(&self) -> Point2<Self::Scalar> {
        Point2::new(self.position.x, self.position.y)
    }
}

pub struct UndirectedEdgeType {
    // pub color: SketchColor,
}

impl AsRef<UndirectedEdgeType> for UndirectedEdgeType {
    fn as_ref(&self) -> &UndirectedEdgeType {
        self
    }
}

impl Default for UndirectedEdgeType {
    fn default() -> Self {
        Self {
            // color: SketchColor::BLACK,
        }
    }
}

#[derive(Default)]
pub struct DirectedEdgeType {}

#[derive(Clone, Copy, Debug)]
pub struct FaceType {
    // pub fill: SketchFill,
}

impl Default for FaceType {
    fn default() -> Self {
        Self {
            // fill: SketchFill::Solid(SketchColor::LIGHT_GRAY),
        }
    }
}

pub type NormalTriangulation =
    DelaunayTriangulation<VertexType, DirectedEdgeType, UndirectedEdgeType, FaceType>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct SpadeIndex(usize);
impl SpadeIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

impl Deref for SpadeIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
