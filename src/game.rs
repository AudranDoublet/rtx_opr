use crate::termidraw::TermiDrawer;
use gl;
use glutin::event;
use glutin::event::WindowEvent;
use glutin::event::{MouseButton, VirtualKeyCode as KeyCode};
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use nalgebra::{Vector2, Vector3};
use utils::framecounter::FrameCounter;
use utils::wininput;

use std::rc::Rc;

use termion::{raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use world::{create_main_world, Chunk, ChunkListener, PlayerInput};
type CTX = ContextWrapper<PossiblyCurrent, glutin::window::Window>;

pub enum Layout {
    Azerty,
    Qwerty,
}

impl Layout {
    pub fn parse(name: &str) -> Layout {
        match name {
            "azerty" | "fr" => Layout::Azerty,
            "qwerty" | "us" | "uk" | "en" => Layout::Qwerty,
            _ => panic!("unknown layout"),
        }
    }

    pub fn forward(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::Z,
            Layout::Qwerty => KeyCode::W,
        }
    }

    pub fn backward(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::S,
            Layout::Qwerty => KeyCode::S,
        }
    }

    pub fn right(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::Q,
            Layout::Qwerty => KeyCode::A,
        }
    }

    pub fn left(&self) -> KeyCode {
        match self {
            Layout::Azerty => KeyCode::D,
            Layout::Qwerty => KeyCode::D,
        }
    }
}

pub struct MyChunkListener {
    pub loaded_chunks: Vec<(i32, i32)>,
    pub unloaded_chunks: Vec<(i32, i32)>,
}

impl MyChunkListener {
    pub fn new() -> MyChunkListener {
        MyChunkListener {
            loaded_chunks: Vec::new(),
            unloaded_chunks: Vec::new(),
        }
    }

    pub fn has_been_updated(&self) -> bool {
        self.loaded_chunks.len() + self.unloaded_chunks.len() > 0
    }

    pub fn clear(&mut self) {
        self.loaded_chunks.clear();
        self.unloaded_chunks.clear();
    }
}

impl ChunkListener for MyChunkListener {
    fn chunk_load(&mut self, x: i32, y: i32) {
        self.loaded_chunks.push((x, y));
    }

    fn chunk_unload(&mut self, x: i32, y: i32) {
        self.unloaded_chunks.push((x, y));
    }
}

fn get_window_dim(context: &CTX) -> (u32, u32) {
    let dim = context.window().inner_size();
    (dim.width, dim.height)
}

