use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
struct Capsule {
    p1: Vec2,
    p2: Vec2,
    radius: f32,
    color: Color,
}

// From : https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/
// Computes closest points C1 and C2 of S1(s)=P1+s*(Q1-P1) and
// S2(t)=P2+t*(Q2-P2), returning s and t. Function result is squared
// distance between between S1(s) and S2(t)
fn distance_point_segment_squared(p1: Vec2, q1: Vec2, p2: Vec2, q2: Vec2) -> f32 {
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
            t = (b * s + f) / e;
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
    fn new(p1: Vec2, p2: Vec2, r: f32, color: Color) -> Capsule {
        return Capsule {
            p1: p1,
            p2: p2,
            radius: r,
            color: color,
        };
    }

    fn draw(&self) {
        draw_circle_lines(self.p1.x, self.p1.y, self.radius, 2., self.color);
        draw_circle_lines(self.p2.x, self.p2.y, self.radius, 2., self.color);
        let dir = (self.p2 - self.p1).normalize();
        let cr = vec2(dir.y, -dir.x) * self.radius;
        draw_line(
            self.p1.x + cr.x,
            self.p1.y + cr.y,
            self.p2.x + cr.x,
            self.p2.y + cr.y,
            2.,
            self.color,
        );
        draw_line(
            self.p1.x - cr.x,
            self.p1.y - cr.y,
            self.p2.x - cr.x,
            self.p2.y - cr.y,
            2.,
            self.color,
        );
    }

    fn overlap(caps1: Capsule, caps2: Capsule) -> bool {
        let dist = distance_point_segment_squared(caps1.p1, caps1.p2, caps2.p1, caps2.p2);
        let r = caps1.radius + caps2.radius;
        return dist <= (r * r);
    }
}
