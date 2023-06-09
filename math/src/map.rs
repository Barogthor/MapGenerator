use std::{collections::HashMap, f32::consts::FRAC_PI_2};

use crate::{delaunay::{CsTriangulation, VertexType}, Boundary, voronoi::{VoronoiVertex, VoronoiRegion}, color::{PresetColors, HSL, RGB}};
use nalgebra_glm::Vec2;
use crate::spade::{FloatTriangulation, InsertionError, Triangulation};
use spade::handles::VoronoiVertex::{Inner, Outer};
use bracket_noise::prelude::*;

pub struct Map {
    triangulation: CsTriangulation,
    seed: u64,
    boundary: Boundary,
    regions: Vec<MapRegion>

}
impl Map {
    pub fn get_regions(&self) -> &Vec<MapRegion> {
        &self.regions
    }

    pub fn get_boundary(&self) -> &Boundary {
        &self.boundary
    }

    pub fn regenerate(&self, distance_fn: DistanceFn, reshape_fn: ReshapingFn) -> Self {
        new_map(self.boundary.clone(), self.seed, distance_fn, reshape_fn)
    }
}

pub struct MapRegion {
    pub site: Vec2,
    pub vertices: Vec<VoronoiVertex>,
    pub color: [f32;3]
}

pub fn new_map(boundary: Boundary, seed: u64, distance_fn: DistanceFn, reshape_fn: ReshapingFn) -> Map
{
    let triangulation = init_rand_points().unwrap();
    let mut regions = extract_voronoi_regions(&triangulation, &boundary);
    
    let elevation_map = assign_elevation_map(&regions, seed, distance_fn);
    let moisture_map = assign_moisture_map(&regions, seed);
    let mut map_regions = vec![];
    for (i, region) in regions.into_iter().enumerate() {
        let elevation = elevation_map[i];
        let color = get_biome_color(elevation_map[i], moisture_map[i]);
        let mapr = MapRegion{site: region.site, vertices: region.vertices, color: color.into()  };
        map_regions.push(mapr);
    }
    
    Map { triangulation, boundary, regions: map_regions, seed }
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

fn assign_elevation_map(regions: &Vec<VoronoiRegion>, seed: u64, distance_fn: DistanceFn) -> Vec<f32> {
    let wave_length = 0.5;
    let mut noise = FastNoise::seeded(seed);
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
        let elevation = 1. + n;
        // let d = 2. * nx.abs().max(ny.abs());
        let d = 2. * distance_fn.apply(nx, ny);
        elevation_map.push((1. + elevation - d) / 2.);
    }
    elevation_map
}

fn assign_moisture_map(regions: &Vec<VoronoiRegion>, seed: u64) -> Vec<f32> {
    let wave_length = 0.5;
    let mut noise = FastNoise::seeded(seed);
    let GRID_SIZE = 64.;
    noise.set_noise_type(NoiseType::Simplex);
    noise.set_fractal_type(FractalType::FBM);
    noise.set_fractal_octaves(5);
    noise.set_fractal_gain(0.6);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(2.0);
    let mut moisture_map = vec![];
    for (i, region) in regions.iter().enumerate() {
        let nx = region.site().x / GRID_SIZE - 0.5;
        let ny = region.site().y / GRID_SIZE - 0.5;
        
        let n = noise.get_noise(nx / wave_length, ny / wave_length);
        let m = (1. - n) / 2.;
        moisture_map.push(m);
    }
    moisture_map
}

fn get_biome_color(elevation: f32, moisture: f32) -> RGB {
    let elevation = (elevation - 0.5) * 2.;
    if elevation < 0. {
        let r = 48. + 48.*elevation;
        let g =  64. + 64.*elevation;
        let b = 127.+127.*elevation;
        RGB::new_f32(r / 255., g /255., b /255.)
    } else {
        let moisture = moisture * ( 1. - elevation);
        let elevation = elevation.powi(4);
        let (r, g, b) = {
            let r = 210. -  100. * moisture;
            let g =  185. -  45. * moisture;
            let b = 139. - 45. * moisture;
            let r = 255. * elevation +  r * (1. - elevation);
            let g = 255. * elevation +  g * (1. - elevation);
            let b = 255. * elevation +  b * (1. - elevation);
            (r,g,b)
        };
        RGB::new_f32(r / 255., g / 255., b / 255.)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum DistanceFn {
    Euclidean, Euclidean2, Hyperboloid, Squircle, SquareBump, TrigProduct, Diagonal, Manhattan
}
impl DistanceFn {
    pub fn apply(&self, x: f32, y: f32) -> f32 {
        match self {
            DistanceFn::Euclidean => x.hypot(y) / (2f32).sqrt(),
            DistanceFn::Euclidean2 => (x*x + y * y) / (2f32).sqrt(),
            DistanceFn::Hyperboloid => (x.hypot(y).hypot(0.2) - 0.2) / ((1f32).hypot(1.0).hypot(0.2) - 0.2),
            DistanceFn::Squircle => (x.powi(4) + y.powi(4)).sqrt() / (2f32).sqrt(),
            DistanceFn::SquareBump => 1.0 - (1.0-x*x) * (1.0 - y*y),
            DistanceFn::TrigProduct => 1.0 - (x*FRAC_PI_2).cos() * (y*FRAC_PI_2).cos(),
            DistanceFn::Diagonal => x.abs().max(y.abs()),
            DistanceFn::Manhattan => (x.abs() + y.abs()) / 2.0,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ReshapingFn {
    Input, Flat, Linear, LinearSteep, Clamped, Smooth, Smooth2, Smooth3, ClampedLess, SmoothLow, Smooth3Low, Archipelago
}
impl ReshapingFn {
    pub fn apply(&self, d: f32, e: f32) -> f32 {
        match self {
            ReshapingFn::Input => e,
            ReshapingFn::Flat => d,
            ReshapingFn::Linear => (e+d)/2.0,
            ReshapingFn::LinearSteep => {
                let low = (d-0.5).clamp(0.0, 1.0);
                let high = (d+0.5).clamp(0.0, 1.0);
                (1.0-e)*low + e*high
            },
            ReshapingFn::Clamped => e.clamp(d-0.49, d+0.49),
            ReshapingFn::Smooth => bezier3((d-0.5).max(0.0), 0.5, (d+0.5).min(1.0), e),
            ReshapingFn::Smooth2 => bezier3((d.powi(2)-0.5).max(0.0), 0.5, ((1.0-(1.0-d).powi(2))+0.5).min(1.0), e),
            ReshapingFn::Smooth3 => bezier3((d.powi(3)-0.5).max(0.0), 0.5, ((1.0-(1.0-d).powi(3))+0.5).min(1.0), e),
            ReshapingFn::ClampedLess => e.clamp(d.powi(2) - 0.45, (1.0 - (1.0 - d).powi(2)) + 0.45),
            ReshapingFn::SmoothLow => bezier3(0.0, 0.5, (d+0.5).min(1.0), e),
            ReshapingFn::Smooth3Low => bezier3(0.0, 0.5, ((1.0-(1.0-d).powi(3))+0.5).min(1.0), e),
            ReshapingFn::Archipelago => {
                let d = 1.0 - 2.0 * (d - 0.5).abs();
                bezier3((d-0.75).max(0.0), 5.0/12.0, (d+0.5).min(1.0), e)
            },
        }
    }
}

pub fn bezier3(p0: f32, p1: f32, p2: f32, t: f32) -> f32 {
    p1 + (1.0-t).powi(2) * (p0-p1) + t.powi(2) * (p2-p1)
}