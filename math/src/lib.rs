pub use nalgebra_glm as glm;
use nalgebra_glm::Vec2;
pub use rand;
pub use spade;
pub mod delaunay;
pub mod voronoi;
pub mod map;
pub mod color;

pub type RawMat4 = [[f32; 4]; 4];
#[inline]
pub fn float_eq(value: f32, compared: f32, epsilon: f32) -> bool {
    (value - compared).abs() <= epsilon
}
#[inline]
pub fn to_radians(degree: f32) -> f32 {
    degree.to_radians()
}


pub struct Perspective{
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32
}

impl Perspective {
    pub fn get(&self) -> glm::Mat4 {
        glm::perspective(self.aspect, self.fov, self.near, self.far)
    }
}

impl Default for Perspective {
    fn default() -> Self {
        Self {
            aspect: 1024. / 768.,
            fov: std::f64::consts::FRAC_PI_4 as f32,
            near: 0.1,
            far: 100.0
        }
    }
}

pub struct Ortho{
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32
}

impl Ortho {
    pub fn get(&self) -> glm::Mat4 {
        glm::ortho(self.left, self.right, self.bottom, self.top, self.near, self.far)
    }
    pub fn zoom(&mut self, k : f32) {
        self.left-=k;
        self.right+=k;
        self.top+=k;
        self.bottom-=k;
    }
}

impl Default for Ortho {
    fn default() -> Self {
        Self {
            left: -12.0,
            right: 12.0,
            bottom: -12.0,
            top: 12.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

#[derive(Debug)]
pub struct CameraSystem {
    pub pos: glm::Vec3,
    pub front: glm::Vec3,
    pub up: glm::Vec3
}

impl CameraSystem {
    pub fn view(&self) -> glm::Mat4 {
        glm::look_at(&self.pos, &(&self.pos + &self.front), &self.up)
    }
}

impl Default for CameraSystem {
    fn default() -> Self {
        Self {
            pos: glm::vec3(-1.5, 0., 25.0),
            front: glm::vec3(0., 0., -1.0),
            up: glm::vec3(0.0, 1.0, 0.0f32),
        }
    }
}
impl From<&CameraSystem> for glm::Mat4{
    fn from(cam: &CameraSystem) -> Self {
        cam.view()
    }
}


pub struct Transform {
    transform: glm::Mat4,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            transform: glm::identity(),
        }
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32){
        self.transform = glm::scale(&self.transform, &glm::vec3(x, y, z));
    }

    pub fn move_to(&mut self, x: f32, y: f32, z: f32) {
        self.transform.m14 = x;
        self.transform.m24 = y;
        self.transform.m34 = z;
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32){
        self.transform = glm::translate(&self.transform, &glm::vec3(x, y, z));
    }

    pub fn rotate(&mut self, angle: f32, axis: &glm::Vec3) {
        self.transform = glm::rotate(&self.transform, angle, axis);
    }

    pub fn get(&self) -> &glm::Mat4 {
        &self.transform
    }
    pub fn get_raw(&self) -> RawMat4 {
        self.transform.clone().into()
    }
    pub fn from(mat: glm::Mat4) -> Self {
        Self {
            transform: mat
        }
    }
}

impl From<&Transform> for RawMat4 {
    fn from(v: &Transform) -> Self {
        v.transform.clone().into()
    }
}

pub struct TransformBuilder(Transform);

impl TransformBuilder {
    pub fn new()-> Self {
        Self(Transform::new())
    }

    pub fn scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.0.scale(x, y, z);
        self
    }
    pub fn rotate(mut self, angle: f32, axis: &glm::Vec3) -> Self {
        self.0.rotate(angle, axis);
        self
    }
    pub fn translate(mut self, x: f32, y: f32, z: f32) -> Self {
        self.0.translate(x, y , z);
        self
    }
    pub fn build(self) -> Transform {
        self.0
    }
}

pub fn vector_projection(v1: Vec2, v2: Vec2) -> (Vec2, f32) {
    let k = v1.dot(&v2) / v2.dot(&v2);
    (k * v2, k)
}

