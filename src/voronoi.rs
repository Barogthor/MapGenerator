use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use math::delaunay::{CsTriangulation, VertexType};
use math::glm::Vec2;
use math::spade::{FloatTriangulation, InsertionError, Triangulation};
use math::spade::handles::VoronoiVertex::{Inner, Outer};
use math::{Boundary, Segment, spade, vector_projection};
use math::voronoi::{generate_random_points, OuterType, VoronoiRegionBounded, VoronoiVertex};

#[inline]
fn float_eq(value: f32, compared: f32) -> bool {
    (value - compared).abs() < 1e-9
}

fn init_points1() -> Result<CsTriangulation, InsertionError>{
    let mut result = CsTriangulation::new();

    result.insert(VertexType::new(-4.0, 4.0))?;
    result.insert(VertexType::new(-2.0, -2.0))?;
    result.insert(VertexType::new(8.0, 4.0))?;
    result.insert(VertexType::new(4.0, 0.0))?;
    Ok(result)
}

fn init_points2() -> Result<CsTriangulation, InsertionError>{
    let mut result = CsTriangulation::new();
    result.insert(VertexType::new(2., 1.))?;
    result.insert(VertexType::new(-1.4, 1.3))?;
    result.insert(VertexType::new(1.0, 3.0))?;
    result.insert(VertexType::new(5.0, 3.0))?;
    result.insert(VertexType::new(5.0, 0.0))?;
    result.insert(VertexType::new(3.0, -2.0))?;
    result.insert(VertexType::new(1.0, -1.0))?;
    Ok(result)
}

fn init_rand_points() -> Result<CsTriangulation, InsertionError>{
    let mut result = CsTriangulation::new();
    let points = generate_random_points();
    for pt in points {
        result.insert(VertexType::new(pt.x, pt.y))?;
    }
    Ok(result)
}

pub fn basic_voronoi_example(boundary: Boundary) -> Vec<VoronoiRegionBounded> {
    let (upper, lower) = {
        let upper = boundary.top_right();
        let upper = spade::Point2::new(upper.x, upper.y);
        let lower = boundary.bottom_left();
        let lower = spade::Point2::new(lower.x, lower.y);
        (upper, lower)
    };
    let triangulation = init_rand_points().unwrap();
    let mut regions = vec![];
    for vertex in triangulation.get_vertices_in_rectangle(lower, upper) {
        let region_site = vertex.data().position;
        let region = vertex.as_voronoi_face();
        let mut region_vertices = vec![];
        for (i, edge) in region.adjacent_edges().enumerate() {
            match [edge.from(), edge.to()]{
                [Inner(from), Inner(to)] => {
                    let from = from.circumcenter();
                    let to = to.circumcenter();

                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(to.x, to.y)));
                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(from.x, from.y)));
                },
                [Inner(from), Outer(edge)] | [Outer(edge), Inner(from)] => {
                    let from = from.circumcenter();
                    let from = Vec2::new(from.x, from.y);
                    let dir = edge.direction_vector();
                    let dir = Vec2::new(dir.x, dir.y);

                    let dir = if dir.norm_squared() > 10f32*10f32 {
                        dir.normalize() * 10.
                    } else {
                        dir
                    };
                    let outerv = VoronoiVertex::Inner(from+dir);
                    region_vertices.push(outerv.clone());
                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(from.x, from.y)));
                }
                [_,_] => {  }
            };
        }
        regions.push(VoronoiRegionBounded::new(Vec2::new(region_site.x, region_site.y), region_vertices));
    }
    regions
}