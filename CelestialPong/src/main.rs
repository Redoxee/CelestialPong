// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

mod ball;
mod capsule;
mod quad_tree;

use macroquad::{
    color::{self, colors},
    prelude::*,
    window,
};
use rand::SeedableRng;

use rand::Rng;
use rand_chacha::ChaCha20Rng;
extern crate rand;

use crate::ball::*;
use crate::quad_tree::*;

const NB_BALLS: usize = 100;
const RADII: f32 = 10.;

const GRAVITY: f32 = 15000.;
const BODY_BOUNCYNESS: f32 = 0.9;

const MIN_START_ORBIT: f32 = 200.;
const MAX_START_ORBIT: f32 = 300.;

fn damping(pos: Vec2, target: Vec2, dt: f32, elasticity: f32) -> Vec2 {
    return (target - pos) / elasticity * dt;
}

fn get_gravity_force(ball: &Ball, body: &Ball) -> Vec2 {
    let delta = body.position - ball.position;
    return delta.normalize() * (body.mass * ball.mass) / delta.length().powf(2.) * GRAVITY;
}

fn get_orbital_velocity(b1: &Ball, b2: &Ball) -> Vec2 {
    let delta = b2.position - b1.position;
    let orbit_radius = delta.length();
    let speed = (GRAVITY * (b2.mass) / orbit_radius).sqrt();
    return Vec2::from((delta.y, -delta.x)).normalize() * speed;
}

fn random_orbital_pos(
    center: Vec2,
    min_radius: f32,
    max_radius: f32,
    rng: &mut ChaCha20Rng,
) -> Vec2 {
    let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.;
    let result = Vec2::from((angle.cos(), angle.sin()));
    let rad = rng.gen::<f32>() * (max_radius - min_radius) + min_radius;
    let result = center + result * rad;
    return result;
}

fn window_config() -> Conf {
    Conf {
        window_title: "Celestial pong".to_owned(),
        window_width: 1200,
        window_height: 1000,
        ..Default::default()
    }
}

