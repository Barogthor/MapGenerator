use std::f32::consts::{FRAC_PI_2, PI};
use egui_glium::EguiGlium;
use glium::glutin::dpi::{PhysicalSize, Size};
use glium::glutin::GlProfile;
use glium::glutin::window::WindowBuilder;
use glium::{IndexBuffer, Surface, VertexBuffer};
use glium::uniforms::AsUniformValue;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use MapGenerator::{draw_params, load_glsl, show_window, State, UniformStorage, Vertex, VertexColor};
use MapGenerator::color::Colors;
use MapGenerator::tick::{TICK_DRAW_ID, TICK_FRAME_ID, TICK_RENDER_EGUI_ID, TICK_RENDER_ID, TickSystem};
use MapGenerator::voronoi::{basic_voronoi_example};
use math::{Boundary, CameraSystem, Ortho, Perspective, RawMat4, TransformBuilder};
use math::glm::{cross, normalize, Vec2, vec3};
use math::spade::Triangulation;
use math::voronoi::generate_random_points;
use math::voronoi::VoronoiVertex::{Inner, Outer};
use ui::{Binding, Gesture, Input, LoopType};
use ui::winit::dpi::PhysicalPosition;
use ui::winit::event::{Event, StartCause};
use ui::winit::event_loop::ControlFlow;

const PITCH_MAX: f32 = 1.55334f32;
const WIDTH: f32 = 1920f32;
const HEIGHT: f32 = 1080f32;
const FOV_MIN: f32 = 0.0174533f32;
const FOV_MAX: f32 = 0.785398f32;

