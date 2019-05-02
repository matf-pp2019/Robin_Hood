extern crate ggez;
extern crate ncollide2d;
extern crate rand;
extern crate nalgebra as na;

mod map;
mod guard;
mod anim;
mod player;

use ggez::*;
use ggez::graphics::Drawable;
use na::{Vector2, Isometry2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use ncollide2d::world::{CollisionGroups, CollisionWorld, GeometricQueryType};

use crate::anim::Direction;
use crate::guard::Guard;
use crate::player::Player;

use std::time::{Duration, Instant};

const SCREEN_SIZE: (f32, f32) = (
    640.0, 480.0
    );

const UPDATES_PER_SECOND: f32 = 30.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

/*
#[derive(Debug)]
struct Player {
    pos: mint::Point2<f32>,
    direction: Vector2<f32>,
    collision_ver: Direction,
    collision_hor: Direction,
    walking: bool,
    run_left: Animation,
    run_right: Animation,
    run_down: Animation,
    run_up: Animation,
    idle: graphics::Image,
    animation_state: Direction,
    spd: f32,
    col_handle: CollisionObjectHandle,
    visibility: Vec<mint::Point2<f32>>,
}

impl Player {
    pub fn new(ctx: &mut Context, handle: CollisionObjectHandle) -> Self {
        Player {
            pos: mint::Point2 {x: 64.0, y: 64.0},
            direction: Vector2::new(0.0, 0.0),
            collision_ver: Direction::Null,
            collision_hor: Direction::Null,
            walking: false,
            run_left: Animation::new(ctx, "/images/robin_runleft.png"),
            run_right: Animation::new(ctx, "/images/robin_runright.png"),
            run_down: Animation::new(ctx, "/images/robin_rundown.png"),
            run_up: Animation::new(ctx, "/images/robin_runup.png"),
            idle: graphics::Image::new(ctx, "/images/robin_idle.png").unwrap(),
            animation_state: Direction::Null,
            spd: 4.0,
            col_handle: handle,
            visibility: Vec::new(),

        }
    }

    fn pos_from_move(&self) -> mint::Point2<f32> {
        // ova f-ja se poziva pri svakom apdejtu
        // na osnovu trenutne pozicije i pravca kretanja
        // vraca "sledecu" poziciju igraca
        let mut norm_dir = Vector2::new(0.0, 0.0);
        if self.direction.x != 0.0 || self.direction.y != 0.0 {
            norm_dir = self.direction.normalize();
        }

        mint::Point2 { x: self.pos.x + norm_dir.x * self.spd, y: self.pos.y + norm_dir.y * self.spd }
    }

    pub fn shape_pos(&self, p: Option<mint::Point2<f32>>) -> Isometry2<f32> {
        // Collider zahteva Isometry2 za poziciju oblika
        match p {
            Some(d) => Isometry2::new(Vector2::new(d.x-1.5, d.y+8.0), 0.0),
            None => Isometry2::new(Vector2::new(self.pos.x-1.5, self.pos.y+8.0), 0.0),
        }
    }

    fn update(&mut self, ctx: &mut Context, world: &mut CollisionWorld<f32, ()>, map_handle: CollisionObjectHandle, corners: &mut Vec<mint::Point2<f32>>) {
        /* self.walking je korisno za animaciju
         * npr. if self.walking {
         *          curr_animation = walk_animation;
         *      } else {
         *          curr_animation = idle_animation;
         *      }
         */
        self.visibility.clear();
        // corners.sort_by(|a, b| Rotation2::rotation_between(&Vector2::x(), &Vector2::new(a.x, a.y)).angle().partial_cmp(&Rotation2::rotation_between(&Vector2::x(), &Vector2::new(b.x, b.y)).angle()).unwrap());

        let origin_point = mint::Point2 { x: self.pos.x+5.0, y: self.pos.y+4.0 };
        for corner in corners.iter() {
            let ray = Ray::new(Point2::new(origin_point.x, origin_point.y), Vector2::new(corner.x-origin_point.x, corner.y-origin_point.y).normalize());
            let map_shape: &Compound<f32> = world.collision_object(map_handle).unwrap().shape().as_shape().unwrap();
            let intersect = (*map_shape).toi_with_ray(&Isometry2::identity(), &ray, true).unwrap();
            let ipoint = ray.point_at(intersect);
            self.visibility.push(mint::Point2 { x: ipoint.x, y: ipoint.y });
        }
        self.visibility.sort_by(|a, b| Rotation2::rotation_between(&Vector2::x(), &Vector2::new(origin_point.x - a.x, origin_point.y - a.y)).angle().partial_cmp(&Rotation2::rotation_between(&Vector2::x(), &Vector2::new(origin_point.x - b.x, origin_point.y - b.y)).angle()).unwrap());


        if self.direction.x == 0.0 && self.direction.y == 0.0 {
            self.walking = false;
            self.animation_state = Direction::Null;
            self.run_right.reset();
            self.run_left.reset();
            self.run_down.reset();
            self.run_up.reset();

        } else if self.direction.x == 1.0 && self.direction.y == 0.0 {
            self.animation_state = Direction::Right;
            self.walking = true;
        }
        else if self.direction.x == -1.0 && self.direction.y == 0.0 {
            self.animation_state = Direction::Left;
            self.walking = true;
        }
        else if self.direction.x == 0.0 && self.direction.y == 1.0 {
            self.animation_state = Direction::Down;
            self.walking = true;
        }
        else if self.direction.x == 0.0 && self.direction.y == -1.0 {
            self.animation_state = Direction::Up;
            self.walking = true;
        }
        else { self.walking = true; }
        if self.walking {
            match self.animation_state {
                Direction::Right => self.run_right.next_frame(),
                Direction::Left => self.run_left.next_frame(),
                Direction::Up => self.run_up.next_frame(),
                Direction::Down => self.run_down.next_frame(),
                Direction::Null => (),
            }

            // mora da postoji neki bolji nacin da se ovo uradi
            let mut old_dir = self.direction.clone();
            if self.collision_hor == Direction::Left && input::keyboard::is_key_pressed(ctx, event::KeyCode::Left) {
                self.direction = old_dir;
                self.direction.x = -1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.x = -1.0;
                        ()
                    },
                }
            }
            if self.collision_hor == Direction::Right && input::keyboard::is_key_pressed(ctx, event::KeyCode::Right){
                self.direction = old_dir;
                self.direction.x = 1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.x = 1.0;
                        ()
                    },
                }
            }
            if self.collision_ver == Direction::Up && input::keyboard::is_key_pressed(ctx, event::KeyCode::Up) {
                self.direction = old_dir;
                self.direction.y = -1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.y = -1.0;
                        ()
                    },
                }
            }
            if self.collision_ver == Direction::Down && input::keyboard::is_key_pressed(ctx, event::KeyCode::Down) {
                self.direction = old_dir;
                self.direction.y = 1.0;
                let new_pos = self.pos_from_move();
                world.set_position(self.col_handle, self.shape_pos(Some(new_pos)));
                match world.contact_pair(self.col_handle, map_handle, true) {
                    Some(_) => (),
                    None => {
                        self.collision_hor = Direction::Null;
                        old_dir.y = 1.0;
                        ()
                    },
                }
            }
            self.direction = old_dir;
            world.set_position(self.col_handle, self.shape_pos(None));
            if self.collision_hor == Direction::Right && self.direction.x == -1.0 {
                self.collision_hor = Direction::Null;
            }
            if self.collision_hor == Direction::Left && self.direction.x == 1.0 {
                self.collision_hor = Direction::Null;
            }
            if self.collision_ver == Direction::Down && self.direction.y == -1.0 {
                self.collision_ver = Direction::Null;
            }
            if self.collision_ver == Direction::Up && self.direction.y == 1.0 {
                self.collision_ver = Direction::Null;
            }

            match world.contact_pair(self.col_handle, map_handle, true) {
                Some(c) => {
                    let col = c.3;
                    let dcontact = &col.deepest_contact().unwrap().contact;
                    let ddepth = dcontact.depth;
                    let dvector = dcontact.normal.into_inner();
                    if ddepth >= 0.0 && ddepth < 13.0 {
                        if dvector.x == -1.0 && self.direction.x == 1.0 {
                            self.collision_hor = Direction::Right;
                            self.direction.x = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x-ddepth, y: self.pos.y };
                        }
                        if dvector.x == 1.0 && self.direction.x == -1.0 {
                            self.collision_hor = Direction::Left;
                            self.direction.x = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x-ddepth, y: self.pos.y };
                        }
                        if dvector.y == -1.0 && self.direction.y == 1.0 {
                            self.collision_ver = Direction::Down;
                            self.direction.y = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x, y: self.pos.y-ddepth };
                        }
                        if dvector.y == 1.0 && self.direction.y == -1.0 {
                            self.collision_ver = Direction::Up;
                            self.direction.y = 0.0;
                            self.pos = mint::Point2 { x: self.pos.x, y: self.pos.y+ddepth};
                        }
                        // self.pos = mint::Point2 { x: self.pos.x+dvector.x*(ddepth), y: self.pos.y+dvector.y*(ddepth) };
                    }
                    // self.pos = self.pos_from_move();

                    // world.set_position(self.col_handle, self.shape_pos(None));
                    ()
                },
                None => {
                    self.collision_ver = Direction::Null;
                    self.collision_hor = Direction::Null;
                    ()
                }

            }
            self.pos = self.pos_from_move();
            world.set_position(self.col_handle, self.shape_pos(None));

        }
    }

    fn draw_visibility(&self, ctx: &mut Context) -> GameResult<()> {
        if self.visibility.len() > 0 && self.visibility[0] != self.visibility[1] {
            let vis_clone = self.visibility.clone();

            let vis_mesh_test: graphics::Mesh = graphics::Mesh::new_polygon(ctx, graphics::DrawMode::fill(), &vis_clone, [0.0, 1.0, 0.0, 0.5].into())?;
            // let mut whole_screen: graphics::Mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), [0.0, 0.0, SCREEN_SIZE.0, SCREEN_SIZE.1].into(), [0.0, 0.0, 0.0, 1.0].into())?;
            // whole_screen.set_blend_mode(Some(graphics::BlendMode::Subtract));
            // vis_mesh_test.set_blend_mode(Some(graphics::BlendMode::Invert));
            // let vis_mesh = vis_mesh_builder.build(ctx)?;
            // graphics::draw(ctx, &whole_screen, graphics::DrawParam::new())?;
            graphics::draw(ctx, &vis_mesh_test, graphics::DrawParam::new())?;
        }
        Ok(())
    }

    fn draw(&self, ctx: &mut Context, show_mesh: bool) -> GameResult<()> {
        match self.animation_state {
            Direction::Right => self.run_right.draw(ctx, self.pos)?,
            Direction::Left => self.run_left.draw(ctx, self.pos)?,
            Direction::Up => self.run_up.draw(ctx, self.pos)?,
            Direction::Down => self.run_down.draw(ctx, self.pos)?,
            Direction::Null => graphics::draw(ctx, &self.idle, graphics::DrawParam::new().dest(self.pos))?,
        }
        if show_mesh {
            let shape_mesh = graphics::MeshBuilder::new().rectangle(graphics::DrawMode::stroke(3.0), graphics::Rect::new(self.shape_pos(None).translation.vector.x, self.shape_pos(None).translation.vector.y, 24.0, 16.0), [1.0, 0.0, 0.0, 1.0].into()).build(ctx)?;
            graphics::draw(ctx, &shape_mesh, graphics::DrawParam::new())?;
        }
        Ok(())
    }
}
*/

