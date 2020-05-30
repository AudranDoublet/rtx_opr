use crate::{ivec_to_f, worldf_to_chunk, World, AABB, Block, BlockFace};
use nalgebra::{Vector2, Vector3};
use std::{collections::HashSet, rc::Rc};

const GRAVITY: f32 = 9.81;
const WATER_GRAVITY: f32 = 1.5;
const JUMP_FORCE: f32 = 6.;
const PLAYER_SIZE: f32 = 0.5;
const PLAYER_HEIGHT: f32 = 1.8;

const SPRINT_SPEED_MULTIPLIER: f32 = 1.5;
const SPEED: f32 = 5.0;
const WATER_SPEED: f32 = 1.0;
const FLYING_SPEED: f32 = 30.0;
const FLYING_Y_SPEED: f32 = 30.0;
const WATER_Y_SPEED: f32 = 1.0;

const BLOCK_BREAK_COOLDOWN: f32 = 0.3;
const BLOCK_PLACE_COOLDOWN: f32 = 0.3;

pub enum PlayerInput {
    Jump,
    SprintToggle,
    Sneaking,
    FlyToggle,
    MoveFoward,
    MoveBackward,
    MoveRight,
    MoveLeft,
    LeftInteract,
    RightInteract,
}

pub trait ChunkListener {
    /**
     * Called when a chunk is loaded or modified
     */
    fn chunk_load(&mut self, x: i32, y: i32);

    /**
     * Called when a chunk is unloaded
     */
    fn chunk_unload(&mut self, x: i32, y: i32);
}

pub struct Player {
    view_distance: i32,
    position: Vector3<f32>,
    sprinting: bool,
    grounded: bool,
    in_water: bool,
    sneaking: bool,

    flying: bool,

    velocity: Vector3<f32>,

    /** Chunk provider */
    last_chunk_update: Vector3<f32>,
    pub known_chunks: HashSet<Vector2<i32>>,

    block_break_cooldown: f32,
    block_place_cooldown: f32,
}