fn main() {
    let boundary = Boundary::from_top_left(Vec2::new(-30.0,30.0), 60., 60.);
    let mut voronoi_wires = vec![];
    voronoi_wires.push(VertexColor::new(boundary.top_left().x, boundary.top_left().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.top_right().x, boundary.top_right().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.top_right().x, boundary.top_right().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.bottom_right().x, boundary.bottom_right().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.bottom_right().x, boundary.bottom_right().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.bottom_left().x, boundary.bottom_left().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.bottom_left().x, boundary.bottom_left().y, 0.0, Colors::BLUE.into()));
    voronoi_wires.push(VertexColor::new(boundary.top_left().x, boundary.top_left().y, 0.0, Colors::BLUE.into()));
    let map = basic_voronoi_example(boundary);
    let mut sites = vec![];
    for region in map {
        let center = region.center();
        sites.push(VertexColor::new(center.x, center.y, 0.0, Colors::RED.into()));
        for vertex in region.vertices() {
            match vertex {
                Inner(pt) | Outer(_, pt) => {
                    let v = VertexColor::new(pt.x, pt.y, 0.0, Colors::BLACK.into());
                    voronoi_wires.push(v);
                }
            }
        }
    }
    let mut camera_speed = 0.05f32;
    let draw_params = draw_params();
    let mut tick_system = TickSystem::new();
    tick_system.register_listener(TICK_FRAME_ID);
    tick_system.register_listener(TICK_DRAW_ID);
    tick_system.register_listener(TICK_RENDER_ID);
    let event_loop: LoopType = LoopType::new();
    let wb = WindowBuilder::new()
        .with_title("3D Playground")
        .with_inner_size(Size::Physical(PhysicalSize::new(WIDTH as u32, HEIGHT as u32)));
    let cb = glium::glutin::ContextBuilder::new().with_gl_profile(GlProfile::Core);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();
    let mut egui = EguiGlium::new(&display);
    let mut input = Input::create();
    let binding = Binding::create();

    let map_vertex_src = load_glsl("resources/shaders/map.vs.glsl");
    let map_fragment_src = load_glsl("resources/shaders/map.fs.glsl");
    let map_program =
        glium::Program::from_source(&display, &map_vertex_src, &map_fragment_src, None)
            .unwrap();
    let voronoi_site_vertex_src = load_glsl("resources/shaders/voronoi_site.vs.glsl");
    let voronoi_site_fragment_src = load_glsl("resources/shaders/voronoi_site.fs.glsl");
    let voronoi_site_program =
        glium::Program::from_source(&display, &voronoi_site_vertex_src, &voronoi_site_fragment_src, None)
            .unwrap();
    let voronoi_wire_vertex_src = load_glsl("resources/shaders/voronoi_wire.vs.glsl");
    let voronoi_wire_fragment_src = load_glsl("resources/shaders/voronoi_wire.fs.glsl");
    let voronoi_wire_program =
        glium::Program::from_source(&display, &voronoi_wire_vertex_src, &voronoi_wire_fragment_src, None)
            .unwrap();

    let site_vertexes = VertexBuffer::new(&display, &sites).unwrap();
    let site_indexes = glium::index::NoIndices(glium::index::PrimitiveType::Points);
    let voronoi_wire_vertexes = VertexBuffer::new(&display, &voronoi_wires).unwrap();
    let voronoi_wire_indexes = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    let map_model = TransformBuilder::new()
        .scale(0.5,0.5,0.5)
        .build();

    let mut camera = CameraSystem::default();
    let (mut w, mut h) = (display.get_framebuffer_dimensions().0, display.get_framebuffer_dimensions().1);
    let mut perspective = Ortho::default();
    let vp = perspective.get() * &camera.view();
    let mut pre_vp: RawMat4 = vp.into();
    // let (mut yaw, mut pitch) = (FRAC_PI_2 * 2., 0.0);
    let mut state = State::default();

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
                let mut my_storage =  UniformStorage::default();
                my_storage.add("vp", pre_vp.as_uniform_value());
                my_storage.add("view", view.as_uniform_value());
                my_storage.add("model", model.as_uniform_value());
                my_storage.add("viewPos", view_pos.as_uniform_value());
                frame.draw(&site_vertexes, &site_indexes, &voronoi_site_program, &my_storage, &draw_params).unwrap();
            }
            {
                let model = map_model.get_raw();
                let mut my_storage =  UniformStorage::default();
                my_storage.add("vp", pre_vp.as_uniform_value());
                my_storage.add("view", view.as_uniform_value());
                my_storage.add("model", model.as_uniform_value());
                my_storage.add("viewPos", view_pos.as_uniform_value());
                frame.draw(&voronoi_wire_vertexes, &voronoi_wire_indexes, &voronoi_wire_program, &my_storage, &draw_params).unwrap();
            }

            tick_system.start_tick(TICK_RENDER_EGUI_ID);

            egui.paint(&display, &mut frame, shapes);
            tick_system.end_tick(TICK_RENDER_EGUI_ID);

            frame.finish().unwrap();
            tick_system.end_tick(TICK_RENDER_ID);
            // tick_system.debug_tick(TICK_RENDER_ID);
        }
        Event::RedrawEventsCleared => {
            if input.poll_gesture(&binding.exit) || input.poll_gesture(&Gesture::QuitTrigger) || state.quit {
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
            // let step = input.poll_analog2d(&binding.scroll);
            // if !float_eq(step.y, 0.0, 1e-3) {
            //     perspective.fov -= step.y;
            //     if perspective.fov < FOV_MIN {
            //         perspective.fov = FOV_MIN;
            //     } else if perspective.fov > FOV_MAX {
            //         perspective.fov = FOV_MAX;
            //     }
            // }

            if input.poll_gesture(&binding.speedup) { camera_speed += 0.05; }
            if input.poll_gesture(&binding.speeddown) && camera_speed > 0.1 { camera_speed -= 0.05; }

            pre_vp = (perspective.get() * camera.view()).into();

            if let Some(duration) = tick_system.duration_since_frame_start() {
                let step = input.poll_analog2d(&binding.movement);

                if step.y != 0. || step.x != 0. {
                    camera.pos += vec3(step.x, step.y, 0.0) * camera_speed;
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
        },
        _ => {
            match &event {
                Event::WindowEvent { event, .. } => {
                    if egui.is_quit_event(&event) {
                        *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    }
                    egui.on_event(event)
                },
                _ => {}
            }
            input.update(&event);
        },
    });
}