#[derive(Clone)]
pub struct Segment {
    a: Vec2,
    b: Vec2,
}

impl Segment {
    pub fn new(startp: Vec2, endp: Vec2) -> Self {
        Self {
            a: startp, b: endp
        }
    }

    pub fn intercept_by_ray(&self, line_dir: Vec2, line_point: Vec2) -> Option<Vec2> {
        let ab = self.b - self.a;
        let ac = line_point - self.a;
        let denominator = scalar_cross_product(ab, line_dir);
        let numerator = scalar_cross_product(ab, ac);
        if float_eq(denominator, 0., 1e-6) {
            // println!("parallel");
            return None;
        }
        // println!("num: {} / denom: {}",numerator, denominator);
        let s = numerator / -denominator;
        if s < 0. {
            return None;
        }
        let i = line_point+line_dir*s;
        let t = if float_eq(self.a.x, self.b.x,1e-6) {
            (i.y - self.a.y) / (self.b.y - self.a.y)
        } else {
            (i.x - self.a.x) / (self.b.x - self.a.x)
        };
        if t >= 0. && t <= 1. {
            Some(i)
        } else {
            // println!("point outside segment A={:?},B={:?}, I={:?}, t={}", self.a, self.b, i,t);
            None
        }
    }

    pub fn is_point_on(&self, pt: Vec2) -> bool {
        let ab = self.b - self.a;
        let ap = pt - self.a;
        if !vector_parallel(ab, ap) || !vector_same_direction(ab, ap) {
            return false;
        }
        let len_ab = ab.dot(&ab);
        let len_ap = ap.dot(&ap);
        len_ab >= len_ap
    }

    pub fn startp(&self) -> Vec2 {
        self.a
    }
    pub fn endp(&self) -> Vec2 {
        self.b
    }

    pub fn point_closer_to_start(&self, pt: Vec2) -> bool {
        let ab = (self.b - self.a).norm_squared();
        let ap = (pt - self.a).norm_squared();
        (ab - ap) / ab <= 0.5
    }

    pub fn point_closer_to_end(&self, pt: Vec2) -> bool {
        !self.point_closer_to_start(pt)
    }
}

#[inline]
fn vector_same_direction(u: Vec2, v: Vec2) -> bool {
    float_eq(u.normalize().dot(&v.normalize()),1., 1e-6)
}

#[inline]
fn vector_parallel(u: Vec2, v: Vec2) -> bool {
    float_eq(scalar_cross_product(u,v), 0., 1e-6)
}

fn scalar_cross_product(u: Vec2, v: Vec2) -> f32 {
    u.x * v.y - u.y * v.x
}

#[derive(Clone)]
pub struct Boundary {
    origin: Vec2,
    width: f32,
    height: f32
}

impl Boundary {
    pub fn from_top_left(top_left_origin: Vec2, width: f32, height: f32) -> Self {
        Self {
            origin: top_left_origin,
            width, height
        }
    }
    pub fn top_left(&self) -> Vec2 {
        self.origin
    }
    pub fn bottom_left(&self) -> Vec2 {
        Vec2::new(self.origin.x, self.origin.y - self.height)
    }
    pub fn top_right(&self) -> Vec2 {
        Vec2::new(self.origin.x + self.width, self.origin.y)
    }
    pub fn bottom_right(&self) -> Vec2 {
        Vec2::new(self.origin.x + self.width, self.origin.y - self.height)
    }
}

#[cfg(test)]
mod segment_tests {
    use super::*;

