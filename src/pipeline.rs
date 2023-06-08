use glium::{VertexBuffer, index::{NoIndices, PrimitiveType}, Display, Program, Frame, DrawError, DrawParameters, uniforms, Surface};

use crate::{VertexColor, UniformStorage};



pub struct SitePipeline {
    sites: Vec<VertexColor>,
    vertexes_buffer: VertexBuffer<VertexColor>,
    indexes_buffer: NoIndices,
    program: Program
}

impl SitePipeline {
    pub fn new(sites: Vec<VertexColor>, display: &Display, program: Program) -> Self {
        let vertexes_buffer = VertexBuffer::new(display, &sites).expect(&format!("failed to to build vertex buffer for sites"));
        let indexes_buffer = NoIndices(PrimitiveType::Points);
        Self {
            sites, vertexes_buffer, indexes_buffer, program
        }
    }

    pub fn draw(&self, frame: &mut Frame, uniforms: &UniformStorage, draw_parameters: &DrawParameters<'_>) -> Result<(), DrawError> 
    {
        frame.draw(&self.vertexes_buffer, &self.indexes_buffer, &self.program, uniforms, draw_parameters)
    }
}