use std::time::Instant;

pub struct FrameCounter {
    fps_nb_sample: usize,
    fps_sampling_curr_frame: usize,
    delta_time: f32,
    instant_last_frame: Instant,
    instant_last_fps_sampling_frame: Instant,
}

impl FrameCounter {
    pub fn new(fps_nb_sample: usize) -> Self {
        assert!(
            fps_nb_sample > 0,
            "the number of frame samples must be superior to 0."
        );

        FrameCounter {
            fps_nb_sample,
            fps_sampling_curr_frame: 0,
            delta_time: 0.0,
            instant_last_frame: Instant::now(),
            instant_last_fps_sampling_frame: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        *self = FrameCounter::new(self.fps_nb_sample);
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn tick(&mut self) -> Option<f32> {
        let now = Instant::now();

        self.delta_time = self.instant_last_frame.elapsed().as_secs_f32();
        self.instant_last_frame = now;

        self.fps_sampling_curr_frame += 1;
        if self.fps_sampling_curr_frame == self.fps_nb_sample {
            let fps = self.fps_nb_sample as f32
                / self.instant_last_fps_sampling_frame.elapsed().as_secs_f32();

            self.fps_sampling_curr_frame = 0;
            self.instant_last_fps_sampling_frame = now;

            Some(fps)
        } else {
            None
        }
    }
}
