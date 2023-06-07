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
    let points = generate_random_points();
    for pt in points {
        result.insert(VertexType::new(pt.x, pt.y))?;
    }
    // result.insert(VertexType::new(2., 1.))?;
    // result.insert(VertexType::new(-1.4, 1.3))?;
    // result.insert(VertexType::new(1.0, 3.0))?;
    // result.insert(VertexType::new(5.0, 3.0))?;
    // result.insert(VertexType::new(5.0, 0.0))?;
    // result.insert(VertexType::new(3.0, -2.0))?;
    // result.insert(VertexType::new(1.0, -1.0))?;
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

pub fn basic_voronoi_example(boundary: Boundary) -> Vec<VoronoiRegionBounded> {
    let (upper, lower) = {
        let upper = boundary.top_right();
        let upper = spade::Point2::new(upper.x, upper.y);
        let lower = boundary.bottom_left();
        let lower = spade::Point2::new(lower.x, lower.y);
        (upper, lower)
    };
    let top = Segment::new(boundary.top_left(), boundary.top_right());
    let right = Segment::new(boundary.top_right(), boundary.bottom_right());
    let bot = Segment::new(boundary.bottom_right(), boundary.bottom_left());
    let left = Segment::new(boundary.bottom_left(), boundary.top_left());
    let triangulation = init_points2().unwrap();
    let mut vertices = BTreeSet::new();
    let mut regions = vec![];
    for vertex in triangulation.get_vertices_in_rectangle(lower, upper) {
        let region_site = vertex.data().position;
        let region = vertex.as_voronoi_face();
        let mut region_vertices = vec![];
        let mut tb = None;
        let mut rb = None;
        let mut bb = None;
        let mut lb = None;
        let mut out1 = None;
        let mut out2 = None;
        let mut region_has_outer = false;
        for (i, edge) in region.adjacent_edges().enumerate() {
            // let edge_director = edge.direction_vector();
            // let edge_director = Vec2::new(edge_director.x, edge_director.y);

            // let start_pt = edge.from().position().map(|p| Point2(p.x, p.y));
            // let end_pt = edge.to().position().map(|p| Point2(p.x, p.y));
            match [edge.from(), edge.to()]{
                [Inner(from), Inner(to)] => {
                    let from = from.circumcenter();
                    let to = to.circumcenter();

                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(to.x, to.y)));
                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(from.x, from.y)));
                    vertices.insert(Point2(from.x, from.y));
                    vertices.insert(Point2(to.x, to.y));
                },
                [Inner(from), Outer(edge)] | [Outer(edge), Inner(from)] => {
                    // region_has_outer = true;
                    // continue;
                    let from = from.circumcenter();
                    let from = Vec2::new(from.x, from.y);
                    let dir = edge.direction_vector();
                    let dir = Vec2::new(dir.x, dir.y);
                    // println!("{:?}, {:?}",from, dir);

                    if true {
                        let dir = if dir.norm_squared() > 10f32*10f32 {
                            dir.normalize() * 10.
                        } else {
                            dir
                        };
                        let outerv = VoronoiVertex::Inner(from+dir);
                        region_vertices.push(outerv.clone());
                        if out1.is_none() {
                            out1 = Some(outerv);
                        } else {
                            out2 = Some(outerv);
                        }
                    }
                    else if let Some(point) = left.intercept_by_ray(dir, from)
                    {
                        let outerv = VoronoiVertex::Outer(OuterType::Left, point);
                        region_vertices.push(outerv.clone());
                        vertices.insert(Point2(point.x, point.y));
                        lb = Some(region_vertices.len());
                        if out1.is_none() {
                            out1 = Some(outerv);
                        } else {
                            out2 = Some(outerv);
                        }
                    }
                    else if let Some(point) = top.intercept_by_ray(dir, from)
                    {
                        let outerv = VoronoiVertex::Outer(OuterType::Top, point);
                        region_vertices.push(outerv.clone());
                        vertices.insert(Point2(point.x, point.y));
                        tb = Some(region_vertices.len());
                        if out1.is_none() {
                            out1 = Some(outerv);
                        } else {
                            out2 = Some(outerv);
                        }
                    }
                    else if let Some(point) = right.intercept_by_ray(dir, from)
                    {
                        let outerv = VoronoiVertex::Outer(OuterType::Right, point);
                        region_vertices.push(outerv.clone());
                        vertices.insert(Point2(point.x, point.y));
                        rb = Some(region_vertices.len());
                        if out1.is_none() {
                            out1 = Some(outerv);
                        } else {
                            out2 = Some(outerv);
                        }
                    }
                    else if let Some(point) = bot.intercept_by_ray(dir, from)
                    {
                        let outerv = VoronoiVertex::Outer(OuterType::Bottom, point);
                        region_vertices.push(outerv.clone());
                        vertices.insert(Point2(point.x, point.y));
                        bb = Some(region_vertices.len());
                        if out1.is_none() {
                            out1 = Some(outerv);
                        } else {
                            out2 = Some(outerv);
                        }
                    } else {
                        panic!("should not happen");
                    }

                    region_vertices.push(VoronoiVertex::Inner(Vec2::new(from.x, from.y)));
                }
                [_,_] => {  }
            };
            if i > 0 {
                // region_vertices.push(region_vertices[region_vertices.len()-1].clone());
            }
        }
        //
        // if tb.is_some() && !rb.is_some() && !bb.is_some() && lb.is_some() {
        //     let mut t_index = tb.unwrap();
        //     let mut l_index = lb.unwrap();
        //     let out1 = region_vertices[t_index-1].clone();
        //     let out2 = region_vertices[l_index-1].clone();
        //     region_vertices.push(out1);
        //     region_vertices.push(out2);
        //     // region_vertices.insert(l_index,VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
        //     // region_vertices.insert(t_index, VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
        // } else if tb.is_some() && rb.is_some() && !bb.is_some() && !lb.is_some() {
        //     let mut t_index = tb.unwrap();
        //     let mut r_index = rb.unwrap();
        //     let out1 = region_vertices[t_index-1].clone();
        //     let out2 = region_vertices[r_index-1].clone();
        //     region_vertices.push(out1);
        //     region_vertices.push(out2);
        //     // region_vertices.insert(r_index,VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
        //     // region_vertices.insert(t_index, VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
        // } else if !tb.is_some() && rb.is_some() && bb.is_some() && !lb.is_some() {
        //     let mut r_index = rb.unwrap();
        //     let mut b_index = bb.unwrap();
        //     let out1 = region_vertices[r_index-1].clone();
        //     let out2 = region_vertices[b_index-1].clone();
        //     region_vertices.push(out1);
        //     region_vertices.push(out2);
        //     // region_vertices.insert(b_index,VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
        //     // region_vertices.insert(r_index, VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
        // } else if !tb.is_some() && !rb.is_some() && bb.is_some() && lb.is_some() {
        //     let mut b_index = bb.unwrap();
        //     let mut l_index = lb.unwrap();
        //     let out1 = region_vertices[b_index-1].clone();
        //     let out2 = region_vertices[l_index-1].clone();
        //     region_vertices.push(out1);
        //     region_vertices.push(out2);
        //     // region_vertices.insert(b_index, VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
        //     // region_vertices.insert(l_index, VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
        // } else if tb.is_some() && !rb.is_some() && bb.is_some() && !lb.is_some() {
        //     let mut t_index = tb.unwrap();
        //     let mut b_index = bb.unwrap();
        //     let out1 = region_vertices[t_index-1].clone();
        //     let out2 = region_vertices[b_index-1].clone();
        //     region_vertices.push(out1);
        //     region_vertices.push(out2);
        //     // region_vertices.push(VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
        //     // region_vertices.push(VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
        // } else if !tb.is_some() && rb.is_some() && !bb.is_some() && lb.is_some() {
        //     let mut r_index = rb.unwrap();
        //     let mut l_index = lb.unwrap();
        //     let out1 = region_vertices[r_index-1].clone();
        //     let out2 = region_vertices[l_index-1].clone();
        //     region_vertices.push(out1);
        //     region_vertices.push(out2);
            // region_vertices.push(VoronoiVertex::Outer(OuterType::BottomRightCorner, bot.startp()));
            // region_vertices.push(VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
        // }
        // } else if tb.is_some() && !rb.is_some() && bb.is_some() && !lb.is_some() {
        //     let mut r_index = 0;
        //     let mut l_index = 0;
        //     region_vertices.push(VoronoiVertex::Outer(OuterType::BottomLeftCorner, left.startp()));
        //     region_vertices.push(VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
        // } else if !tb.is_some() && rb.is_some() && !bb.is_some() && lb.is_some() {
        //     region_vertices.push(VoronoiVertex::Outer(OuterType::TopLeftCorner, top.startp()));
        //     region_vertices.push(VoronoiVertex::Outer(OuterType::TopRightCorner, right.startp()));
        // }
        if out1.is_some() && out2.is_some() {
            region_vertices.push(out1.unwrap());
            region_vertices.push(out2.unwrap());
        }

        if !region_has_outer {
            regions.push(VoronoiRegionBounded::new(Vec2::new(region_site.x, region_site.y), region_vertices));
        }
    }
    // println!("count={}, {:?}",vertices.len(), vertices);
    println!("region count={}", regions.len());
    regions
}