impl Player {
    pub fn new(view_distance: usize) -> Player {
        Player {
            view_distance: view_distance as i32,
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),

            sprinting: false,
            grounded: false,
            in_water: false,
            flying: true,
            sneaking: false,

            block_break_cooldown: 0.0,
            block_place_cooldown: 0.0,

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
        listener: &mut dyn ChunkListener,
        movement: Vector3<f32>,
        dt: f32,
    ) {
        self.velocity = self.velocity + Vector3::new(0.0, -self.gravity(), 0.0) * dt;

        if self.grounded && self.velocity.y < 0.0 {
            self.velocity.y = -self.gravity();
        }

        let mut diff = (movement + self.velocity) * dt;

        /* apply collisions */
        let mut collider = self.collider();
        let blocks: Vec<AABB> = collider
            .augment(diff)
            .blocks()
            .filter_map(|v| world.block_at(v)?.aabb(ivec_to_f(v)))
            .collect();

        let save_y = diff.y;

        if self.sneaking {
            let delta = 0.05;

            while diff.x != 0.0 && !self.collider().translate3(diff.x, -1.0, 0.0).has_blocks(world) {
                diff.x = match diff.x {
                    dx if dx < delta && dx > -delta => 0.0,
                    dx if dx > 0.0                  => dx - delta,
                    dx                              => dx + delta,
                };
            }

            while diff.z != 0.0 && !self.collider().translate3(diff.x, -1.0, diff.z).has_blocks(world) {
                diff.z = match diff.z {
                    dz if dz < delta && dz > -delta => 0.0,
                    dz if dz > 0.0                  => dz - delta,
                    dz                              => dz + delta,
                };
            }

            while diff.x != 0.0 && diff.z != 0.0 && !self.collider().translate3(diff.x, -1.0, diff.z).has_blocks(world) {
                diff.x = match diff.x {
                    dx if dx < delta && dx > -delta => 0.0,
                    dx if dx > 0.0                  => dx - delta,
                    dx                              => dx + delta,
                };

                diff.z = match diff.z {
                    dz if dz < delta && dz > -delta => 0.0,
                    dz if dz > 0.0                  => dz - delta,
                    dz                              => dz + delta,
                };
            }
        }

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

    pub fn head_position(&self) -> Vector3<f32> {
        self.position + Vector3::new(0.0, match self.sneaking {
            true => 1.3,
            false => 1.5,
        }, 0.0)
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn on_ground(&self) -> bool {
        self.grounded
    }

    pub fn in_water(&self) -> bool {
        self.in_water
    }

    pub fn gravity(&self) -> f32 {
        if self.flying {
            return 0.0;
        }

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
            _ if self.flying => FLYING_SPEED,
            true => WATER_SPEED,
            false => SPEED,
        };

        let sprint = match self.sprinting {
            true => SPRINT_SPEED_MULTIPLIER,
            false => 1.0,
        };

        base * sprint
    }

    pub fn looked_block(&self, world: &World, forward: Vector3<f32>) -> Option<(Vector3<i32>, BlockFace)> {
        let direction = forward.normalize();
        let origin = self.head_position();

        let inv_dir = Vector3::new(1. / direction.x, 1. / direction.y, 1. / direction.z);

        let bbox = AABB::new(origin, origin).augment(direction * 4.);

        let mut min = std::f32::INFINITY;
        let mut result = None;

        for pos in bbox.blocks() {
            if let Some(block) = world.block_at(pos) {
                if let Some(aabb) = block.aabb(ivec_to_f(pos)) {
                    if let Some((d, face)) = aabb.ray_intersects(origin, inv_dir) {
                        if d < min {
                            min = d;
                            result = Some((pos, face));
                        }
                    }
                }
            }
        }

        result
    }

    pub fn update(
        &mut self,
        world: &mut World,
        listener: &mut dyn ChunkListener,
        camera_forward: Vector3<f32>,
        camera_right: Vector3<f32>,
        inputs: Vec<PlayerInput>,
        dt: f32,
    ) {
        let mut directional_input: Vector2<f32> = Vector2::zeros();
        let mut jumping = false;
        let mut sprinting = false;
        let mut sneaking = false;

        self.block_break_cooldown -= dt;
        self.block_place_cooldown -= dt;

        for input in &inputs {
            match input {
                PlayerInput::MoveLeft => directional_input.x -= 1.,
                PlayerInput::MoveRight => directional_input.x += 1.,
                PlayerInput::MoveFoward => directional_input.y += 1.,
                PlayerInput::MoveBackward => directional_input.y -= 1.,
                PlayerInput::Jump => jumping = true,
                PlayerInput::Sneaking => sneaking = true,
                PlayerInput::SprintToggle => sprinting = true,
                PlayerInput::LeftInteract => {
                    if self.block_break_cooldown <= 0.0 {
                        if let Some((pos, _)) = self.looked_block(world, camera_forward) {
                            world.set_block_at(pos, Block::Air);
                            self.block_break_cooldown = BLOCK_BREAK_COOLDOWN;
                        }
                    }
                },
                PlayerInput::RightInteract => {
                    if self.block_place_cooldown <= 0.0 {
                        if let Some((pos, face)) = self.looked_block(world, camera_forward) {
                            let pos = pos + face.relative();
                            let btype = Block::TallGrass;

                            let mut allowed = true;

                            if let Some(aabb) = btype.aabb(ivec_to_f(pos)) {
                                if self.collider().box_intersects(&aabb) {
                                    allowed = false;
                                }
                            }

                            if allowed {
                                world.set_block_at(pos, btype);
                                self.block_place_cooldown = BLOCK_PLACE_COOLDOWN;
                            }
                        }
                    }
                },
                PlayerInput::FlyToggle => {
                    self.flying = !self.flying;
                    self.velocity.y = 0.0;
                },
            }
        }

        let mut desired_move: Vector3<f32> =
            camera_forward * directional_input.y + camera_right * directional_input.x;

        desired_move.y = 0.0;

        if desired_move.norm() > 1e-7 {
            desired_move.normalize_mut();
        }

        self.update_sprint(desired_move, sprinting);

        if self.flying && jumping {
            desired_move.y += FLYING_Y_SPEED / self.movement_speed();
        } else if self.in_water() && jumping {
            desired_move.y += WATER_Y_SPEED / self.movement_speed();
            self.velocity.y = 0.0;
        } else if self.on_ground() && jumping {
            self.velocity.y = JUMP_FORCE;
        }

        self.sneaking = false;

        if self.flying && sneaking {
            desired_move.y -= FLYING_Y_SPEED / self.movement_speed();
        } else if self.in_water() && sneaking {
            desired_move.y -= WATER_Y_SPEED/ self.movement_speed();
        } else if self.on_ground() && sneaking {
            self.sneaking = true;
        }

        desired_move *= self.movement_speed();
        self.move_player(world, listener, desired_move, dt);
    }

    fn update_seen_chunks(
        &mut self,
        world: &World,
        listener: &mut dyn ChunkListener,
        position: Vector3<f32>,
    ) -> Vec<Vector2<i32>> {
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

        let curr_chunk = Vector2::new(cx, cz);
        let mut new_chunks_vec: Vec<&Vector2<i32>> = new_chunks.iter().collect();

        new_chunks_vec.sort_by(|a, b| {
            let va = *a - curr_chunk;
            let vb = *b - curr_chunk;

            (va.x*va.x + va.y*va.y).cmp(&(vb.x*vb.x + vb.y*vb.y))
        });

        for chunk in new_chunks_vec {
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
        listener: &mut dyn ChunkListener,
        position: Vector3<f32>,
    ) {
        self.position = position;

        self.in_water = self.collider().blocks().any(|b| match world.block_at(b) {
            Some(v) => v.is_liquid(),
            _ => false,
        });

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
                    let chunk = unsafe { Rc::get_mut_unchecked(chunk) };
                    chunk.decorated() && (chunk.check_modified() || new.contains(&chunk.coords()))
                } else {
                    false
                }
            })
            .for_each(|v| listener.chunk_load(v.x, v.y));
    }
}
