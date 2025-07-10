use egui::ViewportId;
use egui_glium::EguiGlium;
use glium::backend::glutin::SimpleWindowBuilder;
use glium::glutin::surface::WindowSurface;
use glium::uniforms::AsUniformValue;
use glium::winit::application::ApplicationHandler;
use glium::winit::dpi::{PhysicalSize, Size};
use glium::winit::event::{StartCause, WindowEvent};
use glium::winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopBuilder};
use glium::winit::window::WindowId;
use glium::{glutin, winit, Display, Surface};

use math::color::PresetColors;
use math::glm::{vec3, Vec2};
use math::map::{new_map, Map};
use math::voronoi::VoronoiVertex::{self, Inner, Outer};
use math::{float_eq, Boundary, CameraSystem, Ortho, RawMat4, Transform, TransformBuilder};
use ui::winit::event::Event;
use ui::winit::event_loop::ControlFlow;
use ui::{Binding, Gesture, Input, LoopType};
use MapGenerator::pipeline::{RegionPipeline, SitePipeline, WirePipeline};
use MapGenerator::tick::{
    TickSystem, TICK_DRAW_ID, TICK_FRAME_ID, TICK_RENDER_EGUI_ID, TICK_RENDER_ID,
};
use MapGenerator::{draw_params, show_window, State, UniformStorage, VertexColor};

const WIDTH: f32 = 1920f32;
const HEIGHT: f32 = 1080f32;

fn extract_region_mesh(map: &Map) -> Vec<VertexColor> {
    let mut meshes_vertices = vec![];
    for region in map.get_regions() {
        let site = region.site;
        let region_vertices = &region.vertices;
        for (i, vertice) in region_vertices.iter().enumerate() {
            let v2_index = (i + 1) % (region_vertices.len() - 1);
            let v1 = match vertice {
                VoronoiVertex::Inner(pt) | VoronoiVertex::Outer(_, pt) => *pt,
            };
            let v2 = match region_vertices[v2_index] {
                VoronoiVertex::Inner(pt) | VoronoiVertex::Outer(_, pt) => *pt,
            };
            meshes_vertices.push(VertexColor::new(site.x, site.y, 0.0, region.color));
            meshes_vertices.push(VertexColor::new(v1.x, v1.y, 0.0, region.color));
            meshes_vertices.push(VertexColor::new(v2.x, v2.y, 0.0, region.color));
        }
    }
    meshes_vertices
}

fn main() {
    let zoom_factor = 0.0;
    let state = State::default();
    let boundary = Boundary::from_top_left(Vec2::new(-32.0, 32.0), 64., 64.);
    let seed = 12345;
    let map = new_map(boundary, seed, state.distance_fn, state.reshape_fn);
    let (voronoi_sites, voronoi_wires) = setup_wires_and_sites_vertexes(&map);
    let region_vertexes = extract_region_mesh(&map);
    let camera_speed = 50.0f32;
    let draw_params = draw_params();
    let mut tick_system = TickSystem::new();
    tick_system.register_listener(TICK_FRAME_ID);
    tick_system.register_listener(TICK_DRAW_ID);
    tick_system.register_listener(TICK_RENDER_ID);
    let event_loop = EventLoop::builder().build().unwrap();
    let window_attributes = winit::window::Window::default_attributes()
        .with_title("Map generation")
        .with_inner_size(Size::Physical(PhysicalSize::new(
            WIDTH as u32,
            HEIGHT as u32,
        )));
    let _config_template_builder = glutin::config::ConfigTemplateBuilder::new();
    let (window, display) = SimpleWindowBuilder::new()
        .set_window_builder(window_attributes)
        .build(&event_loop);

    let egui = EguiGlium::new(ViewportId::ROOT, &display, &window, &event_loop);
    let input = Input::create();
    let binding = Binding::create();

    let region_pipeline = RegionPipeline::new(region_vertexes, &display);
    let site_pipeline = SitePipeline::new(voronoi_sites, &display);
    let wire_pipeline = WirePipeline::new(voronoi_wires, &display);

    let map_model = TransformBuilder::new().scale(0.5, 0.5, 0.5).build();

    let camera = CameraSystem::default();
    // let (mut w, mut h) = (
    //     display.get_framebuffer_dimensions().0,
    //     display.get_framebuffer_dimensions().1,
    // );
    let perspective = Ortho::default();
    let vp = perspective.get() * camera.view();
    let pre_vp: RawMat4 = vp.into();

    // Initialize App with all the context
    let context = AppContext {
        input,
        camera,
        ui_state: state,
        frame_diagnostic: tick_system,
        bindings: binding,
        map,
        region_pipeline,
        site_pipeline,
        wire_pipeline,
        map_model,
        camera_speed,
        zoom_factor,
        perspective,
        pre_vp,
        draw_params,
    };

    let mut app = App {
        egui_glium: egui,
        window,
        display,
        context,
    };

    event_loop.run_app(&mut app).unwrap();
}

