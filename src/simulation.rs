use std::num::NonZeroU8;

pub mod quadtree;

use cgmath::{Point2, Vector2};
use quadtree::Positioned;
use wgpu::Color;

#[derive(Debug, Clone)]
pub struct Body {
    pub position: Point2<f32>,
    pub velocity: Vector2<f32>,
    pub radius: f32,
    pub color: Color,
}

impl Positioned for Body {
    fn position(&self) -> Point2<f32> {
        self.position
    }
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
