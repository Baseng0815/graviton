use std::num::NonZeroU8;
use std::time::{
    Duration,
    Instant,
};

pub mod quadtree;

type SimFloat = f32;

use cgmath::{
    EuclideanSpace,
    Point2,
    Vector2,
};
use quadtree::{
    Positioned,
    Quadtree,
    QuadtreeChild,
};
use wgpu::Color;

use crate::new_map_key;
use crate::utility::index_map::{MapKey, PrimaryMap};

#[derive(Debug, Clone)]
pub struct Body {
    pub position: Point2<SimFloat>,
    pub velocity: Vector2<SimFloat>,
    pub mass: SimFloat,
    pub radius: SimFloat,
    pub color: Color,
}

impl Body {
    pub fn new(
        position: Point2<SimFloat>,
        velocity: Vector2<SimFloat>,
        mass: SimFloat,
        radius: SimFloat,
        color: Color,
    ) -> Self {
        Self {
            position,
            velocity,
            mass,
            radius,
            color,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn radius(&self) -> SimFloat {
        self.radius
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pseudobody {
    position: Point2<SimFloat>,
    mass: SimFloat,
}

impl Pseudobody {
    pub fn new(
        position: Point2<SimFloat>,
        mass: SimFloat,
    ) -> Self {
        Self { position, mass }
    }
}

impl Default for Pseudobody {
    fn default() -> Self {
        Self {
            position: Point2::new(0.0, 0.0),
            mass: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct QuadtreeBody {
    position: Point2<SimFloat>,
    body_key: BodyKey,
}

impl Positioned for QuadtreeBody {
    fn position(&self) -> Point2<SimFloat> {
        self.position
    }
}

new_map_key! { pub struct BodyKey; "BODY"; }

pub struct Simulation {
    bodies: PrimaryMap<BodyKey, Body>,
    quadtree: Quadtree<QuadtreeBody, Pseudobody>,

    // if the size of a pseudoparticle (s) divided by its distance (d) is below
    // this threshold, the pseudoparticle's mass is used and its children are ignored
    pseudobody_threshold: SimFloat,
}

impl Simulation {
    pub fn new<T>(
        bodies: T,
        pseudobody_threshold: SimFloat,
    ) -> Self
    where
        T: ExactSizeIterator<Item = Body>,
    {
        let mut slf = Self {
            bodies: PrimaryMap::with_capacity(bodies.len()),
            quadtree: Quadtree::new(10000.0),
            pseudobody_threshold,
        };

        for body in bodies {
            slf.bodies.insert(body);
        }

        slf
    }

    pub fn advance(
        &mut self,
        dt: Duration,
    ) -> Result<(), String> {
        log::trace!("Updating simulation with dt={:?}", dt);

        // 0. apply old velocity
        for body in self.bodies.values_mut() {
            body.position += body.velocity * dt.as_millis() as SimFloat;
        }

        // 1. rebuild quadtree
        let start = Instant::now();

        self.quadtree.clear();
        for (body_key, body) in self.bodies.items() {
            self.quadtree.insert(QuadtreeBody { position: body.position, body_key })?;
        }

        let duration = Instant::now() - start;
        log::trace!("Built quadtree with {} nodes in {:?}", self.quadtree.nodes().len(), duration);

        // 2. calculate pseudobodies
        let start = Instant::now();


        let duration = Instant::now() - start;
        log::trace!("Calculated pseudobodies in {:?}", duration);

        // 3. calculate forces for every body and update velocities
        let start = Instant::now();



        let duration = Instant::now() - start;
        log::trace!("Calculated forces in {:?}", duration);

        // 3. apply force to body

        Ok(())
    }

    pub fn bodies(&self) -> impl ExactSizeIterator<Item = &Body> {
        self.bodies.values()
    }

    pub fn quadtree(&self) -> &Quadtree<QuadtreeBody, Pseudobody> {
        &self.quadtree
    }

    fn calculate_body_force(
        &self,
        body: &Body,
    ) -> Vector2<SimFloat> {

        // start at root and resolve children until we are below the threshold
        // let
        todo!()
    }
}