struct AppContext {
    input: Input,
    camera: CameraSystem,
    ui_state: State,
    frame_diagnostic: TickSystem,
    bindings: Binding,
    map: Map,
    region_pipeline: RegionPipeline,
    site_pipeline: SitePipeline,
    wire_pipeline: WirePipeline,
    map_model: Transform,
    camera_speed: f32,
    zoom_factor: f32,
    perspective: Ortho,
    pre_vp: RawMat4,
    draw_params: glium::DrawParameters<'static>,
}

struct App {
    egui_glium: egui_glium::EguiGlium,
    window: winit::window::Window,
    display: glium::Display<WindowSurface>,
    context: AppContext,
}

impl App {
    fn redraw_window(&mut self) {
        self.context.frame_diagnostic.start_tick(TICK_RENDER_ID);

        self.egui_glium.run(&self.window, |egui_ctx| {
            show_window(egui_ctx, &mut self.context.ui_state);
        });

        let mut frame = self.display.draw();
        let bgc = {
            let c = &self.context.ui_state.background_color;
            (c[0], c[1], c[2], c[3])
        };
        frame.clear_color_and_depth(bgc, 1.);
        let view_pos: [f32; 3] = self.context.camera.pos.into();
        let view: RawMat4 = self.context.camera.view().into();

        // Draw regions
        {
            let model = self.context.map_model.get_raw();
            let mut my_storage = UniformStorage::default();
            my_storage.add("vp", self.context.pre_vp.as_uniform_value());
            my_storage.add("view", view.as_uniform_value());
            my_storage.add("model", model.as_uniform_value());
            my_storage.add("viewPos", view_pos.as_uniform_value());
            self.context
                .region_pipeline
                .draw(&mut frame, &my_storage, &self.context.draw_params)
                .unwrap();
        }

        // Draw sites if enabled
        if self.context.ui_state.show_sites {
            let model = self.context.map_model.get_raw();
            let mut my_storage = UniformStorage::default();
            my_storage.add("vp", self.context.pre_vp.as_uniform_value());
            my_storage.add("view", view.as_uniform_value());
            my_storage.add("model", model.as_uniform_value());
            my_storage.add("viewPos", view_pos.as_uniform_value());
            self.context
                .site_pipeline
                .draw(&mut frame, &my_storage, &self.context.draw_params)
                .unwrap();
        }

        // Draw wires
        {
            let model = self.context.map_model.get_raw();
            let mut my_storage = UniformStorage::default();
            my_storage.add("vp", self.context.pre_vp.as_uniform_value());
            my_storage.add("view", view.as_uniform_value());
            my_storage.add("model", model.as_uniform_value());
            my_storage.add("viewPos", view_pos.as_uniform_value());
            self.context
                .wire_pipeline
                .draw(&mut frame, &my_storage, &self.context.draw_params)
                .unwrap();
        }

        self.context
            .frame_diagnostic
            .start_tick(TICK_RENDER_EGUI_ID);
        self.egui_glium.paint(&self.display, &mut frame);
        self.context.frame_diagnostic.end_tick(TICK_RENDER_EGUI_ID);

        frame.finish().unwrap();
        self.context.frame_diagnostic.end_tick(TICK_RENDER_ID);
    }

