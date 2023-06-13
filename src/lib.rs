use egui::{Grid, Layout, SidePanel, TopBottomPanel, Ui, Widget};
use egui_glium::EguiGlium;
use glium::uniforms::{UniformValue, Uniforms};
use glium::DrawParameters;
use math::color::PresetColors;
use math::map::{DistanceFn, ReshapingFn};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

pub mod pipeline;
pub mod tick;

pub fn draw_params() -> DrawParameters<'static> {
    use glium::{BackfaceCullingMode, Depth, DepthTest};
    DrawParameters {
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default()
        },
        point_size: Some(3.),
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
pub struct UniformStorage<'a>(HashMap<String, UniformValue<'a>>);

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
    pub regenerate: bool,
    pub reshape_fn: ReshapingFn,
    pub distance_fn: DistanceFn,
    pub seed: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open_debug: false,
            background_color: PresetColors::Other(40, 40, 40, 255).into(),
            frame_time: 0,
            quit: false,
            show_sites: false,
            regenerate: false,
            reshape_fn: ReshapingFn::Flat,
            distance_fn: DistanceFn::Diagonal,
            seed: 12345,
        }
    }
}

fn label<'a>(title: &'a str) -> impl Widget + 'a {
    let label = format!("{}:", title);
    move |ui: &mut Ui| ui.label(label)
}

fn show_widgets(ui: &mut Ui, state: &mut State) {
    ui.add(label("Background"));
    ui.color_edit_button_rgba_premultiplied(&mut state.background_color);
    ui.end_row();
    ui.add(label("Show sites"));
    ui.checkbox(&mut state.show_sites, "");
    ui.end_row();
    ui.add(label("Distance"));
    egui::ComboBox::from_id_source("distancefn").show_ui(ui, |ui| {
        ui.selectable_value(&mut state.distance_fn, DistanceFn::Euclidean, "Euclidean");
        ui.selectable_value(&mut state.distance_fn, DistanceFn::Euclidean2, "Euclidean2");
        ui.selectable_value(
            &mut state.distance_fn,
            DistanceFn::Hyperboloid,
            "Hyperboloid",
        );
        ui.selectable_value(&mut state.distance_fn, DistanceFn::Squircle, "Squircle");
        ui.selectable_value(&mut state.distance_fn, DistanceFn::SquareBump, "SquareBump");
        ui.selectable_value(
            &mut state.distance_fn,
            DistanceFn::TrigProduct,
            "TrigProduct",
        );
        ui.selectable_value(&mut state.distance_fn, DistanceFn::Diagonal, "Diagonal");
        ui.selectable_value(&mut state.distance_fn, DistanceFn::Manhattan, "Manhattan");
    });
    ui.end_row();
    ui.add(label("Reshaping"));
    egui::ComboBox::from_id_source("reshapingfn").show_ui(ui, |ui| {
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Input, "Input");
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Flat, "Flat");
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Clamped, "Clamped");
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Linear, "Linear");
        ui.selectable_value(
            &mut state.reshape_fn,
            ReshapingFn::LinearSteep,
            "LinearSteep",
        );
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Smooth, "Smooth");
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Smooth2, "Smooth2");
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Smooth3, "Smooth3");
        ui.selectable_value(
            &mut state.reshape_fn,
            ReshapingFn::ClampedLess,
            "ClampedLess",
        );
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::SmoothLow, "SmoothLow");
        ui.selectable_value(&mut state.reshape_fn, ReshapingFn::Smooth3Low, "Smooth3Low");
        ui.selectable_value(
            &mut state.reshape_fn,
            ReshapingFn::Archipelago,
            "Archipelago",
        );
    });
    ui.end_row();
}

pub fn show_window(egui: &mut EguiGlium, state: &mut State) {
    TopBottomPanel::top("my_top_bar").show(egui.ctx(), |ui| {
        ui.with_layout(Layout::left_to_right(), |ui| {
            if ui.button("Generate").clicked() {
                state.regenerate = true;
            }
            ui.add(label("Seed"));
            ui.add(egui::Slider::new(&mut state.seed, 1..=2u64.pow(16)).logarithmic(true));
        });
    });
    SidePanel::left("my_side_panel")
        .min_width(150.)
        .show(egui.ctx(), |ui| {
            Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    show_widgets(ui, state);
                });
        });
}
