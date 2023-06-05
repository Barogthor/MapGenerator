use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use math::delaunay::{CsTriangulation, VertexType};
use math::glm::Vec2;
use math::spade::{InsertionError, Triangulation};
use math::spade::handles::VoronoiVertex::{Inner, Outer};
use math::{Boundary, Segment, vector_projection};
use math::voronoi::{OuterType, VoronoiRegionBounded, VoronoiVertex};

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

#[derive(PartialOrd, Debug)]
struct Point2(f32, f32);
impl Ord for Point2 {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.eq(other) {
            Ordering::Equal
        } else if self.1 >= other.1 {
            Ordering::Greater
        } else if self.1 < other.1 {
            Ordering::Less
        } else if self.1.eq(&other.1) && self.0 >= other.0 {
            Ordering::Greater
        } else if self.1.eq(&other.1) && self.0 < other.0 {
            Ordering::Less
        } else {
            Ordering::Less
        }
    }
}
impl PartialEq for Point2 {
    fn eq(&self, other: &Self) -> bool {
        float_eq(self.0, other.0) && float_eq(self.1, other.1)
    }
}
impl Eq for Point2 {
}

pub fn basic_voronoi_example(boundary: Boundary) {
    let top = Segment::new(boundary.top_right(), boundary.top_left());
    let right = Segment::new(boundary.bottom_right(), boundary.top_right());
    let bot = Segment::new(boundary.bottom_left(), boundary.bottom_right());
    let left = Segment::new(boundary.top_left(), boundary.bottom_left());
    let triangulation = init_points2().unwrap();
    let mut vertices = BTreeSet::new();
    let mut regions = vec![];
    for vertex in triangulation.vertices() {
        let region_site = vertex.data().position;
        let region = vertex.as_voronoi_face();
        // println!("next region");
        let mut region_vertices = vec![];
        let mut tb = false;
        let mut rb = false;
        let mut bt = false;
        let mut lb = false;
        for edge in region.adjacent_edges() {
            // let edge_director = edge.direction_vector();
            // let edge_director = Vec2::new(edge_director.x, edge_director.y);

            // let start_pt = edge.from().position().map(|p| Point2(p.x, p.y));
            // let end_pt = edge.to().position().map(|p| Point2(p.x, p.y));

            match edge.as_undirected().vertices() {
                [Inner(from), Inner(to)] => {
                    let from = from.circumcenter();
                    let to = to.circumcenter();
                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(from.x, from.y)));
                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(to.x, to.y)));
                    vertices.insert(Point2(from.x, from.y));
                    vertices.insert(Point2(to.x, to.y));
                },
                [Inner(from), Outer(edge)] | [Outer(edge), Inner(from)] => {
                    let from = from.circumcenter();
                    let from = Vec2::new(from.x, from.y);
                    let dir = edge.direction_vector();
                    let dir = Vec2::new(dir.x, dir.y);
                    // println!("{:?}, {:?}",from, dir);
                    if let Some(point) = left.intercept_by_ray(dir, from)
                    {
                        lb = true;
                        region_vertices.push(VoronoiVertex::Outer(OuterType::Left, point));
                        vertices.insert(Point2(point.x, point.y));
                    }
                    else if let Some(point) = top.intercept_by_ray(dir, from)
                    {
                        tb = true;
                        region_vertices.push(VoronoiVertex::Outer(OuterType::Top, point));
                        vertices.insert(Point2(point.x, point.y));
                    }
                    else if let Some(point) = right.intercept_by_ray(dir, from)
                    {
                        rb = true;
                        region_vertices.push(VoronoiVertex::Outer(OuterType::Right, point));
                        vertices.insert(Point2(point.x, point.y));
                    }
                    else if let Some(point) = bot.intercept_by_ray(dir, from)
                    {
                        bt = true;
                        region_vertices.push(VoronoiVertex::Outer(OuterType::Bottom, point));
                        vertices.insert(Point2(point.x, point.y));
                    } else {
                        panic!("should not happen");
                    }
                }
                [_,_] => {}
            };
        }
        if tb && !rb && !bt && lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
        } else if tb && rb && !bt && !lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
        } else if !tb && rb && bt && !lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
        } else if !tb && !rb && bt && lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
        } else if tb && !rb && bt && !lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
            region_vertices.push(VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
        } else if !tb && rb && !bt && lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
            region_vertices.push(VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
        } else if tb && !rb && bt && !lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
            region_vertices.push(VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
        } else if !tb && rb && !bt && lb {
            region_vertices.push(VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
            region_vertices.push(VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
        }
        regions.push(VoronoiRegionBounded::new(Vec2::new(region_site.x, region_site.y), region_vertices));
    }
    println!("count={}, {:?}",vertices.len(), vertices);
    println!("region count={}", regions.len());
    for region in regions {
        println!("{:?}",region);
    }


    // for edge in triangulation.undirected_voronoi_edges() {
    //     match edge.vertices() {
    //         [Inner(from), Inner(to)] => {
    //             // sketch.add(
    //             //     SketchElement::line(
    //             //         convert_point(from.circumcenter()),
    //             //         convert_point(to.circumcenter()),
    //             //     )
    //             //         .stroke_color(LINE_COLOR),
    //             // );
    //         }
    //         [Inner(from), Outer(edge)] | [Outer(edge), Inner(from)] => {
    //             let from = convert_point(from.circumcenter());
    //             let to_direction = edge.direction_vector();
    //             let to_direction = Vec2::new(to_direction.x, to_direction.y);
    //
    //         }
    //         [Outer(_), Outer(_)] => {}
    //     }
    // }

    // for vertex in triangulation.vertices() {
    //     let position = vertex.position();
    //     min = min.zip(convert_point(position), f64::min);
    //     max = max.zip(convert_point(position), f64::max);
    // }

}