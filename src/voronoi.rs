use math::glm::{Vec2, vec2};

pub type Site = Vec2;

pub enum Event {
    Site(Site),
    Circle,
    Edge
}

pub fn voronoi() {
    let sites = vec![vec2(-4.0,4.0), vec2(-2.0, -2.0), vec2(8.0, 4.0), vec2(4.0, 0.0)];
    // let mut events = vec![];
}