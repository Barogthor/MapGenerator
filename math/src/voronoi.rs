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

pub fn generate_random_points() -> Vec<Vec2>{
    let GRIDSIZE = 64;
    let HALF_GRID = GRIDSIZE / 2;
    let JITTER = 0.5f32;

    let mut points = vec![];

    for x in -HALF_GRID..HALF_GRID {
        for y in -HALF_GRID..HALF_GRID {
            let x = x as f32;
            let y = y as f32;
            let x_displace = JITTER * (rand::random::<f32>() - rand::random::<f32>());
            let y_displace = JITTER * (rand::random::<f32>() - rand::random::<f32>());
            points.push(Vec2::new(x + x_displace, y + y_displace));
        }
    }
    points
}