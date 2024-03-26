use macroquad::prelude::*;

use crate::quad_tree::{self, Rect};

#[derive(Clone, Copy, Debug)]
pub struct Ball {
    pub position: Vec2,
    pub prev_position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub mass: f32,
    pub color: Color,
    playing_field: Rect,
}

impl Ball {
    pub fn new(
        position: Vec2,
        velocity: Vec2,
        radius: f32,
        mass: f32,
        color: Color,
        playing_field: Rect,
    ) -> Ball {
        Ball {
            position,
            prev_position: position - velocity,
            velocity,
            radius,
            mass,
            color,
            playing_field,
        }
    }

    pub fn get_collision_area(&self) -> quad_tree::Rect {
        let p = self.position;
        let s = self.radius * 4.;
        quad_tree::Rect::new(p.x, p.y, s, s)
    }

    pub fn draw(&self) {
        let pos = self.position;
        draw_circle(pos.x, pos.y, self.radius, self.color);
    }

    pub fn update(&mut self, dt: f32, acc: Vec2) {
        self.velocity += acc * dt;
        let pos = self.position;

        if pos.x < self.playing_field.left && self.velocity.x < 0.
            || pos.x > self.playing_field.right && self.velocity.x > 0.
        {
            self.velocity.x *= -1.;
        }

        if pos.y < self.playing_field.up && self.velocity.y < 0.
            || pos.y > self.playing_field.down && self.velocity.y > 0.
        {
            self.velocity.y *= -1.;
        }

        self.prev_position = self.position;
        self.position = pos + self.velocity * dt;
    }

    pub fn update_verlet(&mut self, dt: f32, acc: Vec2) {
        let temp_pos = self.position;
        self.position = self.position * 2. - self.prev_position + acc * dt * dt;
        self.prev_position = temp_pos;
        self.velocity = (self.position - self.prev_position) / dt;
    }

    pub fn set_velocity(&mut self, velocity: Vec2, dt: f32) {
        self.prev_position = self.position + -velocity * dt;
        self.velocity = velocity;
    }

    pub fn check_collision(&self, other: &Ball) -> bool {
        other.position.distance(self.position) <= other.radius + self.radius
    }

    // Does collision effect for both self and the other object
    // Based on https://www.vobarian.com/collisions/2dcollisions2.pdf
    // The individual steps from the document are commented
    pub fn collide(&mut self, other: &mut Ball, dt: f32) {
        const HEAT_DISIPATION: f32 = 0.999;
        let pos_diff = self.position - other.position;

        // 1
        let unit_normal = pos_diff.normalize();
        let unit_tangent = Vec2::from((-unit_normal.y, unit_normal.x));

        // 3
        let v1n = self.velocity.dot(unit_normal);
        let v1t = self.velocity.dot(unit_tangent) * HEAT_DISIPATION;
        let v2n = other.velocity.dot(unit_normal);
        let v2t = other.velocity.dot(unit_tangent) * HEAT_DISIPATION;

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
        if (self.velocity - other.velocity).dot(self.position - other.position) < 0. {
            self.set_velocity(final_v1, dt);
            other.set_velocity(final_v2, dt);
        }
    }
}
