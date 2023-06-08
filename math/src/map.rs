use std::collections::HashMap;

use crate::{delaunay::{CsTriangulation, VertexType}, Boundary, voronoi::{VoronoiVertex, VoronoiRegion}, color::Colors};
use nalgebra_glm::Vec2;
use crate::spade::{FloatTriangulation, InsertionError, Triangulation};
use spade::handles::VoronoiVertex::{Inner, Outer};
use bracket_noise::prelude::*;

pub struct Map {
    triangulation: CsTriangulation,
    boundary: Boundary,
    regions: Vec<MapRegion>

}
impl Map {
    pub fn get_regions(&self) -> &Vec<MapRegion> {
        &self.regions
    }
}

pub struct MapRegion {
    pub site: Vec2,
    pub vertices: Vec<VoronoiVertex>,
    pub color: Colors
}

pub fn new_map(boundary: Boundary) -> Map
{
    let triangulation = init_rand_points().unwrap();
    let mut regions = extract_voronoi_regions(&triangulation, &boundary);
    let elevation_map = assign_elevation_map(&regions);
    let mut map_regions = vec![];
    for (i, region) in regions.into_iter().enumerate() {
        let elevation = elevation_map[i];
        let color = if elevation < 0.5 {
            Colors::BLUE
        } else {
            Colors::GREEN
        };
        let mapr = MapRegion{site: region.site, vertices: region.vertices, color  };
        map_regions.push(mapr);
    }
    
    Map { triangulation, boundary, regions: map_regions }
}

fn generate_random_points() -> Vec<Vec2>{
    let GRID_SIZE = 64;
    let HALF_GRID = GRID_SIZE / 2;
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

fn init_rand_points() -> Result<CsTriangulation, InsertionError>{
    let mut result = CsTriangulation::new();
    let points = generate_random_points();
    for pt in points {
        result.insert(VertexType::new(pt.x, pt.y))?;
    }
    Ok(result)
}

fn extract_voronoi_regions(triangulation: &CsTriangulation, boundary: &Boundary) -> Vec<VoronoiRegion> {
    let (upper, lower) = {
        let upper = boundary.top_right();
        let upper = spade::Point2::new(upper.x, upper.y);
        let lower = boundary.bottom_left();
        let lower = spade::Point2::new(lower.x, lower.y);
        (upper, lower)
    };
    let mut regions = vec![];
    for vertex in triangulation.get_vertices_in_rectangle(lower, upper) {
        let region_site = vertex.data().position;
        let region = vertex.as_voronoi_face();
        let mut region_vertices = vec![];
        for edge in region.adjacent_edges() {
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
        regions.push(VoronoiRegion::new(Vec2::new(region_site.x, region_site.y), region_vertices));
    }
    regions
}

fn assign_elevation_map(regions: &Vec<VoronoiRegion>) -> Vec<f32> {
    let wave_length = 0.5;
    let mut noise = FastNoise::seeded(rand::random::<u64>());
    let GRID_SIZE = 64.;
    noise.set_noise_type(NoiseType::Simplex);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(5);
    noise.set_fractal_gain(0.6);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(2.0);
    let mut elevation_map = vec![];
    for (i, region) in regions.iter().enumerate() {
        let nx = region.site().x / GRID_SIZE - 0.5;
        let ny = region.site().y / GRID_SIZE - 0.5;
        
        let n = noise.get_noise(nx - wave_length, ny - wave_length);
        let elevation = (1. + n);
        let d = 2. * nx.abs().max(ny.abs());
        elevation_map.push((1. + elevation - d) / 2.);
    }
    elevation_map
}