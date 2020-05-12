use cubetracer;

use gl;
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use nalgebra::Vector3;

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
    // --- World SetUp --
    let mut listener = MyChunkListener::new();

    let mut world = World::new();
    let mut player = world.create_player(&mut listener);

    // FIXME main loop
    player.update(
        &mut world,
        &mut listener,
        Vector3::z(),
        Vector3::x(),
        Vec::new(),
        0.1,
    );

    //listener.update_renderer();

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

    let (width, height) = get_window_dim(&context);

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
                    let (width, height) = get_window_dim(&context);
                    cubetracer.args.set_roll(frame as f32 * 0.01).unwrap();
                    cubetracer.compute_image(width, height).unwrap();

                    context.window().request_redraw();

                    frame += 1;
                }
                glutin::event::Event::RedrawRequested(_) => {
                    context.swap_buffers().unwrap();
                    cubetracer.draw().unwrap();
                }
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::Resized(physical_size) => {
                        context.resize(physical_size);

                        cubetracer
                            .resize(physical_size.width, physical_size.height)
                            .unwrap();

                        //context.window().request_redraw();
                    }
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                    }
                    _ => (),
                },
                _ => (),
            };
        },
    )
}

/*
fn main() -> Result<(), cubetracer::GLError> {
}
*/
