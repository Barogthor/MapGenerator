use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use egui::{Grid, Layout, SidePanel, TopBottomPanel, Ui, Widget};
use egui_glium::EguiGlium;
use egui::Window as DWindow;
use glium::DrawParameters;
use glium::uniforms::{Uniforms, UniformValue};
use math::color::Colors;

pub mod tick;

pub fn draw_params() -> DrawParameters<'static> {
    use glium::{Depth, DepthTest, BackfaceCullingMode};
    DrawParameters {
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default()
        },
        point_size: Some(5.),
        backface_culling: BackfaceCullingMode::CullClockwise,
        ..DrawParameters::default()
    }
}

pub fn load_glsl(path: &str) -> String {
    let mut nice_shader = String::new();
    File::open(path)
        .unwrap()
        .read_to_string(&mut nice_shader)
        .unwrap();
    nice_shader
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, normal: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self {
            position: [x, y, z],
            normal,
            tex_coords,
        }
    }
}

glium::implement_vertex!(Vertex, position, normal, tex_coords);


#[derive(Copy, Clone, Debug)]
pub struct VertexColor {
    position: [f32; 3],
    color: [f32; 3],
}

impl VertexColor {
    pub fn new(x: f32, y: f32, z: f32, color: [f32; 3]) -> Self {
        Self {
            position: [x, y, z],
            color,
        }
    }
}

glium::implement_vertex!(VertexColor, position, color);


#[derive(Default, Clone)]
pub struct UniformStorage<'a> (HashMap<String, UniformValue<'a>>);

impl<'a> UniformStorage<'a> {
    pub fn add(&mut self, name: &str, value: UniformValue<'a>) {
        self.0.insert(name.to_string(), value);
    }
}

impl Uniforms for UniformStorage<'_> {
    fn visit_values<'a, F: FnMut(&str, UniformValue<'a>)>(&'a self, mut f: F) {
        self.0.iter().for_each(|(name, uniform)| {
            f(name, *uniform);
        })
    }
}




pub struct State {
    pub open_debug: bool,
    pub background_color: [f32; 4],
    pub frame_time: u128,
    pub quit: bool,
    pub show_sites: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open_debug: false,
            background_color: Colors::Other(40, 40, 40, 255).into(),
            frame_time: 0,
            quit: false,
            show_sites: false,
        }
    }
}


fn label<'a>(title: &'a str) -> impl Widget + 'a {
    let label = format!("{}:", title);
    move |ui: &mut Ui| {
        ui.label(label)
    }
}

fn show_widgets(ui: &mut Ui, state: &mut State) {
    ui.add(label("Background"));
    ui.color_edit_button_rgba_premultiplied(&mut state.background_color);
    ui.end_row();

    ui.checkbox(&mut state.show_sites, "Show sites");
}

pub fn show_window(egui: &mut EguiGlium, state: &mut State) {
    TopBottomPanel::top("my_top_bar").show(egui.ctx(), |ui| {
        ui.with_layout(Layout::left_to_right(), |ui| {
            ui.add(label("Hello world"));
            if ui.button("New window").clicked() {
                state.open_debug = true;
            }
        });
    });
    SidePanel::left("my_side_panel").min_width(150.).show(egui.ctx(), |ui| {
        ui.heading("Hello World!");
        if ui.button("Quit").clicked() {
            state.quit = true;
        }

        Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                show_widgets(ui, state);
            });
    });
    if state.open_debug {
        DWindow::new("Debug Window").min_width(150.).open(&mut state.open_debug).show(egui.ctx(), |ui| {
            ui.add(label("Debug label"));
        });
    }
}