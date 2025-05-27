use std::num::NonZeroU8;

pub mod quadtree;

type SimFloat = f32;

use cgmath::{
    Point2,
    Vector2,
};
use quadtree::{
    Positioned,
    Quadtree,
};
use wgpu::Color;

#[derive(Debug, Clone)]
pub struct Body {
    pub position: Point2<SimFloat>,
    pub velocity: Vector2<SimFloat>,
    pub mass: SimFloat,
    pub radius: SimFloat,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct Pseudobody {
    pub position: Point2<SimFloat>,
    pub mass: SimFloat,
}

impl Default for Pseudobody {
    fn default() -> Self {
        Self {
            position: Point2::new(0.0, 0.0),
            mass: 0.0,
        }
    }
}

impl Positioned for Body {
    fn position(&self) -> Point2<SimFloat> {
        self.position
    }
}

pub struct Simulation {
    bodies: Vec<Body>,
    quadtree: Quadtree<Body, Pseudobody>,

    // if the size of a pseudoparticle (s) divided by its distance (d) is below
    // this threshold, the pseudoparticle's mass is used and its children are ignored
    pseudobody_threshold: SimFloat,
}

impl Simulation {
    pub fn new<T>(
        num_bodies: usize,
        body_init: T,
        pseudobody_threshold: SimFloat,
    ) -> Self
    where
        T: Iterator<Item = Body>,
    {
        Self {
            bodies: Vec::from_iter(body_init.take(num_bodies)),
            quadtree: Quadtree::new(2.0),
            pseudobody_threshold
        }
    }

    pub fn advance(&mut self, dt: SimFloat) -> Result<(), String> {
        // 1. rebuild quadtree
        self.quadtree.clear();
        for body in self.bodies.iter() {
            self.quadtree.insert(body.clone())?;
        }

        // 2. calculate force for every body
        let mut forces: Vec<Vector2<SimFloat>> = Vec::with_capacity(self.bodies.len());
        for body in self.bodies.iter() {
            // start at root and resolve children until we are below the threshold
        }

        // 3. apply force to body

        Ok(())
    }

    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    pub fn quadtree(&self) -> &Quadtree<Body, Pseudobody> {
        &self.quadtree
    }
}
