pub use nalgebra_glm as glm;
pub use rand;
pub use spade;
pub mod delaunay;

pub type RawMat4 = [[f32; 4]; 4];

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