use glium::{
    index::{NoIndices, PrimitiveType},
    Display, DrawError, DrawParameters, Frame, Program, Surface, VertexBuffer,
};

use crate::{load_glsl, UniformStorage, VertexColor};

pub struct SitePipeline {
    vertexes_buffer: VertexBuffer<VertexColor>,
    indexes_buffer: NoIndices,
    program: Program,
}

impl SitePipeline {
    pub fn new(sites: Vec<VertexColor>, display: &Display) -> Self {
        let vertex_src = load_glsl("resources/shaders/voronoi_site.vs.glsl");
        let frag_src = load_glsl("resources/shaders/voronoi_site.fs.glsl");
        let program = glium::Program::from_source(display, &vertex_src, &frag_src, None).unwrap();
        let vertexes_buffer = VertexBuffer::new(display, &sites)
            .expect(&format!("failed to to build vertex buffer for sites"));
        let indexes_buffer = NoIndices(PrimitiveType::Points);
        Self {
            vertexes_buffer,
            indexes_buffer,
            program,
        }
    }

    pub fn update_vertexes(&mut self, display: &Display, data: Vec<VertexColor>) {
        self.vertexes_buffer = VertexBuffer::new(display, &data)
            .expect(&format!("failed to to build vertex buffer for sites"));
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        uniforms: &UniformStorage,
        draw_parameters: &DrawParameters<'_>,
    ) -> Result<(), DrawError> {
        frame.draw(
            &self.vertexes_buffer,
            &self.indexes_buffer,
            &self.program,
            uniforms,
            draw_parameters,
        )
    }
}

pub struct WirePipeline {
    vertexes_buffer: VertexBuffer<VertexColor>,
    indexes_buffer: NoIndices,
    program: Program,
}

impl WirePipeline {
    pub fn new(wires: Vec<VertexColor>, display: &Display) -> Self {
        let vertex_src = load_glsl("resources/shaders/voronoi_wire.vs.glsl");
        let frag_src = load_glsl("resources/shaders/voronoi_wire.fs.glsl");
        let program = glium::Program::from_source(display, &vertex_src, &frag_src, None).unwrap();
        let vertexes_buffer = VertexBuffer::new(display, &wires)
            .expect(&format!("failed to to build vertex buffer for wires"));
        let indexes_buffer = NoIndices(PrimitiveType::LinesList);
        Self {
            vertexes_buffer,
            indexes_buffer,
            program,
        }
    }

    pub fn update_vertexes(&mut self, display: &Display, data: Vec<VertexColor>) {
        self.vertexes_buffer = VertexBuffer::new(display, &data)
            .expect(&format!("failed to to build vertex buffer for wires"));
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        uniforms: &UniformStorage,
        draw_parameters: &DrawParameters<'_>,
    ) -> Result<(), DrawError> {
        frame.draw(
            &self.vertexes_buffer,
            &self.indexes_buffer,
            &self.program,
            uniforms,
            draw_parameters,
        )
    }
}
pub struct RegionPipeline {
    vertexes_buffer: VertexBuffer<VertexColor>,
    indexes_buffer: NoIndices,
    program: Program,
}

impl RegionPipeline {
    pub fn new(regions: Vec<VertexColor>, display: &Display) -> Self {
        let vertex_src = load_glsl("resources/shaders/map.vs.glsl");
        let frag_src = load_glsl("resources/shaders/map.fs.glsl");
        let program = glium::Program::from_source(display, &vertex_src, &frag_src, None).unwrap();
        let vertexes_buffer = VertexBuffer::new(display, &regions)
            .expect(&format!("failed to to build vertex buffer for regions"));
        let indexes_buffer = NoIndices(PrimitiveType::TrianglesList);
        Self {
            vertexes_buffer,
            indexes_buffer,
            program,
        }
    }

    pub fn update_vertexes(&mut self, display: &Display, data: Vec<VertexColor>) {
        self.vertexes_buffer = VertexBuffer::new(display, &data)
            .expect(&format!("failed to to build vertex buffer for regions"));
    }

    pub fn draw(
        &self,
        frame: &mut Frame,
        uniforms: &UniformStorage,
        draw_parameters: &DrawParameters<'_>,
    ) -> Result<(), DrawError> {
        frame.draw(
            &self.vertexes_buffer,
            &self.indexes_buffer,
            &self.program,
            uniforms,
            draw_parameters,
        )
    }
}