fn reset_balls(
    balls: &mut Vec<Ball>,
    tree_area: quad_tree::Rect,
    static_bodies: &Vec<Ball>,
    mut rng: &mut ChaCha20Rng,
) {
    balls.clear();

    for i in 0..NB_BALLS {
        let position = random_orbital_pos(
            static_bodies[0].position,
            MIN_START_ORBIT,
            MAX_START_ORBIT,
            &mut rng,
        );

        let mut ball = Ball::new(
            position,
            Vec2::ZERO,
            RADII,
            1.,
            Color {
                r: rng.gen::<f32>() + 0.25,
                g: rng.gen::<f32>() + 0.25,
                b: rng.gen::<f32>() + 0.25,
                a: 1.,
            },
            tree_area,
        );

        // let ball_speed = Vec2::from((rng.gen::<f32>() * 20. - 10., rng.gen::<f32>() * 20. - 10.));
        let ball_speed = get_orbital_velocity(&ball, &static_bodies[0]);

        ball.velocity = ball_speed;

        balls.push(ball);
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let play_area_size = Vec2::new(window::screen_width(), window::screen_height());

    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(1);
    let mut paused = true;
    let mut drawing_enabled = true;

    let mut balls = Vec::with_capacity(NB_BALLS);
    let mut static_bodies = Vec::new();

    const FPS_FRAMES: usize = 100;
    let mut fps: [f32; FPS_FRAMES] = [0.; FPS_FRAMES];
    let mut fps_index: usize = 0;

    let mut selected_ball = None;

    let tree_area = quad_tree::Rect::new(
        play_area_size.x / 2.,
        play_area_size.y / 2.,
        play_area_size.x * 4.,
        play_area_size.x * 4.,
    );

    let mut quad_tree;

    static_bodies.push(Ball::new(
        Vec2::new(play_area_size.x / 2., play_area_size.y / 2.),
        Vec2::ZERO,
        30.,
        1000.,
        color::WHITE,
        tree_area,
    ));

    reset_balls(&mut balls, tree_area, &static_bodies, &mut rng);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            return;
        }

        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::V) {
            drawing_enabled = !drawing_enabled;
        }

        if is_key_down(KeyCode::S) {
            for ball in &mut balls {
                ball.velocity *= 0.5;
            }
        }

        if is_key_down(KeyCode::R) {
            selected_ball = None;
            paused = true;
            rng = rand_chacha::ChaChaRng::seed_from_u64(1);
            reset_balls(&mut balls, tree_area, &static_bodies, &mut rng);
        }

        if is_key_down(KeyCode::O) {
            for ball in &mut balls {
                ball.velocity = get_orbital_velocity(ball, &static_bodies[0]);
            }
        }

        let dt = get_frame_time();
        fps[fps_index] = dt;
        fps_index = (fps_index + 1) % FPS_FRAMES;

        let dt = 1. / 60.;

        quad_tree = QuadTree::new(tree_area);

        if !paused {
            for index in 0..balls.len() {
                let ball = balls.get_mut(index).unwrap();
                quad_tree.add(QuadTreeEntry::new(ball.position, index));

                let mut local_force = Vec2::ZERO;
                if selected_ball == None || selected_ball.unwrap() != index {
                    for body in &static_bodies {
                        local_force = local_force + get_gravity_force(ball, body)
                    }
                }

                ball.update(dt, local_force);
            }

            for i in 0..balls.len() {
                let zone_check = balls[i].get_collision_area();
                let mut near_balls = Vec::new();
                quad_tree.query_entries(&zone_check, &mut near_balls);
                for entry in near_balls {
                    if entry.payload == i {
                        continue;
                    }

                    let other_ball_index = entry.payload;

                    if balls[i].check_collision(&balls[other_ball_index]) {
                        if i > other_ball_index {
                            let (left, right) = balls.split_at_mut(i);
                            right[0].collide(&mut left[other_ball_index]);
                        } else {
                            let (left, right) = balls.split_at_mut(other_ball_index);
                            right[0].collide(&mut left[i]);
                        }
                    }
                }
            }

            for body in static_bodies.iter_mut() {
                let query = body.get_collision_area();
                let mut near_objects = Vec::new();
                quad_tree.query_entries(&query, &mut near_objects);
                for near in near_objects {
                    let ball = balls.get_mut(near.payload).unwrap();
                    if body.check_collision(&ball) {
                        let delta = ball.position - body.position;
                        if delta.dot(ball.velocity) < 0. && ball.velocity.length_squared() > 0.001 {
                            let delta = delta.normalize();
                            ball.position = body.position + delta * (body.radius + ball.radius);
                            ball.velocity = (ball.velocity - 2. * delta.dot(ball.velocity) * delta)
                                * BODY_BOUNCYNESS;
                        }
                    }
                }
            }
        }

        let (spx, spy) = mouse_position();
        let mouse_pos = Vec2::new(spx, spy);
        let mut near_balls = Vec::new();
        quad_tree.query_entries(
            &quad_tree::Rect::new(spx, spy, RADII * 2., RADII * 2.),
            &mut near_balls,
        );

        let dist_check = RADII * RADII;
        let under = near_balls
            .into_iter()
            .find(|b| (balls[b.payload].position - mouse_pos).length_squared() < dist_check);

        match under {
            Some(entry) => {
                let b = balls[entry.payload];
                draw_circle_lines(b.position.x, b.position.y, b.radius, 2., colors::GOLD);

                let dist = (b.position - static_bodies[0].position).length();
                draw_text_ex(
                    &format!("dist : {}", dist),
                    32.,
                    64.,
                    TextParams {
                        font_size: 15,
                        ..Default::default()
                    },
                );
            }
            _ => {}
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            match under {
                Some(entry) => {
                    selected_ball = Some(entry.payload);
                }
                _ => {}
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            selected_ball = None;
        }

        match selected_ball {
            Some(ball_index) => {
                let ball = balls.get_mut(ball_index).unwrap();
                let force = damping(ball.position, mouse_pos, dt, 0.01);
                let delta = (mouse_pos - ball.position);
                let mut factor = 0.;
                if delta != Vec2::ZERO {
                    factor = delta.length() / 1000.;
                }

                ball.velocity = ball.velocity * factor + delta;
            }
            _ => {}
        }

        let mean_fps = Iterator::sum::<f32>(fps.iter()) / FPS_FRAMES as f32;
        draw_text_ex(
            &format!("fps : {}", 1. / mean_fps),
            32.,
            32.,
            TextParams {
                font_size: 15,
                ..Default::default()
            },
        );

        if drawing_enabled {
            for ball in &balls {
                ball.draw();
                // ball.get_collision_area().debug_draw(1., ball.color);
            }

            for body in &static_bodies {
                body.draw();
            }
            // quad_tree.debug_draw();
        }

        /*
        if is_mouse_button_pressed(MouseButton::Left) {
            (spx, spy) = mouse_position();
        }


        let caps1 = Capsule::new(
            Vec2 { x: spx, y: spy },
            Vec2 { x: mpx, y: mpy },
            30.,
            color::BLUE);

        let caps2 = Capsule::new(
            Vec2 { x: 200., y: 250. },
            Vec2 { x: 400., y: 250. },
            15.,
            color::BLUE);

        caps1.draw();
        caps2.draw();

        match Capsule::overlap(caps1, caps2) {
            true => {
                draw_text("overlap", 50., 50., 18., color::BEIGE);
                let d1 = caps1.p2 - caps1.p1;
                let v1 = f32::sqrt(Vec2::dot(d1, d1));
                let factor1 =  v1 / caps1.radius;
                let d2 = caps2.p2 - caps2.p1;
                let v2 = f32::sqrt(Vec2::dot(d2, d2));
                let factor2 =  v2 / caps2.radius;
                let msg = format!("{factor1:.3} | {factor2:.3}");
                draw_text(&msg, 50., 70., 18., color::BEIGE);
                let factor = f32::max(factor1, factor2);
                let iterations = factor.ceil() as i32;
                for i in 0..=iterations {
                    let p1t = caps1.p1 + (d1 * i as f32 / iterations as f32);
                    let p2t = caps2.p1 + (d2 * i as f32 / iterations as f32);

                    let radi_squared = caps1.radius + caps2.radius;
                    let radi_squared = radi_squared * radi_squared;
                    let delta = p2t - p1t;
                    if Vec2::dot(delta, delta) < radi_squared {
                        draw_circle_lines(p1t.x, p1t.y, caps1.radius,1., color::GOLD);
                        draw_circle_lines(p2t.x, p2t.y, caps2.radius,1., color::GOLD);
                        break;
                    }
                }
            }
            _ => {}
        };
        */

        next_frame().await
    }
}