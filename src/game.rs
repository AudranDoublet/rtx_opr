use winit::event::WindowEvent;
use winit::event::{MouseButton, VirtualKeyCode as KeyCode};
use winit::event_loop::EventLoop;

use utils::framecounter::FrameCounter;
use utils::wininput;

use world::{create_main_world, main_world, ChunkListener, PlayerInput};

use crate::config::*;

use std::sync::Arc;

use cubetracer::context::Context;
use cubetracer::window::*;

use ash::{version::DeviceV1_0, vk, Device};

use rayon::prelude::*;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

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

const FOV_RANGE: std::ops::Range<f32> = (std::f32::consts::PI / 16.)..(std::f32::consts::PI / 2.);

pub struct BaseApp {
    config: Config,

    window: winit::window::Window,

    context: Arc<Context>,
    swapchain_properties: SwapchainProperties,
    depth_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
    render_pass: RenderPass,
    swapchain: Swapchain,

    input_handler: wininput::WinInput,

    in_flight_frames: InFlightFrames,

    layout: Layout,

    mouse_is_focused: bool,
    tracer: cubetracer::Cubetracer,

    player: world::Player,
}

impl BaseApp {
    pub fn run(
        world_path: &str,
        seed: isize,
        flat: bool,
        view_distance: usize,
        config: Config, layout: Layout) {

        // --- World SetUp --
        let mut listener = MyChunkListener::new();

        let world = create_main_world(world_path, seed, flat);
        let player = world.create_player(&mut listener, view_distance);

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("RTX")
            .build(&event_loop)
            .unwrap();
        window.set_cursor_visible(false);

        let context = Arc::new(Context::new(&window));

        let swapchain_support_details = SwapchainSupportDetails::new(
            context.physical_device(),
            context.surface(),
            context.surface_khr(),
        );

        let swapchain_properties = swapchain_support_details
            .get_ideal_swapchain_properties(config.resolution, config.vsync);
        let depth_format = Self::find_depth_format(&context);
        let msaa_samples = context.get_max_usable_sample_count(config.msaa);

        let render_pass = RenderPass::create(
            Arc::clone(&context),
            swapchain_properties.extent,
            swapchain_properties.format.format,
            depth_format,
            msaa_samples,
        );

        let swapchain = Swapchain::create(
            Arc::clone(&context),
            swapchain_support_details,
            config.resolution,
            config.vsync,
            &render_pass,
        );

        let in_flight_frames = Self::create_sync_objects(context.device());

        let tracer = cubetracer::Cubetracer::new(
            &context,
            &swapchain,
            16. / 9.,
            FOV_RANGE.start + (FOV_RANGE.end - FOV_RANGE.start) / 2.,
        );

        let game = Self {
            config,
            window,

            context,
            swapchain_properties,
            render_pass,
            swapchain,
            depth_format,
            msaa_samples,
            in_flight_frames,
            input_handler: wininput::WinInput::default(),
            layout,
            mouse_is_focused: false,

            tracer,

            player,
        };
        game.process_event(event_loop);
    }

    fn find_depth_format(context: &Context) -> vk::Format {
        let candidates = vec![
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];
        context
            .find_supported_format(
                &candidates,
                vk::ImageTiling::OPTIMAL,
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
            )
            .expect("Failed to find a supported depth format")
    }

    fn create_sync_objects(device: &Device) -> InFlightFrames {
        let mut sync_objects_vec = Vec::new();
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = {
                let semaphore_info = vk::SemaphoreCreateInfo::builder();
                unsafe { device.create_semaphore(&semaphore_info, None).unwrap() }
            };

            let render_finished_semaphore = {
                let semaphore_info = vk::SemaphoreCreateInfo::builder();
                unsafe { device.create_semaphore(&semaphore_info, None).unwrap() }
            };

            let in_flight_fence = {
                let fence_info =
                    vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
                unsafe { device.create_fence(&fence_info, None).unwrap() }
            };

            let sync_objects = SyncObjects {
                image_available_semaphore,
                render_finished_semaphore,
                fence: in_flight_fence,
            };
            sync_objects_vec.push(sync_objects)
        }

