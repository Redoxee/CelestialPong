// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

use std::{f32::consts::PI, sync::PoisonError};

use macroquad::{color, prelude::*};
use rand::Rng;
extern crate rand;

const WINDOW_SIZE: Vec2 = Vec2::from_array([1400., 800.]);

fn window_config() -> Conf {
    Conf {
        window_title: "Collision Simulator".to_owned(),
        window_width: WINDOW_SIZE.x.round() as i32,
        window_height: WINDOW_SIZE.y.round() as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug)]
struct Capsule {
    p1: Vec2,
    p2: Vec2,
    radius: f32,
    color: Color
}

// From : https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/
// Computes closest points C1 and C2 of S1(s)=P1+s*(Q1-P1) and
// S2(t)=P2+t*(Q2-P2), returning s and t. Function result is squared
// distance between between S1(s) and S2(t)
fn distance_point_segment_squared(p1 : Vec2, q1 : Vec2, p2 : Vec2, q2 : Vec2) -> f32 {
    let d1 = q1 - p1; // Direction vector of segment S1
    let d2 = q2 - p2; // Direction vector of segment S2
    let r = p1 - p2;
    let a = Vec2::dot(d1, d1); // Squared length of segment S1, always nonnegative
    let e = Vec2::dot(d2, d2); // Squared length of segment S2, always nonnegative
    let f = Vec2::dot(d2, r);
    let mut s;
    let mut t;
    
    // Check if either or both segments degenerate into points
    if a <= f32::EPSILON && e <= f32::EPSILON {
        // Both segments degenerate into points
        return Vec2::dot(p1 - p2, p1 - p2);
    }

    if a <= f32::EPSILON {
        // First segment degenerates into a point
        s = 0.0;
        t = f / e; // s = 0 => t = (b*s + f) / e = f / e
        t = f32::clamp(t, 0., 1.);
    } else {
        let c = Vec2::dot(d1, r);
        if e <= f32::EPSILON {
            // Second segment degenerates into a point
            t = 0.;
            s = f32::clamp(-c / a, 0., 1.); // t = 0 => s = (b*t - c) / a = -c / a
        } else {
            // The general nondegenerate case starts here
            let b = Vec2::dot(d1, d2);
            let denom = a * e - b * b; // Always nonnegative
            
            // If segments not parallel, compute closest point on L1 to L2 and
            // clamp to segment S1. Else pick arbitrary s (here 0)
            if denom != 0. {
                s = f32::clamp((b * f - c * e) / denom, 0., 1.);
            } else {
                s = 0.;
            }

            // Compute point on L2 closest to S1(s) using
            // t = Dot((P1 + D1*s) - P2,D2) / Dot(D2,D2) = (b*s + f) / e
            t = (b*s + f) / e;
            // If t in [0,1] done. Else clamp t, recompute s for the new value
            // of t using s = Dot((P2 + D2*t) - P1,D1) / Dot(D1,D1)= (t*b - c) / a
            // and clamp s to [0, 1]
            if t < 0. {
                t = 0.;
                s = f32::clamp(-c / a, 0., 1.);
            } else if t > 1. {
                t = 1.;
                s = f32::clamp((b - c) / a, 0., 1.);
            }
        }
    }

    let c1 = p1 + d1 * s;
    let c2 = p2 + d2 * t;
    return Vec2::dot(c1 - c2, c1 - c2);
}

impl Capsule {
    fn new(p1 : Vec2, p2 : Vec2, r : f32, color : Color) -> Capsule {
        return Capsule {
            p1 : p1,
            p2 : p2,
            radius : r,
            color : color
        };
    }

    fn draw(&self) {
        draw_circle_lines(self.p1.x, self.p1.y, self.radius, 2., self.color);
        draw_circle_lines(self.p2.x, self.p2.y, self.radius, 2., self.color);
        let dir = (self.p2 - self.p1).normalize();
        let cr = vec2(dir.y, -dir.x) * self.radius;
        draw_line(self.p1.x + cr.x, self.p1.y + cr.y, self.p2.x + cr.x, self.p2.y + cr.y, 2., self.color);
        draw_line(self.p1.x - cr.x, self.p1.y - cr.y, self.p2.x - cr.x, self.p2.y - cr.y, 2., self.color);
    }

    fn overlap(caps1 : Capsule, caps2 : Capsule) -> bool {
        let dist = distance_point_segment_squared(caps1.p1, caps1.p2, caps2.p1, caps2.p2);
        let r = caps1.radius + caps2.radius;
        return dist <= (r * r);
    }
}

static mut BALL_POS_INDEX : usize = 0;

#[derive(Clone, Copy, Debug)]
struct Ball {
    positions: [Vec2;2],
    velocity: Vec2,
    radius: f32,
    mass: f32,
    color: Color,
}

impl Ball {
    fn new(position:Vec2, velocity:Vec2, radius:f32, mass:f32, color:Color) -> Ball {
        let mut positions : [Vec2; 2] = Default::default();
        
        unsafe {
            positions[BALL_POS_INDEX] = position;
        }

        Ball {
            positions,
            velocity,
            radius,
            mass,
            color
        }
    }

    fn pos(&self) -> Vec2 {
        unsafe {
            self.positions[BALL_POS_INDEX]
        }
    }

    fn set_pos(&mut self, new_pos : Vec2) {
        unsafe {
            self.positions[BALL_POS_INDEX] = new_pos;
        }
    }

