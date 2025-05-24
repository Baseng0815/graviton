use cgmath::{Point2, Vector2};
use wgpu::Color;

pub struct Body {
    // the bottom-left corner of the simulation coordinate system is (0, 0) and the extend of the axes is
    // [0, 1]
    pub position: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub radius: f32,
    pub color: Color,
}

pub struct Simulation {
    pub bodies: Vec<Body>,
}

impl Simulation {
    pub fn new<T>(num_bodies: usize, body_init: T) -> Self
        where T: Iterator<Item = Body>
    {
        Self {
            bodies: Vec::from_iter(body_init.take(num_bodies))
        }
    }

    pub fn advance(&mut self) {

    }
}