struct GameState {
    castle_map: map::Map,
    player: Player,
    guard: Guard,
    world: CollisionWorld<f32, ()>,
    last_update: Instant,
}

impl GameState {
    pub fn new(ctx: &mut Context) -> Self {
        let mut world_mut = CollisionWorld::new(0.02);
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(12.0, 8.0)));
        let mut groups = CollisionGroups::new();
        groups.set_membership(&[0 as usize]);
        groups.set_blacklist(&[0 as usize]);
        groups.set_whitelist(&[1 as usize]);
        let query = GeometricQueryType::Contacts(0.0, 0.0);

        GameState {
            castle_map: map::Map::load(ctx, "/levels/level1.txt", "/images/castle_spritesheet.png", mint::Point2 { x:0.0, y:0.0 }, mint::Point2 { x:32.0, y:32.0 }, &mut world_mut).unwrap(),
            player: Player::new(ctx, world_mut.add(Isometry2::new(Vector2::new(64.0, 74.0), 0.0), shape.clone(), groups, query, ()).handle()),
            guard: Guard::new(ctx, mint::Point2{x: 1.0*32.0, y: 8.0*32.0}, mint::Point2{x: 10.0*32.0, y: 11.0*32.0}, 4),
            world: world_mut,
            last_update: Instant::now(),
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // da kontrolisemo broj apdejta u sekundi, ili FPS
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            self.player.update(ctx, &mut self.world, self.castle_map.map_handle, &mut self.castle_map.get_corners());
            self.world.update();
            self.guard.update();
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.2, 0.2, 0.2, 1.0].into());
        self.castle_map.draw(ctx, 1, false)?;
        self.player.draw(ctx, false)?;
        self.guard.draw(ctx)?;
        self.castle_map.draw(ctx, 2, false)?;
        // self.player.draw_visibility(ctx)?;
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
        ) {
        match keycode {
            event::KeyCode::Up if self.player.collision_ver != Direction::Up => {
                self.player.direction.y = -1.0;
                self.player.animation_state = Direction::Up;
            },
            event::KeyCode::Down if self.player.collision_ver != Direction::Down => {
                self.player.direction.y = 1.0;
                self.player.animation_state = Direction::Down;
            },
            event::KeyCode::Left if self.player.collision_hor != Direction::Left => {
                self.player.direction.x = -1.0;
                self.player.animation_state = Direction::Left;
            },
            event::KeyCode::Right if self.player.collision_hor != Direction::Right => {
                self.player.direction.x = 1.0;
                self.player.animation_state = Direction::Right;
            },
            _ => (),
        }
    }
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        ) {
        match keycode {
            event::KeyCode::Up if self.player.direction.y == -1.0 => self.player.direction.y = 0.0,
            event::KeyCode::Down if self.player.direction.y == 1.0 => self.player.direction.y = 0.0,
            event::KeyCode::Left if self.player.direction.x == -1.0 => self.player.direction.x = 0.0,
            event::KeyCode::Right if self.player.direction.x == 1.0 => self.player.direction.x = 0.0,
            _ => (),
        }
    }
}

fn main() -> GameResult {
    let (ctx, events_loop) = &mut ContextBuilder::new("robin_hood", "lkh01")
        .window_setup(conf::WindowSetup::default().title("Robin Hood"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = &mut GameState::new(ctx);
    event::run(ctx, events_loop, state)
}