pub fn game(
    seed: isize,
    flat: bool,
    view_distance: usize,
    with_shadows: bool,
    resolution_coeff: f32,
    layout: Layout,
) -> Result<(), Box<dyn std::error::Error>> {
    // --- Configuration ---
    let fov_range = (std::f32::consts::PI / 16.)..(std::f32::consts::PI / 2.);

    // --- World SetUp --
    let mut listener = MyChunkListener::new();

    let world = create_main_world(seed, flat);
    let mut player = world.create_player(&mut listener, view_distance);

    // --- debug tools SetUp ---
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    let mut termidrawer = TermiDrawer::new(30, false);

    // --- Window Helper ---
    let mut input_handler = wininput::WinInput::default();

    // --- Build Window ---
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new().with_title("Audran is stupid");

    let context = ContextBuilder::new()
        //        .with_vsync(true)
        .with_double_buffer(Some(true))
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 3)))
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
    unsafe { gl::Enable(gl::FRAMEBUFFER_SRGB) };

    context.window().set_cursor_visible(false);
    context.window().set_cursor_grab(true)?;

    let (width, height) = get_window_dim(&context);

    let mut camera = cubetracer::Camera::new(
        width as f32,
        height as f32,
        Vector3::new(0., 80., 0.),
        Vector2::new(std::f32::consts::PI / 2.0, 0.0),
        fov_range.start + (fov_range.end - fov_range.start) / 2.,
        16. / 9.,
    );

    // --- Cube Tracer ---
    let mut cubetracer = cubetracer::CubeTracer::new(
        width,
        height,
        view_distance,
        resolution_coeff,
        with_shadows,
        false,
    )
    .unwrap();

    // --- Main loop ---
    let mut frame_counter = FrameCounter::new(60);
    let mut fps_mean = 0.0;
    let mut fps_nb_ticks = 0.0;

    let mut __debug_min_coords: Vector2<i32> = Vector2::zeros();

    let mut total_time = 0.0;
    let mut update_rendering = false;

    event_loop.run(
        move |event, _, control_flow: &mut glutin::event_loop::ControlFlow| {
            *control_flow = glutin::event_loop::ControlFlow::Poll;
            let delta_time = frame_counter.delta_time();

            match event {
                glutin::event::Event::LoopDestroyed => return,
                glutin::event::Event::MainEventsCleared => {
                    input_handler.update_time(delta_time);
                    total_time += delta_time;

                    // --- Process inputs ---
                    if input_handler.updated(wininput::StateChange::MouseScroll) {
                        let fov = fov_range.start
                            + input_handler.get_scroll() * (fov_range.end - fov_range.start);
                        update_rendering = true;
                        camera.set_fov(fov)
                    }

                    if input_handler.updated(wininput::StateChange::MouseMotion) {
                        let offset = input_handler.get_mouse_offset() * delta_time;
                        update_rendering = true;
                        camera.reorient(offset);
                    }

                    let mut inputs = vec![];

                    if input_handler.is_button_pressed(MouseButton::Left) {
                        inputs.push(PlayerInput::LeftInteract);
                    }

                    if input_handler.is_button_pressed(MouseButton::Right) {
                        inputs.push(PlayerInput::RightInteract);
                    }

                    if input_handler.is_pressed(KeyCode::LShift) {
                        inputs.push(PlayerInput::Sneaking);
                    }
                    if input_handler.is_pressed(layout.forward()) {
                        inputs.push(PlayerInput::MoveFoward);
                    }
                    if input_handler.is_pressed(layout.right()) {
                        inputs.push(PlayerInput::MoveRight);
                    }
                    if input_handler.is_pressed(layout.backward()) {
                        inputs.push(PlayerInput::MoveBackward);
                    }
                    if input_handler.is_pressed(layout.left()) {
                        inputs.push(PlayerInput::MoveLeft);
                    }
                    if input_handler.is_pressed(KeyCode::Space) {
                        inputs.push(PlayerInput::Jump);
                    }
                    if input_handler.is_pressed(KeyCode::LControl) {
                        inputs.push(PlayerInput::SprintToggle);
                    }
                    if input_handler.is_double_pressed(KeyCode::Space) {
                        inputs.push(PlayerInput::FlyToggle);
                    }

                    camera.origin.y = camera.origin.y.clamp(0.0, 255.9);

                    // --- Update States ---

                    update_rendering = player.update(
                        world,
                        &mut listener,
                        camera.forward(),
                        camera.left(),
                        inputs,
                        delta_time,
                    ) || update_rendering;

                    camera.origin = player.head_position();

                    if input_handler.is_pressed(KeyCode::K) {
                        camera.update_sun_pos();
                    }
                    //player.set_position(world, &mut listener, camera.origin);

                    termidrawer.update_var(
                        "screen_top_left".to_string(),
                        format!("{:?}", camera.get_virtual_screen_top_left().data),
                    );
                    termidrawer.update_var(
                        "player_position".to_string(),
                        format!("{:?}", camera.origin.data),
                    );
                    termidrawer.update_var(
                        "v_forward".to_string(),
                        format!("{:?}", camera.forward().data),
                    );

                    termidrawer
                        .update_var("v_left".to_string(), format!("{:?}", camera.left().data));
                    termidrawer.update_var("v_up".to_string(), format!("{:?}", camera.up().data));

                    let __debug_curr_chunk = Vector2::new(
                        (camera.origin.x / 16.0).floor() as i32,
                        (camera.origin.z / 16.0).floor() as i32,
                    );

                    termidrawer.update_var(
                        "__debug_curr_chunk_world".to_string(),
                        format!("{:?}", __debug_curr_chunk.data),
                    );

                    if let Some(chunk) = world.chunk(__debug_curr_chunk.x, __debug_curr_chunk.y) {
                        termidrawer.update_var(
                            "__debug_chunk_empty".to_string(),
                            format!(
                                "{:?}",
                                chunk.chunk_filled_metadata()
                                    [(camera.origin.y / 16.0).floor() as usize]
                            ),
                        );
                    }

                    if listener.has_been_updated() {
                        let chunks_to_add: Vec<Rc<Chunk>> = listener
                            .loaded_chunks
                            .iter()
                            .map(|c| world.chunk(c.0, c.1).unwrap().clone())
                            .collect();

                        let chunks_to_rm: Vec<(i32, i32)> = listener.unloaded_chunks.clone();

                        __debug_min_coords = cubetracer
                            .args
                            .update_chunks(chunks_to_rm, chunks_to_add)
                            .unwrap();

                        termidrawer.update_var(
                            "__debug_min_coords".to_string(),
                            format!("{:?}", __debug_min_coords.data),
                        );
                        termidrawer.update_var(
                            "nb_chunks_listener".to_string(),
                            format!("{:?}", cubetracer.args.nb_mapped_chunks()),
                        );

                        termidrawer.log(format!("> chunks loaded  : {:?}", listener.loaded_chunks));
                        termidrawer
                            .log(format!("> chunks unloaded: {:?}", listener.unloaded_chunks));

                        listener.clear();
                    }
                    termidrawer.update_var(
                        "__debug_curr_chunk_local".to_string(),
                        format!("{:?}", (__debug_curr_chunk - __debug_min_coords).data),
                    );

                    // - Cube Tracer -

                    let highlighted_block = match player.looked_block(&world, camera.forward()) {
                        Some((b, _)) => b,
                        _ => Vector3::new(0, -100, 0),
                    };

                    //FIXME improve wind
                    let wind =
                        Vector3::new((total_time + 0.8).cos() / 4., 1.0, total_time.sin() / 4.)
                            .normalize();

                    cubetracer
                        .args
                        .set_camera(
                            total_time,
                            update_rendering,
                            &camera,
                            wind,
                            highlighted_block,
                        )
                        .unwrap();

                    context.window().request_redraw();
                    update_rendering = false;
                }
                event::Event::RedrawRequested(_) => {
                    let (width, height) = get_window_dim(&context);

                    cubetracer.compute_image(width, height).unwrap();
                    cubetracer.draw().unwrap();

                    context.swap_buffers().unwrap();

                    if let Some(fps) = frame_counter.tick() {
                        termidrawer.update_var("fps".to_string(), format!("{}", fps));
                        termidrawer.update_fps(fps as f64);
                        fps_mean = fps_mean * fps_nb_ticks + fps;
                        fps_nb_ticks += 1.0;
                        fps_mean /= fps_nb_ticks;
                        termidrawer.update_var("fps_mean".to_string(), format!("{}", fps_mean));
                    }

                    termidrawer.draw(&mut terminal).unwrap();
                }
                event::Event::DeviceEvent { event, .. } => input_handler.on_device_event(event),
                event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput { input, .. } => {
                        input_handler.on_keyboard_input(input);
                        if input_handler.is_pressed_once(KeyCode::P) {
                            update_rendering = true;
                            cubetracer.toggle_global_illum().unwrap();
                        }
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        input_handler.on_mouse_input(button, state)
                    }
                    glutin::event::WindowEvent::Resized(physical_size) => {
                        context.resize(physical_size);

                        camera.set_image_size(
                            physical_size.width as f32 / resolution_coeff,
                            physical_size.height as f32 / resolution_coeff,
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