    fn draw(&self) {
        let pos = self.pos();
        draw_circle(pos.x, pos.y, self.radius, self.color);
    }

    fn update(&mut self, dt: f32, acc: Vec2) {
        self.velocity += acc * dt;
        let mut pos = self.pos();

        if pos.x < self.radius && self.velocity.x < 0.
            || WINDOW_SIZE.x - pos.x < self.radius && self.velocity.x > 0.
        {
            self.velocity.x *= -1.;
        }
        
        if pos.y < self.radius && self.velocity.y < 0.
            || WINDOW_SIZE.y - pos.y < self.radius && self.velocity.y > 0.
        {
            self.velocity.y *= -1.;
        }

        self.set_pos(pos + self.velocity);
    }

    fn check_collision(&self, other: &Ball) -> bool {
        other.pos().distance(self.pos()) <= other.radius + self.radius
    }

    // Does collision effect for both self and the other object
    // Based on https://www.vobarian.com/collisions/2dcollisions2.pdf
    // The individual steps from the document are commented
    fn collide(&mut self, other: &mut Ball) {
        let pos_diff = self.pos() - other.pos();

        // 1
        let unit_normal = pos_diff.normalize();
        let unit_tangent = Vec2::from((-unit_normal.y, unit_normal.x));

        // 3
        let v1n = self.velocity.dot(unit_normal);
        let v1t = self.velocity.dot(unit_tangent);
        let v2n = other.velocity.dot(unit_normal);
        let v2t = other.velocity.dot(unit_tangent);

        // 5
        let new_v1n =
            (v1n * (self.mass - other.mass) + 2. * other.mass * v2n) / (self.mass + other.mass);
        let new_v2n =
            (v2n * (other.mass - self.mass) + 2. * self.mass * v1n) / (self.mass + other.mass);

        // 6
        let final_v1n = new_v1n * unit_normal;
        let final_v1t = v1t * unit_tangent;
        let final_v2n = new_v2n * unit_normal;
        let final_v2t = v2t * unit_tangent;

        // 7
        let final_v1 = final_v1n + final_v1t;
        let final_v2 = final_v2n + final_v2t;

        // The if statement makes them not get stuck in each other
        if (self.velocity - other.velocity).dot(self.pos() - other.pos()) < 0. {
            self.velocity = final_v1;
            other.velocity = final_v2;
        }
    }
}

fn draw_cross(p : Vec2, color:Color) {
    draw_line(p.x - 5., p.y - 5., p.x + 5., p.y + 5., 1., color);
    draw_line(p.x - 5., p.y + 5., p.x + 5., p.y - 5., 1., color);
}

#[macroquad::main(window_config)]
async fn main() {
    let mut rng = rand::thread_rng();
    let mut paused = true;
    let mut drawing_enabled = true;

    let n_balls = 250;
    let mut balls = Vec::with_capacity(n_balls);

    for i in 0..n_balls {
        let r = 2.;
        balls.push(Ball::new(
            Vec2::from((r * 2. + r * 2. * i as f32, r * 2. + r * i as f32)),
            Vec2::from((rng.gen::<f32>() * 4. - 2., rng.gen::<f32>() * 4. - 2.)),
            r,
            PI * r.powf(2.),
            Color {
                r: rng.gen::<f32>() + 0.25,
                g: rng.gen::<f32>() + 0.25,
                b: rng.gen::<f32>() + 0.25,
                a: 1.,
            },
        ))
    }

    // test variables
    let mut spx = 300.;
    let mut spy = 180.;

    // acceleration
    let mut a;
    let strength = 5.;

    println!("{}", std::mem::size_of::<Ball>());

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

        a = Vec2::ZERO;
        if is_key_down(KeyCode::Left) {
            a.x = -strength;
        }
        if is_key_down(KeyCode::Up) {
            a.y = -strength;
        }
        if is_key_down(KeyCode::Right) {
            a.x = strength;
        }
        if is_key_down(KeyCode::Down) {
            a.y = strength;
        }

        if is_key_down(KeyCode::S) {
            for ball in &mut balls {
                ball.velocity *= 0.9;
            }
        }

        if !paused {
            let dt = get_frame_time();

            for ball in balls.iter_mut() {
                ball.update(dt, a);
            }

            balls.sort_by(|a, b| a.pos().x.partial_cmp(&b.pos().x).unwrap());
            let mut left_ball = 0;
            let mut right_bound = balls[left_ball].pos().x + balls[left_ball].radius;

            for i in 1..balls.len() {
                if balls[i].pos().x - balls[i].radius <= right_bound {
                    if balls[i].pos().x + balls[i].radius > right_bound {
                        right_bound = balls[i].pos().x + balls[i].radius;
                    }

                    let (left, right) = balls.split_at_mut(i);

                    for other_ball in &mut left[left_ball..i] {
                        if right[0].check_collision(other_ball) {
                            right[0].collide(other_ball);
                        }
                    }
                } else {
                    left_ball = i;
                    right_bound = balls[i].pos().x + balls[i].radius;
                }
            }
        }

        clear_background(BLACK);
        
        if drawing_enabled {
            for ball in &balls {
                ball.draw();
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            (spx, spy) = mouse_position();
        }

        let (mpx, mpy) = mouse_position();

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

        next_frame().await
    }
}