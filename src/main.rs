use egui_glium::EguiGlium;
use glium::glutin::dpi::{PhysicalSize, Size};
use glium::glutin::window::WindowBuilder;
use glium::glutin::GlProfile;
use glium::uniforms::AsUniformValue;
use glium::Surface;
use math::color::PresetColors;
use math::glm::{vec3, Vec2};
use math::map::{new_map, Map};
use math::voronoi::VoronoiVertex::{self, Inner, Outer};
use math::{float_eq, Boundary, CameraSystem, Ortho, RawMat4, TransformBuilder};
use ui::winit::event::{Event, StartCause};
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
    let mut zoom_factor = 0.0;
    let mut state = State::default();
    let boundary = Boundary::from_top_left(Vec2::new(-32.0, 32.0), 64., 64.);
    let seed = 12345;
    let mut map = new_map(boundary, seed, state.distance_fn, state.reshape_fn);
    let (voronoi_sites, voronoi_wires) = setup_wires_and_sites_vertexes(&map);
    let region_vertexes = extract_region_mesh(&map);
    let mut camera_speed = 50.0f32;
    let draw_params = draw_params();
    let mut tick_system = TickSystem::new();
    tick_system.register_listener(TICK_FRAME_ID);
    tick_system.register_listener(TICK_DRAW_ID);
    tick_system.register_listener(TICK_RENDER_ID);
    let event_loop: LoopType = LoopType::new();
    let wb = WindowBuilder::new()
        .with_title("Map generation")
        .with_inner_size(Size::Physical(PhysicalSize::new(
            WIDTH as u32,
            HEIGHT as u32,
        )));
    let cb = glium::glutin::ContextBuilder::new().with_gl_profile(GlProfile::Core);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let mut egui = EguiGlium::new(&display);
    let mut input = Input::create();
    let binding = Binding::create();

    let mut region_pipeline = RegionPipeline::new(region_vertexes, &display);
    let mut site_pipeline = SitePipeline::new(voronoi_sites, &display);
    let mut wire_pipeline = WirePipeline::new(voronoi_wires, &display);

    let map_model = TransformBuilder::new().scale(0.5, 0.5, 0.5).build();

    let mut camera = CameraSystem::default();
    // let (mut w, mut h) = (
    //     display.get_framebuffer_dimensions().0,
    //     display.get_framebuffer_dimensions().1,
    // );
    let mut perspective = Ortho::default();
    let vp = perspective.get() * camera.view();
    let mut pre_vp: RawMat4 = vp.into();

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(cause) => match cause {
            StartCause::ResumeTimeReached { .. } => {}
            StartCause::WaitCancelled { .. } => {}
            StartCause::Poll => {}
            StartCause::Init => {
                tick_system.start_tick(TICK_FRAME_ID);
            }
        },
        Event::MainEventsCleared => {
            display.gl_window().window().request_redraw();
        }
        Event::RedrawRequested(_) => {
            tick_system.start_tick(TICK_RENDER_ID);

            egui.begin_frame(&display);

            show_window(&mut egui, &mut state);

            let (_needs_repaint, shapes) = egui.end_frame(&display);

            let mut frame = display.draw();
            let bgc = {
                let c = &state.background_color;
                (c[0], c[1], c[2], c[3])
            };
            frame.clear_color_and_depth(bgc, 1.);
            let view_pos: [f32; 3] = camera.pos.into();
            let view: RawMat4 = camera.view().into();
            {
                let model = map_model.get_raw();
                let mut my_storage = UniformStorage::default();
                my_storage.add("vp", pre_vp.as_uniform_value());
                my_storage.add("view", view.as_uniform_value());
                my_storage.add("model", model.as_uniform_value());
                my_storage.add("viewPos", view_pos.as_uniform_value());
                region_pipeline
                    .draw(&mut frame, &my_storage, &draw_params)
                    .unwrap();
            }
            if state.show_sites {
                let model = map_model.get_raw();
                let mut my_storage = UniformStorage::default();
                my_storage.add("vp", pre_vp.as_uniform_value());
                my_storage.add("view", view.as_uniform_value());
                my_storage.add("model", model.as_uniform_value());
                my_storage.add("viewPos", view_pos.as_uniform_value());
                site_pipeline
                    .draw(&mut frame, &my_storage, &draw_params)
                    .unwrap();
            }
            {
                let model = map_model.get_raw();
                let mut my_storage = UniformStorage::default();
                my_storage.add("vp", pre_vp.as_uniform_value());
                my_storage.add("view", view.as_uniform_value());
                my_storage.add("model", model.as_uniform_value());
                my_storage.add("viewPos", view_pos.as_uniform_value());
                wire_pipeline
                    .draw(&mut frame, &my_storage, &draw_params)
                    .unwrap();
            }

            tick_system.start_tick(TICK_RENDER_EGUI_ID);

            egui.paint(&display, &mut frame, shapes);
            tick_system.end_tick(TICK_RENDER_EGUI_ID);

            frame.finish().unwrap();
            tick_system.end_tick(TICK_RENDER_ID);
            // tick_system.debug_tick(TICK_RENDER_ID);
        }
        Event::RedrawEventsCleared => {
            if input.poll_gesture(&binding.exit)
                || input.poll_gesture(&Gesture::QuitTrigger)
                || state.quit
            {
                *control_flow = ControlFlow::Exit;
            }
            // if input.poll_gesture(&binding.toggle_mouse) {
            //     let window_context = display.gl_window();
            //     let window = window_context.window();
            //     // let win_pos = window.set_cursor_visible(false);
            //     let mouse = &input.poll_analog2d(&binding.look);
            //
            //     w = display.get_framebuffer_dimensions().0;
            //     h = display.get_framebuffer_dimensions().1;
            //     window.set_cursor_position(PhysicalPosition::new(w / 2, h / 2)).unwrap();
            //     yaw += mouse.x;
            //     pitch += mouse.y;
            //     if pitch > PITCH_MAX {
            //         pitch = PITCH_MAX;
            //     }
            //     if pitch < -PITCH_MAX {
            //         pitch = -PITCH_MAX;
            //     }
            //     let direction = vec3(
            //         yaw.cos() * pitch.cos(),
            //         pitch.sin(),
            //         yaw.sin() * pitch.cos(),
            //     );
            //     camera.front = direction.normalize();
            //     // light_spot.direction.data = direction.normalize();
            // }
            let step = input.poll_analog2d(&binding.scroll);
            if !float_eq(step.y, 0.0, 1e-3) {
                let future_zoom = zoom_factor - step.y * 10.;
                if future_zoom > -5. && future_zoom < 10. {
                    zoom_factor = future_zoom;
                    perspective.zoom(-step.y * 10.);
                }
            }

            if input.poll_gesture(&binding.speedup) {
                camera_speed += 5.;
            }
            if input.poll_gesture(&binding.speeddown) && camera_speed > 10. {
                camera_speed -= 5.;
            }

            pre_vp = (perspective.get() * camera.view()).into();

            if let Some(duration) = tick_system.duration_since_frame_start() {
                let step = input.poll_analog2d(&binding.movement);

                if step.y != 0. || step.x != 0. {
                    camera.pos += vec3(step.x, step.y, 0.0) * camera_speed * duration as f32;
                    // println!("{}", camera.pos);
                }
            }
            input.tick_reset();
            tick_system.end_tick(TICK_FRAME_ID);
            // tick_system.debug_tick(TICK_FRAME_ID);
            tick_system.update_time();
            if tick_system.should_reset() {
                tick_system.debug_tick_iteration();
                tick_system.reset();
            }
            tick_system.start_tick(TICK_FRAME_ID);
        }
        _ => {
            match &event {
                Event::WindowEvent { event, .. } => {
                    if egui.is_quit_event(event) {
                        *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    }
                    egui.on_event(event)
                }
                _ => {}
            }
            input.update(&event);
            if state.regenerate {
                state.regenerate = false;
                map = map.regenerate(state.seed, state.distance_fn, state.reshape_fn);
                let regions_vertexes = extract_region_mesh(&map);
                let (sites_vertexes, wires_vertexes) = setup_wires_and_sites_vertexes(&map);
                region_pipeline.update_vertexes(&display, regions_vertexes);
                site_pipeline.update_vertexes(&display, sites_vertexes);
                wire_pipeline.update_vertexes(&display, wires_vertexes);
            }
        }
    });
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