        InFlightFrames::new(sync_objects_vec)
    }

    pub fn process_event(mut self, event_loop: EventLoop<()>) {
        let mut total_time = 0.0;

        let mut frame_counter = FrameCounter::new(60);
        let mut listener = MyChunkListener::new();

        event_loop.run(
            move |event, _, control_flow: &mut winit::event_loop::ControlFlow| {
                *control_flow = winit::event_loop::ControlFlow::Poll;

                let delta_time = frame_counter.delta_time();

                match event {
                    winit::event::Event::LoopDestroyed => return,
                    winit::event::Event::MainEventsCleared => {
                        self.input_handler.update_time(delta_time);
                        total_time += delta_time;

                        // --- Process inputs ---
                        if self.input_handler.updated(wininput::StateChange::MouseScroll) {
                            let fov = FOV_RANGE.start
                                + self.input_handler.get_scroll() * (FOV_RANGE.end - FOV_RANGE.start);
                            self.tracer.camera().set_fov(fov)
                        }

                        if self.input_handler.updated(wininput::StateChange::MouseMotion) {
                            let offset = self.input_handler.get_mouse_offset() * delta_time;
                            self.tracer.camera().reorient(offset.x, offset.y);
                        }

                        let mut inputs = vec![];

                        if self.input_handler.is_button_pressed(MouseButton::Left) {
                            inputs.push(PlayerInput::LeftInteract);
                        }

                        if self.input_handler.is_button_pressed(MouseButton::Right) {
                            inputs.push(PlayerInput::RightInteract);
                        }

                        if self.input_handler.is_pressed(KeyCode::LShift) {
                            inputs.push(PlayerInput::Sneaking);
                        }
                        if self.input_handler.is_pressed(self.layout.forward()) {
                            inputs.push(PlayerInput::MoveFoward);
                        }
                        if self.input_handler.is_pressed(self.layout.right()) {
                            inputs.push(PlayerInput::MoveRight);
                        }
                        if self.input_handler.is_pressed(self.layout.backward()) {
                            inputs.push(PlayerInput::MoveBackward);
                        }
                        if self.input_handler.is_pressed(self.layout.left()) {
                            inputs.push(PlayerInput::MoveLeft);
                        }
                        if self.input_handler.is_pressed(KeyCode::Space) {
                            inputs.push(PlayerInput::Jump);
                        }
                        if self.input_handler.is_pressed(KeyCode::LControl) {
                            inputs.push(PlayerInput::SprintToggle);
                        }
                        if self.input_handler.is_double_pressed(KeyCode::Space) {
                            inputs.push(PlayerInput::FlyToggle);
                        }
                        if self.input_handler.is_pressed(KeyCode::K) {
                            self.tracer.camera().update_sun_pos();
                        }
                        if self.input_handler.is_pressed(KeyCode::N) {
                            self.tracer.camera().sun_light_cycle(delta_time);
                        }

                        // --- Update States ---
                        self.player.update(
                            main_world(),
                            &mut listener,
                            self.tracer.camera().forward(),
                            self.tracer.camera().left(),
                            inputs,
                            delta_time,
                        );

                        self.tracer.camera().origin = self.player.head_position();

                        if listener.has_been_updated() {
                            let chunks = listener
                                .loaded_chunks
                                .par_iter()
                                .map(|c| (c.0, c.1, main_world().chunk(c.0, c.1).unwrap().mesh(main_world())))
                                .collect::<Vec<_>>();

                            for (x, z, chunk) in chunks {
                                self.tracer.register_or_update_chunk(
                                    &self.context, x, z, chunk
                                );
                            }

                            listener
                                .unloaded_chunks
                                .iter()
                                .for_each(|(x, y)| self.tracer.delete_chunk(*x, *y));

                            listener.clear();
                        }

                        // - Cube Tracer -

                        /* FIXME send it to RTX
                        let highlighted_block = match self.player.looked_block(&main_world(), self.tracer.camera().forward()) {
                            Some((b, _)) => b,
                            _ => Vector3::new(0, -100, 0),
                        };
                        */

                        //FIXME improve wind (and use it with RTX)
                        /*let wind =
                            Vector3::new((total_time + 0.8).cos() / 4., 1.0, total_time.sin() / 4.)
                                .normalize();*/

                        //FIXME send wind & highlighted_block & total time
                        self.window.request_redraw();
                    }
                    winit::event::Event::RedrawRequested(_) => {
                        self.tracer.draw_frame(&self.context);
                        if let Some(_) = self.draw_frame() {
                            self.tracer.resize(&self.context, &self.swapchain);
                        }
                        frame_counter.tick();
                    }
                    winit::event::Event::DeviceEvent { event, .. } => self.input_handler.on_device_event(event),
                    winit::event::Event::WindowEvent { event, .. } => match event {
                        WindowEvent::KeyboardInput { input, .. } => {
                            self.input_handler.on_keyboard_input(input);
                            if self.input_handler.is_pressed_once(KeyCode::P) {
                                //cubetracer.toggle_global_illum().unwrap();
                            }
                            if self.input_handler.is_pressed_once(KeyCode::L) {
                                //cubetracer.toggle_ambient_light().unwrap();
                            }
                            if self.input_handler.is_pressed_once(KeyCode::M) {
                                //cubetracer.toggle_sky_atm().unwrap();
                            }

                            if self.input_handler.is_pressed_once(KeyCode::Escape) {
                                self.mouse_is_focused = false;
                                self.window.set_cursor_grab(false).unwrap();
                            }
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            if self.mouse_is_focused {
                                self.input_handler.on_mouse_input(button, state)
                            }
                        }
                        WindowEvent::Focused(is_focused) => {
                            match self.window.set_cursor_grab(is_focused) {
                                Ok(_) => self.mouse_is_focused = is_focused,
                                _ => (),
                            }
                        }
                        winit::event::WindowEvent::CloseRequested => {
                            *control_flow = winit::event_loop::ControlFlow::Exit
                        }
                        _ => (),
                    },
                    _ => (),
                };
            },
        );
    }

    pub fn draw_frame(
        &mut self,
    ) -> Option<SwapchainProperties> {
        let command_buffers = self.tracer.commands();

        log::trace!("Drawing frame.");
        let sync_objects = self.in_flight_frames.next().unwrap();
        let image_available_semaphore = sync_objects.image_available_semaphore;
        let render_finished_semaphore = sync_objects.render_finished_semaphore;
        let in_flight_fence = sync_objects.fence;
        let wait_fences = [in_flight_fence];

        unsafe {
            self.context
                .device()
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .unwrap()
        };

        let result = self
            .swapchain
            .acquire_next_image(None, Some(image_available_semaphore), None);
        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain();
                return Some(self.swapchain_properties);
            }
            Err(error) => panic!("Error while acquiring next image. Cause: {}", error),
        };

        unsafe { self.context.device().reset_fences(&wait_fences).unwrap() };

        let device = self.context.device();
        let wait_semaphores = [image_available_semaphore];
        let signal_semaphores = [render_finished_semaphore];

        // Submit command buffer
        {
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = [command_buffers[image_index as usize]];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores)
                .build();
            let submit_infos = [submit_info];
            unsafe {
                device
                    .queue_submit(
                        self.context.graphics_queue(),
                        &submit_infos,
                        in_flight_fence,
                    )
                    .unwrap()
            };
        }

        let swapchains = [self.swapchain.swapchain_khr()];
        let images_indices = [image_index];

        {
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&images_indices);
            let result = self.swapchain.present(&present_info);
            match result {
                Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.recreate_swapchain();
                }
                Err(error) => panic!("Failed to present queue. Cause: {}", error),
                _ => {}
            }

            None
        }
    }

    /// Recreates the swapchain.
    ///
    /// If the window has been resized, then the new size is used
    /// otherwise, the size of the current swapchain is used.
    ///
    /// If the window has been minimized, then the functions block until
    /// the window is maximized. This is because a width or height of 0
    /// is not legal.
    fn recreate_swapchain(&mut self) {
        log::debug!("Recreating swapchain.");

        unsafe { self.context.device().device_wait_idle().unwrap() };

        self.cleanup_swapchain();

        let dimensions = [
            self.swapchain.properties().extent.width,
            self.swapchain.properties().extent.height,
        ];

        let swapchain_support_details = SwapchainSupportDetails::new(
            self.context.physical_device(),
            self.context.surface(),
            self.context.surface_khr(),
        );
        let swapchain_properties =
            swapchain_support_details.get_ideal_swapchain_properties(dimensions, self.config.vsync);

        let render_pass = RenderPass::create(
            Arc::clone(&self.context),
            swapchain_properties.extent,
            swapchain_properties.format.format,
            self.depth_format,
            self.msaa_samples,
        );

        let swapchain = Swapchain::create(
            Arc::clone(&self.context),
            swapchain_support_details,
            dimensions,
            self.config.vsync,
            &render_pass,
        );

        self.swapchain = swapchain;
        self.swapchain_properties = swapchain_properties;
        self.render_pass = render_pass;

        self.tracer.resize(&self.context, &self.swapchain);
    }

    /// Clean up the swapchain and all resources that depends on it.
    fn cleanup_swapchain(&mut self) {
        self.swapchain.destroy();
    }
}

impl Drop for BaseApp {
    fn drop(&mut self) {
        log::debug!("Dropping application.");
        self.cleanup_swapchain();
        let device = self.context.device();
        self.in_flight_frames.destroy(device);
    }
}

#[derive(Clone, Copy)]
struct SyncObjects {
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    fence: vk::Fence,
}

impl SyncObjects {
    fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.render_finished_semaphore, None);
            device.destroy_fence(self.fence, None);
        }
    }
}

struct InFlightFrames {
    sync_objects: Vec<SyncObjects>,
    current_frame: usize,
}

impl InFlightFrames {
    fn new(sync_objects: Vec<SyncObjects>) -> Self {
        Self {
            sync_objects,
            current_frame: 0,
        }
    }

    fn destroy(&self, device: &Device) {
        self.sync_objects.iter().for_each(|o| o.destroy(&device));
    }
}

impl Iterator for InFlightFrames {
    type Item = SyncObjects;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.sync_objects[self.current_frame];

        self.current_frame = (self.current_frame + 1) % self.sync_objects.len();

        Some(next)
    }
}