    fn handle_input_events_window(&mut self) -> Option<()> {
        if self.context.input.poll_gesture(&self.context.bindings.exit)
            || self.context.input.poll_gesture(&Gesture::QuitTrigger)
            || self.context.ui_state.quit
        {
            return Some(());
        }

        let step = self
            .context
            .input
            .poll_analog2d(&self.context.bindings.scroll);
        if !float_eq(step.y, 0.0, 1e-3) {
            let future_zoom = self.context.zoom_factor - step.y * 10.;
            if future_zoom > -5. && future_zoom < 10. {
                self.context.zoom_factor = future_zoom;
                self.context.perspective.zoom(-step.y * 10.);
            }
        }

        if self
            .context
            .input
            .poll_gesture(&self.context.bindings.speedup)
        {
            self.context.camera_speed += 5.;
        }
        if self
            .context
            .input
            .poll_gesture(&self.context.bindings.speeddown)
            && self.context.camera_speed > 10.
        {
            self.context.camera_speed -= 5.;
        }

        self.context.pre_vp = (self.context.perspective.get() * self.context.camera.view()).into();

        if let Some(duration) = self.context.frame_diagnostic.duration_since_frame_start() {
            let step = self
                .context
                .input
                .poll_analog2d(&self.context.bindings.movement);

            if step.y != 0. || step.x != 0. {
                self.context.camera.pos +=
                    vec3(step.x, step.y, 0.0) * self.context.camera_speed * duration as f32;
            }
        }

        self.context.input.tick_reset();
        self.context.frame_diagnostic.end_tick(TICK_FRAME_ID);
        self.context.frame_diagnostic.update_time();
        if self.context.frame_diagnostic.should_reset() {
            self.context.frame_diagnostic.debug_tick_iteration();
            self.context.frame_diagnostic.reset();
        }
        self.context.frame_diagnostic.start_tick(TICK_FRAME_ID);
        None
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        use winit::event::WindowEvent;
        match &event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                self.display.resize((*new_size).into());
            }
            WindowEvent::RedrawRequested => {
                self.redraw_window();
            }
            _ => {}
        }
        self.context.input.update_window_event(&event);

        let event_response = self.egui_glium.on_event(&self.window, &event);

        if event_response.repaint {
            self.window.request_redraw();
        }

        if self.handle_input_events_window().is_some() {
            event_loop.exit();
        }

        // Handle map regeneration
        if self.context.ui_state.regenerate {
            self.context.ui_state.regenerate = false;
            self.context.map = self.context.map.regenerate(
                self.context.ui_state.seed,
                self.context.ui_state.distance_fn,
                self.context.ui_state.reshape_fn,
            );
            let regions_vertexes = extract_region_mesh(&self.context.map);
            let (sites_vertexes, wires_vertexes) =
                setup_wires_and_sites_vertexes(&self.context.map);
            self.context
                .region_pipeline
                .update_vertexes(&self.display, regions_vertexes);
            self.context
                .site_pipeline
                .update_vertexes(&self.display, sites_vertexes);
            self.context
                .wire_pipeline
                .update_vertexes(&self.display, wires_vertexes);
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::ResumeTimeReached { .. } => {}
            StartCause::WaitCancelled { .. } => {}
            StartCause::Poll => {}
            StartCause::Init => {
                self.context.frame_diagnostic.start_tick(TICK_FRAME_ID);
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: ()) {}

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.context.input.update_device_event(&event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {}

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {}
}

fn setup_wires_and_sites_vertexes(map: &Map) -> (Vec<VertexColor>, Vec<VertexColor>) {
    let mut voronoi_wires = vec![];
    let mut voronoi_sites = vec![];
    let boundary = map.get_boundary();
    voronoi_wires.push(VertexColor::new(
        boundary.top_left().x,
        boundary.top_left().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.top_right().x,
        boundary.top_right().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.top_right().x,
        boundary.top_right().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.bottom_right().x,
        boundary.bottom_right().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.bottom_right().x,
        boundary.bottom_right().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.bottom_left().x,
        boundary.bottom_left().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.bottom_left().x,
        boundary.bottom_left().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    voronoi_wires.push(VertexColor::new(
        boundary.top_left().x,
        boundary.top_left().y,
        1.0,
        PresetColors::TEAL.into(),
    ));
    for region in map.get_regions() {
        let site = region.site;
        voronoi_sites.push(VertexColor::new(
            site.x,
            site.y,
            1.0,
            PresetColors::RED.into(),
        ));
        for vertex in &region.vertices {
            match vertex {
                Inner(pt) | Outer(_, pt) => {
                    let v = VertexColor::new(pt.x, pt.y, 1.0, PresetColors::BLACK.into());
                    voronoi_wires.push(v);
                }
            }
        }
    }
    (voronoi_sites, voronoi_wires)
}