    #[test]
    fn test_intersect_horizontal_vertical() {
        let a = Vec2::new(-6., 4.);
        let b = Vec2::new(2., 4.);
        let c = Vec2::new(-2., 2.);
        let d = Vec2::new(-2., 6.);
        let i = Vec2::new(-2., 4.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        if let Some(res) = ab.intercept_by_ray(cd, c) {
            assert_eq!(res.x, i.x);
            assert_eq!(res.y, i.y);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn test_intersect_horizontal() {
        let a = Vec2::new(-6., 4.);
        let b = Vec2::new(2., 4.);
        let c = Vec2::new(-4., 2.);
        let d = Vec2::new(2., 8.);
        let i = Vec2::new(-2., 4.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        if let Some(res) = ab.intercept_by_ray(cd, c) {
            assert_eq!(res.x, i.x);
            assert_eq!(res.y, i.y);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn test_intersect_vertical() {
        let a = Vec2::new(-6., 2.);
        let b = Vec2::new(2., 6.);
        let c = Vec2::new(-4., 2.);
        let d = Vec2::new(2., 8.);
        let i = Vec2::new(-2., 4.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        if let Some(res) = ab.intercept_by_ray(cd, c) {
            assert_eq!(res.x, i.x);
            assert_eq!(res.y, i.y);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn test_intersect() {
        let a = Vec2::new(6., 6.);
        let b = Vec2::new(14., 8.);
        let c = Vec2::new(8., 14.);
        let d = Vec2::new(12., 0.);
        let i = Vec2::new(10., 7.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        if let Some(res) = ab.intercept_by_ray(cd, c) {
            assert_eq!(res.x, i.x);
            assert_eq!(res.y, i.y);
        } else {
            panic!("fail");
        }
    }

    #[test]
    fn test_no_intercept_inverse_direction() {
        let a = Vec2::new(0., 0.);
        let b = Vec2::new(2., 2.);
        let c = Vec2::new(2., 1.);
        let d = Vec2::new(3., 1.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        assert!(ab.intercept_by_ray(cd, c).is_none());

    }

    #[test]
    fn test_no_intercept() {
        let a = Vec2::new(5., -5.);
        let b = Vec2::new(10., 0.);
        let c = Vec2::new(10., -5.);
        let d = Vec2::new(15., -3.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        assert!(ab.intercept_by_ray(cd, c).is_none());
    }

    #[test]
    fn test_parallel() {
        let a = Vec2::new(-5., -5.);
        let b = Vec2::new(0., 0.);
        let c = Vec2::new(0., -5.);
        let d = Vec2::new(5., 0.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        assert!(ab.intercept_by_ray(cd, c).is_none());
    }

    #[test]
    fn test_parallel_vertical() {
        let a = Vec2::new(0., 0.);
        let b = Vec2::new(0., 1.);
        let c = Vec2::new(1., 0.);
        let d = Vec2::new(1., 1.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        assert!(ab.intercept_by_ray(cd, c).is_none());
    }

    #[test]
    fn test_parallel_horizontal() {
        let a = Vec2::new(0., 0.);
        let b = Vec2::new(1., 0.);
        let c = Vec2::new(0., 1.);
        let d = Vec2::new(1., 1.);
        let ab = Segment::new(a, b);
        let cd = d - c;
        assert!(ab.intercept_by_ray(cd, c).is_none());
    }

    #[test]
    fn test_point_on() {
        let a = Vec2::new(-6.5, 2.);
        let b = Vec2::new(-5.5, 3.);
        let p = Vec2::new(-6., 2.5);
        let ab = Segment::new(a,b);
        assert!(ab.is_point_on(p));
    }

    #[test]
    fn test_point_on_vertical() {
        let a = Vec2::new(-5., 2.);
        let b = Vec2::new(-5., 3.);
        let p = Vec2::new(-5., 2.5);
        let ab = Segment::new(a,b);
        assert!(ab.is_point_on(p));
    }
    #[test]
    fn test_point_on_horizontal() {
        let a = Vec2::new(-6.5, 1.5);
        let b = Vec2::new(-5.5, 1.5);
        let p = Vec2::new(-6., 1.5);
        let ab = Segment::new(a,b);
        assert!(ab.is_point_on(p));
    }

    #[test]
    fn test_not_point_on() {
        let a = Vec2::new(-6.5, 2.);
        let b = Vec2::new(-5.5, 3.);
        let p = Vec2::new(-5.6, 2.2);
        let ab = Segment::new(a,b);
        assert!(!ab.is_point_on(p));
    }

    #[test]
    fn test_not_point_on_vertical() {
        let a = Vec2::new(-5., 2.);
        let b = Vec2::new(-5., 3.);
        let p = Vec2::new(-5.6, 2.2);
        let ab = Segment::new(a,b);
        assert!(!ab.is_point_on(p));
    }
    #[test]
    fn test_not_point_on_horizontal() {
        let a = Vec2::new(-6.5, 1.5);
        let b = Vec2::new(-5.5, 1.5);
        let p = Vec2::new(-5.6, 2.2);
        let ab = Segment::new(a,b);
        assert!(!ab.is_point_on(p));
    }
    #[test]
    fn test_not_point_on_but_on_line() {
        let a = Vec2::new(-7., 1.);
        let b = Vec2::new(-6., 1.);
        let p = Vec2::new(-7.5, 1.);
        let ab = Segment::new(a,b);
        assert!(!ab.is_point_on(p));
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;

    #[test]
    fn test_intercept_voronoi_4p() {
        let boundary = Boundary::from_top_left(Vec2::new(-10.0,10.0), 20., 20.);
        let top = Segment::new(boundary.top_right(), boundary.top_left());
        let right = Segment::new(boundary.bottom_right(), boundary.top_right());
        let bot = Segment::new(boundary.bottom_left(), boundary.bottom_right());
        let left = Segment::new(boundary.top_left(), boundary.bottom_left());
        let in_vertex1 = Vec2::new(0.0,2.0);
        let in_vertex2 = Vec2::new(2.,6.0);
        let bound_top_intersect = Vec2::new(2.,10.);
        let bound_left_intersect = Vec2::new(-10., -1.3333333);
        let bound_bot_intersect = Vec2::new(4., -10.);
        let bound_right_intersect = Vec2::new(10., -2.);
        let vert1_dir_left = Vec2::new(-6., -2.);
        let vert1_dir_bot = Vec2::new(2., -6.);
        let vert2_dir_top = Vec2::new(0., 12.);
        let vert2_dir_right = Vec2::new(4., -4.);
        {
            assert!(top.intercept_by_ray(vert1_dir_left, in_vertex1).is_none());
            assert!(right.intercept_by_ray(vert1_dir_left, in_vertex1).is_none());
            assert!(bot.intercept_by_ray(vert1_dir_left, in_vertex1).is_none());
            if let Some(point) = left.intercept_by_ray(vert1_dir_left, in_vertex1) {
                assert_eq!(point.x, bound_left_intersect.x);
                assert!(float_eq(point.y, bound_left_intersect.y, 1e-6));
            } else {
                panic!("fail");
            }

            assert!(top.intercept_by_ray(vert1_dir_bot, in_vertex1).is_none());
            assert!(right.intercept_by_ray(vert1_dir_bot, in_vertex1).is_none());
            assert!(left.intercept_by_ray(vert1_dir_bot, in_vertex1).is_none());
            if let Some(point) = bot.intercept_by_ray(vert1_dir_bot, in_vertex1) {
                assert_eq!(point.x, bound_bot_intersect.x);
                assert_eq!(point.y, bound_bot_intersect.y);
            } else {
                panic!("fail");
            }
        }
        {
            assert!(right.intercept_by_ray(vert2_dir_top, in_vertex2).is_none());
            assert!(bot.intercept_by_ray(vert2_dir_top, in_vertex2).is_none());
            assert!(left.intercept_by_ray(vert2_dir_top, in_vertex2).is_none());
            if let Some(point) = top.intercept_by_ray(vert2_dir_top, in_vertex2) {
                assert_eq!(point.x, bound_top_intersect.x);
                assert_eq!(point.y, bound_top_intersect.y);
            } else {
                panic!("fail");
            }

            assert!(top.intercept_by_ray(vert2_dir_right, in_vertex2).is_none());
            assert!(bot.intercept_by_ray(vert2_dir_right, in_vertex2).is_none());
            assert!(left.intercept_by_ray(vert2_dir_right, in_vertex2).is_none());
            if let Some(point) = right.intercept_by_ray(vert2_dir_right, in_vertex2) {
                assert_eq!(point.x, bound_right_intersect.x);
                assert_eq!(point.y, bound_right_intersect.y);
            } else {
                panic!("fail");
            }
        }
    }
}