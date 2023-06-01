use nalgebra_glm::{Scalar, Vec2};
use spade::{DelaunayTriangulation, HasPosition, Point2};

pub struct VertexType {
    position: Vec2,
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

pub type Triangulation =
DelaunayTriangulation<VertexType, DirectedEdgeType, UndirectedEdgeType, FaceType>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EdgeMode {
    Disabled,
    Undirected,
    Directed { reversed: bool },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConversionOptions {
    pub directed_edge_mode: EdgeMode,
    // pub vertex_stroke_color: SketchColor,
    // pub vertex_color: SketchColor,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            directed_edge_mode: EdgeMode::Undirected,
            // vertex_stroke_color: SketchColor::BLACK,
            // vertex_color: SketchColor::DIM_GRAY,
        }
    }
}