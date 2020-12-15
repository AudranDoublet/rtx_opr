use crate::datatypes::UniformCamera;

use nalgebra::{Vector2, Vector3, Vector4};

fn vec3to4(v: Vector3<f32>) -> Vector4<f32> {
    Vector4::new(v.x, v.y, v.z, 0.0)
}

impl Camera {
    pub fn uniform(&self) -> UniformCamera {
        let origin = self.origin;
        let top_left = self.get_virtual_screen_top_left();
        let (left, up) = self.get_virtual_screen_axes_scaled();

        UniformCamera {
            top_left: vec3to4(top_left),
            left: vec3to4(left),
            up: vec3to4(up),
            origin: vec3to4(origin),
        }
    }
}

pub struct Camera {
    virtual_screen_size: Vector2<f32>,

    pub origin: Vector3<f32>,

    up: Vector3<f32>,
    forward: Vector3<f32>,
    left: Vector3<f32>,

    rotation: Vector2<f32>,

    aspect_ratio: f32,
    sun_direction: Vector3<f32>,

    light_cycle: f32,
}

fn compute_virtual_screen_size(fov: f32, aspect_ratio: f32) -> Vector2<f32> {
    let h = 2.0 * (fov / 2.0).tan();
    let w = h * aspect_ratio;

    Vector2::new(w, h)
}

impl Camera {
    pub fn get_virtual_screen_axes_scaled(&self) -> (Vector3<f32>, Vector3<f32>) {
        let scales = self.virtual_screen_size;

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

    pub fn reorient(&mut self, x: f32, y: f32) {
        self.rotation += Vector2::new(x, y);
        self.rotation.y = self
            .rotation
            .y
            .max(-std::f32::consts::PI / 2.).min(std::f32::consts::PI / 2.);

        self.update_axes();
    }

    pub fn sun_light_cycle(&mut self, dt: f32) {
        self.light_cycle = (self.light_cycle + dt / 4.) % (std::f32::consts::PI * 1.2);

        let x = self.light_cycle.cos();
        let y = -self.light_cycle.sin();

        self.sun_direction = Vector3::new(x, y, x * y).normalize();
    }

    pub fn update_sun_pos(&mut self) {
        self.sun_direction = -self.forward();
    }

    pub fn sun_direction(&self) -> Vector3<f32> {
        self.sun_direction
    }

    fn update_axes(&mut self) {
        let cos_rot_x = self.rotation.x.cos();
        let cos_rot_y = self.rotation.y.cos();
        let sin_rot_x = self.rotation.x.sin();
        let sin_rot_y = self.rotation.y.sin();

        self.forward =
            Vector3::new(cos_rot_x * cos_rot_y, sin_rot_y, sin_rot_x * cos_rot_y).normalize();
        self.left = -self.forward.cross(&Vector3::y()).normalize();
        self.up = self.left.cross(&self.forward).normalize();
    }

    pub fn get_virtual_screen_top_left(&self) -> Vector3<f32> {
        self.forward
            + 0.5 * self.left * self.virtual_screen_size.x
            + 0.5 * self.up * self.virtual_screen_size.y
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

    pub fn set_origin(&mut self, x: f32, y: f32, z: f32) {
        self.origin = Vector3::new(x, y, z)
    }

    pub fn new(
        origin: Vector3<f32>,
        rotation: Vector2<f32>,
        fov: f32,
        aspect_ratio: f32,
    ) -> Camera {
        let mut camera = Camera {
            origin,
            rotation,
            up: Vector3::new(0.0, 0.0, 0.0),
            forward: Vector3::new(0.0, 0.0, 0.0),
            left: Vector3::new(0.0, 0.0, 0.0),
            aspect_ratio,
            virtual_screen_size: compute_virtual_screen_size(fov, aspect_ratio),
            sun_direction: Vector3::new(-0.7, -1.5, -1.1),
            light_cycle: 0.0,
        };

        camera.update_axes();

        camera
    }
}
