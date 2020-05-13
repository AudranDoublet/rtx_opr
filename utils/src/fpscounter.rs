use std::time::Instant;

pub struct FPSCounter {
    nb_sample: usize,
    curr_frame: usize,
    last_clock: Instant,
}

impl FPSCounter {
    pub fn new(nb_sample: usize) -> Self {
        assert!(
            nb_sample > 0,
            "the number of frame samples must be superior to 0."
        );

        FPSCounter {
            nb_sample,
            curr_frame: 0,
            last_clock: Instant::now(),
        }
    }

    pub fn tick(&mut self) -> Option<f32> {
        self.curr_frame += 1;

        if self.curr_frame == self.nb_sample {
            let fps = self.nb_sample as f32 / self.last_clock.elapsed().as_secs_f32();

            self.curr_frame = 0;
            self.last_clock = Instant::now();

            Some(fps)
        } else {
            None
        }
    }
}
