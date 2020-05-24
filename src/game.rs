use crate::termidraw::TermiDrawer;
use gl;
use glutin::dpi;
use glutin::event;
use glutin::event::VirtualKeyCode as KeyCode;
use glutin::event::WindowEvent;
use glutin::{ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use nalgebra::{Vector2, Vector3};
use utils::framecounter::FrameCounter;
use utils::wininput;

use std::{collections::HashSet, rc::Rc};

use termion::{raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

use world::{create_main_world, Chunk, ChunkListener, Player, PlayerInput};
type CTX = ContextWrapper<PossiblyCurrent, glutin::window::Window>;

pub struct MyChunkListener {
    updated: bool,
    pub chunks: HashSet<(i32, i32)>,
}

impl MyChunkListener {
    pub fn new() -> MyChunkListener {
        MyChunkListener {
            updated: false,
            chunks: HashSet::new(),
        }
    }

    pub fn has_been_updated(&mut self) -> bool {
        if self.updated {
            self.updated = false;
            true
        } else {
            false
        }
    }
}

impl ChunkListener for MyChunkListener {
    fn chunk_load(&mut self, x: i32, y: i32) {
        self.chunks.insert((x, y));
        self.updated = true;
    }

    fn chunk_unload(&mut self, x: i32, y: i32) {
        self.chunks.remove(&(x, y));
        self.updated = true;
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

pub fn game(seed: isize, view_distance: usize) -> Result<(), Box<dyn std::error::Error>> {
    // --- Configuration ---
    let fov_range = (std::f32::consts::PI / 16.)..(std::f32::consts::PI / 2.);
    let mut movement_speed: f32 = 50.0;

    // --- World SetUp --
    let mut listener = MyChunkListener::new();

    let world = create_main_world(seed);
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
        .with_vsync(true)
        .with_double_buffer(Some(true))
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 3)))
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };
    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    context.window().set_cursor_visible(false);
    //set_cursor_middle_window(&context);

    let (width, height) = get_window_dim(&context);

    let mut camera = cubetracer::Camera::new(
        width as f32,
        height as f32,
        Vector3::new(0., 80., 0.),
        Vector2::new(std::f32::consts::PI / 2.0, 0.0),
        fov_range.start + (fov_range.end - fov_range.start) / 2.,
        16. / 9.,
    );

    // --- Cube Tracer -i-
    let mut cubetracer = cubetracer::CubeTracer::new(width, height, view_distance).unwrap();

    // --- Main loop ---
    let mut frame_counter = FrameCounter::new(60);

    let mut __debug_min_coords: Vector2<i32> = Vector2::zeros();

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
                    if input_handler.is_pressed(KeyCode::PageUp) {
                        movement_speed += 0.25;
                        movement_speed = movement_speed.min(100.0);
                    } else if input_handler.is_pressed(KeyCode::PageDown) {
                        movement_speed -= 0.25;
                        movement_speed = movement_speed.max(0.5);
                    }

                    let mut inputs = vec![];

                    let speed = movement_speed * delta_time;
                    if input_handler.is_pressed(KeyCode::W) {
                        inputs.push(PlayerInput::MoveFoward);
                    }
                    if input_handler.is_pressed(KeyCode::A) {
                        inputs.push(PlayerInput::MoveLeft);
                    }
                    if input_handler.is_pressed(KeyCode::S) {
                        inputs.push(PlayerInput::MoveBackward);
                    }
                    if input_handler.is_pressed(KeyCode::D) {
                        inputs.push(PlayerInput::MoveRight);
                    }
                    if input_handler.is_pressed(KeyCode::Space) {
                        inputs.push(PlayerInput::Jump);
                    }
                    if input_handler.is_pressed(KeyCode::LControl) {
                        inputs.push(PlayerInput::SprintToggle);
                    }
                    camera.origin.y = camera.origin.y.clamp(0.0, 255.9);
                    // FIXME-END

                    set_cursor_middle_window(&context);

                    // --- Update States ---

                    player.update(world, &mut listener, camera.forward(), camera.left(), inputs, delta_time);
                    camera.origin = player.head_position();
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
                    termidrawer.update_var("speed".to_string(), format!("{:?}", movement_speed));

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
                        let chunks: Vec<Rc<Chunk>> = listener
                            .chunks
                            .iter()
                            .map(|c| world.chunk(c.0, c.1).unwrap().clone())
                            .collect();

                        __debug_min_coords = cubetracer.args.set_chunks(chunks).unwrap();

                        termidrawer.update_var(
                            "__debug_min_coords".to_string(),
                            format!("{:?}", __debug_min_coords.data),
                        );
                        termidrawer.update_var(
                            "nb_chunks_listener".to_string(),
                            format!("{:?}", listener.chunks.len()),
                        );
                        termidrawer.log(format!("> chunks : {:?}", listener.chunks));
                    }
                    termidrawer.update_var(
                        "__debug_curr_chunk_local".to_string(),
                        format!("{:?}", (__debug_curr_chunk - __debug_min_coords).data),
                    );

                    /*
                    player.update(
                        world,
                        &listener,
                        camera.forward(),
                        -camera.left(),
                        Vec::new(),
                        delta_time,
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
                        termidrawer.update_var("fps".to_string(), format!("{}", fps));
                        termidrawer.update_fps(fps as f64);
                    }

                    termidrawer.draw(&mut terminal).unwrap();
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
