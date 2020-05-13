#![feature(clamp)]

use cubetracer;

use gl;
use glutin::event;
use glutin::event::VirtualKeyCode as KeyCode;
use glutin::event::WindowEvent;
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use nalgebra::Vector3;
use utils::{FPSCounter, WinInput};

use world::{ChunkListener, World};
type CTX = ContextWrapper<PossiblyCurrent, glutin::window::Window>;

pub struct MyChunkListener {
    pub loaded_chunks: Vec<(i64, i64)>,
    pub unloaded_chunks: Vec<(i64, i64)>,
}

impl MyChunkListener {
    pub fn new() -> MyChunkListener {
        MyChunkListener {
            loaded_chunks: Vec::new(),
            unloaded_chunks: Vec::new(),
        }
    }

    pub fn update_renderer(&mut self) {
        //FIXME
        self.clear();
    }

    pub fn clear(&mut self) {
        self.loaded_chunks.clear();
        self.unloaded_chunks.clear();
    }
}

impl ChunkListener for MyChunkListener {
    fn chunk_load(&mut self, x: i64, y: i64) {
        self.loaded_chunks.push((x, y));
    }

    fn chunk_unload(&mut self, x: i64, y: i64) {
        self.unloaded_chunks.push((x, y));
    }
}

fn get_window_dim(context: &CTX) -> (u32, u32) {
    let dim = context.window().inner_size();
    (dim.width, dim.height)
}

fn main() {
    // --- Configuration ---
    let mut fps_counter = FPSCounter::new(60);
    let fov_steps = std::f32::consts::PI / 64.;
    let mut fov = std::f32::consts::PI / 2.;
    let keyboard_steps = 0.1;

    // --- World SetUp --
    let mut listener = MyChunkListener::new();

    let mut world = World::new();
    let mut player = world.create_player(&mut listener);

    //listener.update_renderer();

    // --- Window Helper ---
    let mut input_handler = WinInput::new();

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

    let (width, height) = get_window_dim(&context);

    let mut camera = cubetracer::Camera::new(
        width as f32,
        height as f32,
        Vector3::zeros(),
        Vector3::z(),
        Vector3::y(),
        std::f32::consts::PI / 2.,
        16. / 9.,
    );

    // --- Cube Tracer ---
    let mut cubetracer = cubetracer::CubeTracer::new(width, height).unwrap();

    let mut frame = 0;

    // --- Main loop ---
    event_loop.run(
        move |event, _, control_flow: &mut glutin::event_loop::ControlFlow| {
            *control_flow = glutin::event_loop::ControlFlow::Poll;

            match event {
                glutin::event::Event::LoopDestroyed => {
                    return;
                }
                glutin::event::Event::MainEventsCleared => {
                    // -- Update Cube Tracer program arguments --
                    cubetracer.args.set_camera(&camera).unwrap();

                    // -- Update World --
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
                    let v: Vec<KeyCode> = vec![KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D]
                        .into_iter()
                        .filter(|k| input_handler.is_pressed(*k))
                        .collect();
                    if v.len() > 0 {
                        println!("pushed_keys: {:?}", v);
                    }

                    let (width, height) = get_window_dim(&context);
                    cubetracer.compute_image(width, height).unwrap();

                    context.window().request_redraw();
                }
                glutin::event::Event::RedrawRequested(_) => {
                    context.swap_buffers().unwrap();
                    cubetracer.draw().unwrap();

                    if let Some(fps) = fps_counter.tick() {
                        println!("fps: {}", fps);
                    }
                }
                event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => input_handler.update(input),
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            glutin::event::MouseScrollDelta::LineDelta(dx, dy) => {
                                fov += (dx + dy) * fov_steps
                            }
                            _ => panic!("unexpected"),
                        };

                        fov = fov.clamp(std::f32::consts::PI / 16., std::f32::consts::PI / 2.);
                        camera.set_fov(fov);
                    }
                    glutin::event::WindowEvent::Resized(physical_size) => {
                        context.resize(physical_size);

                        camera.set_image_size(
                            physical_size.width as f32,
                            physical_size.height as f32,
                        );

                        cubetracer
                            .resize(physical_size.width, physical_size.height)
                            .unwrap();
                    }
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => (),
                },
                _ => (),
            };

            frame += 1;
        },
    )
}
