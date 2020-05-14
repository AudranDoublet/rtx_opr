use cubetracer;

use gl;
use glutin::dpi;
use glutin::event;
use glutin::event::VirtualKeyCode as KeyCode;
use glutin::event::WindowEvent;
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use nalgebra::{Vector2, Vector3};
use utils::framecounter::FrameCounter;
use utils::wininput;

use std::sync::RwLock;

use world::{create_main_world, ChunkListener, World};
type CTX = ContextWrapper<PossiblyCurrent, glutin::window::Window>;

pub struct MyChunkListener {
    pub loaded_chunks: RwLock<Vec<(i64, i64)>>,
    pub unloaded_chunks: RwLock<Vec<(i64, i64)>>,
}

impl MyChunkListener {
    pub fn new() -> MyChunkListener {
        MyChunkListener {
            loaded_chunks: RwLock::new(Vec::new()),
            unloaded_chunks: RwLock::new(Vec::new()),
        }
    }

    pub fn update_renderer(&self, _: &World) {
        self.clear();
    }

    pub fn clear(&self) {
        self.loaded_chunks.write().unwrap().clear();
        self.unloaded_chunks.write().unwrap().clear();
    }
}

impl ChunkListener for MyChunkListener {
    fn chunk_load(&self, x: i64, y: i64) {
        self.loaded_chunks.write().unwrap().push((x, y));
    }

    fn chunk_unload(&self, x: i64, y: i64) {
        self.unloaded_chunks.write().unwrap().push((x, y));
    }
}

fn get_window_dim(context: &CTX) -> (u32, u32) {
    let dim = context.window().inner_size();
    (dim.width, dim.height)
}

fn set_cursor_middle_window(context: &CTX) {
    let window = context.window();

    let window_size = window.inner_size();
    let center = dpi::Position::new(dpi::LogicalPosition::new(
        window_size.width as f32 / 2.,
        window_size.height as f32 / 2.,
    ));

    window.set_cursor_position(center).unwrap();
}

pub fn game(seed: isize, view_distance: usize) {
    // --- Configuration ---
    let mut frame_counter = FrameCounter::new(60);
    let fov_range = (std::f32::consts::PI / 16.)..(std::f32::consts::PI / 2.);

    // --- World SetUp --
    let listener = MyChunkListener::new();

    let world = create_main_world(seed);
    let mut player = world.create_player(view_distance, &listener);

    // FIXME main loop
    player.update(world, Vector3::z(), Vector3::x(), Vec::new(), 0.1);
    listener.update_renderer(&world);

    // --- Window Helper ---
    let mut input_handler = wininput::WinInput::default();

    // --- Build Window ---
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new().with_title("Audran is stupid");

    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_double_buffer(Some(true))
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 3)))
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    context.window().set_cursor_visible(false);
    set_cursor_middle_window(&context);

    let (width, height) = get_window_dim(&context);

    let mut camera = cubetracer::Camera::new(
        width as f32,
        height as f32,
        Vector3::zeros(),
        Vector2::new(std::f32::consts::PI / 2.0, 0.0),
        fov_range.start + (fov_range.end - fov_range.start) / 2.,
        16. / 9.,
    );

    // --- Cube Tracer ---
    let mut cubetracer = cubetracer::CubeTracer::new(width, height).unwrap();

    // --- Main loop ---
    event_loop.run(
        move |event, _, control_flow: &mut glutin::event_loop::ControlFlow| {
            *control_flow = glutin::event_loop::ControlFlow::Poll;

            match event {
                glutin::event::Event::LoopDestroyed => return,
                glutin::event::Event::MainEventsCleared => {
                    let delta_time = frame_counter.delta_time();

                    // --- Process inputs ---
                    if input_handler.updated(wininput::StateChange::MouseScroll) {
                        let fov = fov_range.start
                            + input_handler.get_scroll() * (fov_range.end - fov_range.start);
                        camera.set_fov(fov)
                    }

                    if input_handler.updated(wininput::StateChange::MouseMotion) {
                        let offset = input_handler.get_mouse_offset() * delta_time;
                        camera.reorient(offset);
                    }

                    // FIXME: this is only for debugging purpose, remove me later
                    if input_handler.is_pressed(KeyCode::W) {
                        camera.origin += camera.forward() * delta_time;
                    }
                    if input_handler.is_pressed(KeyCode::A) {
                        camera.origin += camera.left() * delta_time;
                    }
                    if input_handler.is_pressed(KeyCode::S) {
                        camera.origin -= camera.forward() * delta_time;
                    }
                    if input_handler.is_pressed(KeyCode::D) {
                        camera.origin -= camera.left() * delta_time;
                    }
                    // FIXME-END

                    set_cursor_middle_window(&context);

                    // --- Update States ---

                    // - World -
                    /*
                    player.update(
                        &mut world,
                        &mut listener,
                        camera.forward(),
                        -camera.left(),
                        Vec::new(),
                        0.1,
                    );
                    */

                    // - Cube Tracer -
                    cubetracer.args.set_camera(&camera).unwrap();

                    context.window().request_redraw();
                }
                event::Event::RedrawRequested(_) => {
                    let (width, height) = get_window_dim(&context);

                    cubetracer.compute_image(width, height).unwrap();
                    cubetracer.draw().unwrap();

                    context.swap_buffers().unwrap();

                    if let Some(fps) = frame_counter.tick() {
                        println!("fps: {}", fps);
                    }
                }
                event::Event::DeviceEvent { event, .. } => input_handler.on_device_event(event),
                event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        input_handler.on_keyboard_input(input)
                    }
                    glutin::event::WindowEvent::Resized(physical_size) => {
                        context.resize(physical_size);
                        set_cursor_middle_window(&context);

                        camera.set_image_size(
                            physical_size.width as f32,
                            physical_size.height as f32,
                        );

                        cubetracer
                            .resize(physical_size.width, physical_size.height)
                            .unwrap();
                    }
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit
                    }
                    _ => (),
                },
                _ => (),
            };
        },
    )
}
