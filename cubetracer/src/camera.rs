use crate::datatypes::{UniformCamera, UniformSun};

use nalgebra::{Vector2, Vector3, Vector4, Matrix4};

fn vec3to4(v: Vector3<f32>, last: f32) -> Vector4<f64> {
    Vector4::new(v.x as f64, v.y as f64, v.z as f64, last as f64)
}

impl Camera {
    pub fn uniform(&self) -> UniformCamera {
        UniformCamera {
            origin: vec3to4(self.origin, 0.0),
            screen_to_world: self.world_to_screen().try_inverse().unwrap(),
            prev_world_to_screen: self.prev_world_to_screen,
            updated: self.updated,
        }
    }
}


pub struct Sun {
    sun_direction: Vector3<f32>,
    light_cycle: f32,
    view_distance: f32,
}

impl Sun {
    pub fn sun_light_cycle(&mut self, dt: f32) {
        self.light_cycle = (self.light_cycle + dt / 4.) % (std::f32::consts::PI * 1.2);

        let x = self.light_cycle.cos();
        let y = -self.light_cycle.sin();

        self.sun_direction = Vector3::new(x, y, x * y).normalize();
    }

    pub fn update_sun_pos(&mut self, forward: Vector3<f32>) {
        self.sun_direction = -forward;
    }

    pub fn sun_direction(&self) -> Vector3<f32> {
        self.sun_direction
    }

    fn projection_matrix(&self) -> Matrix4<f32> {
        *nalgebra::Orthographic3::new(
            -self.view_distance * 16.0,
            self.view_distance * 16.0,
            0.0,
            256.0,
            0.0,
            256.0
        ).as_matrix()
    }

    pub fn uniform(&self) -> UniformSun {
        let direction = self.sun_direction();
        let direction: Vector4<f32> = Vector4::new(
            direction.x,
            direction.y,
            direction.z,
            0.0
        );

        UniformSun {
            projection: self.projection_matrix(),
            direction,
        }
    }
}

impl Sun {
    pub fn new(view_distance: usize, direction: Vector3<f32>) -> Self {
        Sun {
            view_distance: view_distance as f32,
            light_cycle: 0.0,
            sun_direction: direction.normalize(),
        }
    }
}

pub struct Camera {
    origin: Vector3<f32>,

    updated: bool,

    up: Vector3<f32>,
    forward: Vector3<f32>,
    left: Vector3<f32>,

    rotation: Vector2<f32>,

    prev_world_to_screen: Matrix4<f64>,

    fov: f32,
    aspect_ratio: f32,
}

impl Camera {
    pub fn store_previous_view(&mut self) {
        self.updated = false;
        self.prev_world_to_screen = self.world_to_screen();
    }

    pub fn world_to_screen(&self) -> Matrix4<f64> {
        self.projection_matrix() * self.view_matrix().try_inverse().unwrap()
    }

    pub fn view_matrix(&self) -> Matrix4<f64> {
        Matrix4::from_columns(&[
            vec3to4(-self.left.normalize(), 0.0),
            vec3to4(self.up.normalize(), 0.0),
            vec3to4(-self.forward.normalize(), 0.0),
            vec3to4(self.origin, 1.0),
        ])
    }

    pub fn projection_matrix(&self) -> Matrix4<f64> {
        let r = self.aspect_ratio as f64;
        let t = 1.0 / (self.fov as f64 / 2.0).tan();

        let far = 5000.0;
        let near = 1e-10;

        Matrix4::new(
            t / r, 0.0 ,  0.0                            ,  0.0,
            0.0  ,  t  ,  0.0                            ,  0.0,
            0.0  ,  0.0, (far + near) / (near - far)     , -1.0,
            0.0  ,  0.0, (2. * far * near) / (near - far),  0.0,
        ).transpose()
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
        self.rotation += Vector2::new(x, -y);
        self.rotation.y = self
            .rotation
            .y
            .max(-std::f32::consts::PI / 2.1)
            .min(std::f32::consts::PI / 2.1);

        self.update_axes();
    }

    fn update_axes(&mut self) {
        self.updated = true;

        let cos_rot_x = self.rotation.x.cos();
        let cos_rot_y = self.rotation.y.cos();
        let sin_rot_x = self.rotation.x.sin();
        let sin_rot_y = self.rotation.y.sin();

        self.forward =
            Vector3::new(cos_rot_x * cos_rot_y, sin_rot_y, sin_rot_x * cos_rot_y).normalize();
        self.left = self.forward.cross(&Vector3::y()).normalize();
        self.up = self.left.cross(&self.forward).normalize();
    }

    fn assert_fov_valid(fov: f32) {
        assert!(fov > 0., "The FOV cannot be null or negative");
        assert!(
            fov < std::f32::consts::PI,
            "The FOV cannot be superior to PI"
        );
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.updated = true;
        Self::assert_fov_valid(fov);
        self.fov = fov;
}

    pub fn set_origin(&mut self, origin: Vector3<f32>) {
        if self.origin != origin {
            self.updated = true;
        }
        self.origin = origin;
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

            updated: true,

            aspect_ratio,
            fov,

            prev_world_to_screen: Matrix4::identity(),
        };

        camera.update_axes();

        camera
    }
}
