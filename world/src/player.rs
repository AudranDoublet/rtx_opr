use crate::{ivec_to_f, worldf_to_chunk, World, AABB};
use nalgebra::{Vector2, Vector3};
use std::collections::HashSet;

const GRAVITY: f32 = 9.81;
const WATER_GRAVITY: f32 = 1.5;
const JUMP_FORCE: f32 = 9.81;
const PLAYER_SIZE: f32 = 0.5;
const PLAYER_HEIGHT: f32 = 1.8;

const SPRINT_SPEED_MULTIPLIER: f32 = 1.5;
const SPEED: f32 = 1.0;
const WATER_SPEED: f32 = 1.0;
const WATER_Y_SPEED: f32 = 1.0;

pub enum PlayerInput {
    Jump,
    SprintToggle,
    MoveFoward,
    MoveBackward,
    MoveRight,
    MoveLeft,
}

pub trait ChunkListener {
    /**
     * Called when a chunk is loaded or modified
     */
    fn chunk_load(&self, x: i64, y: i64);

    /**
     * Called when a chunk is unloaded
     */
    fn chunk_unload(&self, x: i64, y: i64);
}

pub struct Player {
    view_distance: i64,
    position: Vector3<f32>,
    sprinting: bool,
    grounded: bool,

    velocity: Vector3<f32>,

    /** Chunk provider */
    last_chunk_update: Vector3<f32>,
    known_chunks: HashSet<Vector2<i64>>,
}

impl Player {
    pub fn new(view_distance: usize) -> Player {
        Player {
            view_distance: view_distance as i64,
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),

            sprinting: false,
            grounded: false,

            /* Chunk provider */
            last_chunk_update: Vector3::new(std::f32::INFINITY, 0.0, 0.0),
            known_chunks: HashSet::new(),
        }
    }

    pub fn collider(&self) -> AABB {
        AABB::new(self.position, self.position)
            .augment3(PLAYER_SIZE / 2., PLAYER_HEIGHT, PLAYER_SIZE / 2.)
            .augment3(-PLAYER_SIZE / 2., 0.0, -PLAYER_SIZE / 2.)
    }

    fn move_player(
        &mut self,
        world: &mut World,
        listener: &dyn ChunkListener,
        movement: Vector3<f32>,
        dt: f32,
    ) {
        self.velocity = self.velocity + Vector3::new(0.0, -self.gravity(), 0.0) * dt;

        let mut diff = (movement + self.velocity) * dt;

        /* apply collisions */
        let mut collider = self.collider();
        let blocks: Vec<AABB> = collider
            .extend(diff)
            .blocks()
            .filter_map(|v| world.block_at(v)?.aabb(ivec_to_f(v)))
            .collect();

        let save_y = diff.y;

        for block in &blocks {
            diff.y = block.offset(&collider, 1, diff.y);
        }
        collider = collider.translate3(0.0, diff.y, 0.0);

        for block in &blocks {
            diff.x = block.offset(&collider, 0, diff.x);
        }
        collider = collider.translate3(diff.x, 0.0, 0.0);

        for block in &blocks {
            diff.z = block.offset(&collider, 2, diff.z);
        }

        self.grounded = save_y < 0.0 && diff.y > save_y;

        /* move */
        self.set_position(world, listener, self.position + diff);
    }

    pub fn on_ground(&self) -> bool {
        self.grounded
    }

    pub fn in_water(&self) -> bool {
        false // FIXME
    }

    pub fn gravity(&self) -> f32 {
        match self.in_water() {
            true => WATER_GRAVITY,
            false => GRAVITY,
        }
    }

    fn update_sprint(&mut self, movement: Vector3<f32>, request: bool) {
        if self.on_ground() && request {
            self.sprinting = true;
        }

        if !request && movement.norm() < 1e-5 {
            self.sprinting = false;
        }
    }

    fn movement_speed(&self) -> f32 {
        let base = match self.in_water() {
            true => WATER_SPEED,
            false => SPEED,
        };

        let sprint = match self.sprinting {
            true => SPRINT_SPEED_MULTIPLIER,
            false => 1.0,
        };

        base * sprint
    }

    pub fn update(
        &mut self,
        world: &mut World,
        listener: &dyn ChunkListener,
        camera_forward: Vector3<f32>,
        camera_right: Vector3<f32>,
        inputs: Vec<PlayerInput>,
        dt: f32,
    ) {
        let mut directional_input: Vector2<f32> = Vector2::zeros();
        let mut jumping = false;
        let mut sprinting = false;

        for input in &inputs {
            match input {
                PlayerInput::MoveLeft => directional_input.x -= 1.,
                PlayerInput::MoveRight => directional_input.x += 1.,
                PlayerInput::MoveFoward => directional_input.y += 1.,
                PlayerInput::MoveBackward => directional_input.y -= 1.,
                PlayerInput::Jump => jumping = true,
                PlayerInput::SprintToggle => sprinting = true,
            }
        }

        let mut desired_move: Vector3<f32> =
            camera_forward * directional_input.y + camera_right * directional_input.x;
        desired_move.y = 0.0;

        self.update_sprint(desired_move, sprinting);

        if self.in_water() && jumping {
            desired_move.y = WATER_Y_SPEED;
        } else if self.on_ground() && jumping {
            self.velocity.y = JUMP_FORCE;
        }

        desired_move *= self.movement_speed();
        self.move_player(world, listener, desired_move, dt);
    }

    fn update_seen_chunks(
        &mut self,
        world: &World,
        listener: &dyn ChunkListener,
        position: Vector3<f32>,
    ) -> Vec<Vector2<i64>> {
        let (cx, cz) = worldf_to_chunk(position);

        let mut old_chunks = self.known_chunks.clone();
        let mut new_chunks = HashSet::new();

        for x in -self.view_distance..self.view_distance {
            for z in -self.view_distance..self.view_distance {
                let chunk = Vector2::new(cx + x, cz + z);

                old_chunks.remove(&chunk);
                new_chunks.insert(chunk);
            }
        }

        for chunk in &new_chunks {
            world.generate_chunk(chunk.x, chunk.y);
        }

        for chunk in &old_chunks {
            world.unload_chunk(chunk.x, chunk.y);
            listener.chunk_unload(chunk.x, chunk.y);
        }

        let chunks = new_chunks
            .difference(&self.known_chunks)
            .map(|v| *v)
            .collect();

        self.known_chunks = new_chunks;
        chunks
    }

    pub fn set_position(
        &mut self,
        world: &mut World,
        listener: &dyn ChunkListener,
        position: Vector3<f32>,
    ) {
        self.position = position;

        let dx = self.last_chunk_update.x - position.x;
        let dz = self.last_chunk_update.z - position.z;

        // update chunks only when the player moved half a chunk
        let new = if dx * dx + dz * dz < 64. {
            vec![]
        } else {
            self.last_chunk_update = position;
            self.update_seen_chunks(world, listener, position)
        };

        self.known_chunks
            .iter()
            .filter(|v| {
                if let Some(chunk) = world.chunk_mut(v.x, v.y) {
                    chunk.decorated() && (chunk.check_modified() || new.contains(&chunk.coords()))
                } else {
                    false
                }
            })
            .for_each(|v| listener.chunk_load(v.x, v.y));
    }
}
