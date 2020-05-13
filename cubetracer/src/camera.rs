use nalgebra::{Vector2, Vector3};

pub struct Camera {
    image_size: Vector2<f32>,

    virtual_screen_size: Vector2<f32>,

    pub origin: Vector3<f32>,

    up: Vector3<f32>,
    forward: Vector3<f32>,
    left: Vector3<f32>,

    aspect_ratio: f32,
}

fn compute_virtual_screen_size(fov: f32, aspect_ratio: f32) -> Vector2<f32> {
    let h = 2.0 * (fov / 2.0).tan();
    let w = h * aspect_ratio;

    Vector2::new(w, h)
}

impl Camera {
    pub fn get_virtual_screen_axes_scaled(&self) -> (Vector3<f32>, Vector3<f32>) {
        let scales = self.virtual_screen_size.component_div(&self.image_size);

        (self.left * scales.x, self.up * scales.y)
    }

    pub fn forward(&self) -> Vector3<f32> {
        self.forward
    }

    pub fn up(&self) -> Vector3<f32> {
        self.up
    }

    pub fn left(&self) -> Vector3<f32> {
        self.left
    }

    pub fn get_virtual_screen_top_left(&self) -> Vector3<f32> {
        let screen_center = self.origin + self.forward;

        screen_center
            + 0.5 * self.left * self.virtual_screen_size.x
            + 0.5 * self.up * self.virtual_screen_size.y
    }

    pub fn set_image_size(&mut self, width: f32, height: f32) {
        self.image_size = Vector2::new(width, height);
    }

    fn assert_fov_valid(fov: f32) {
        assert!(fov > 0., "The FOV cannot be null or negative");
        assert!(
            fov < std::f32::consts::PI,
            "The FOV cannot be superior to PI"
        );
    }

    pub fn set_fov(&mut self, fov: f32) {
        Self::assert_fov_valid(fov);
        self.virtual_screen_size = compute_virtual_screen_size(fov, self.aspect_ratio);
    }

    pub fn new(
        width: f32,
        height: f32,
        origin: Vector3<f32>,
        forward: Vector3<f32>,
        up: Vector3<f32>,
        fov: f32,
        aspect_ratio: f32,
    ) -> Camera {
        let up = up.normalize();
        let forward = forward.normalize();

        Camera {
            image_size: Vector2::new(width, height),
            origin,
            up,
            forward,
            left: forward.cross(&up),
            aspect_ratio,
            virtual_screen_size: compute_virtual_screen_size(fov, aspect_ratio),
        }
    }
}
