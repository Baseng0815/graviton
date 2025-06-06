use std::num::NonZeroU8;

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
            pseudobody_threshold,
        }
    }

    pub fn advance(
        &mut self,
        dt: SimFloat,
    ) -> Result<(), String> {
        // 1. rebuild quadtree
        self.quadtree.clear();
        for body in self.bodies.iter() {
            self.quadtree.insert(body.clone())?;
        }

        // 2. calculate pseudobodies
        self.calculate_pseudobody_from_node(0);

        // 3. calculate forces for every body
        let mut forces: Vec<Vector2<SimFloat>> = Vec::with_capacity(self.bodies.len());
        for body in self.bodies.iter() {
            // start at root and resolve children until we are below the threshold
            // self.quadtree.traverse_mut(&mut || {});
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

    fn calculate_pseudobody_from_node(
        &mut self,
        node_index: usize,
    ) {
        let node = self.quadtree.nodes()[node_index].as_ref();
        if let Some(node) = node {
            match node.child_index {
                QuadtreeChild::Node(children_index) => {
                    let mut child_mass_sum = 0.0;
                    let mut child_position_sum = Point2::new(0.0, 0.0);

                    let children_index = usize::try_from(children_index.get() - 1).unwrap();
                    for child_index in 0..4 {
                        self.calculate_pseudobody_from_node(children_index + child_index);
                        if let Some(child_node) = self.quadtree.nodes()[children_index + child_index] {
                            let child_data = child_node.data;
                            child_mass_sum += child_data.mass;
                            child_position_sum += child_data.position.to_vec();
                        }
                    }

                    child_mass_sum *= 0.25;
                    child_position_sum *= 0.25;

                    self.quadtree.nodes_mut()[node_index].as_mut().unwrap().data =
                        Pseudobody::new(child_position_sum, child_mass_sum);
                }
                QuadtreeChild::Element(element_index) => {
                    let element_index = usize::try_from(element_index.get() - 1).unwrap();
                    let element_position = self.quadtree.elements()[element_index].element.position();
                    let element_mass = self.quadtree.elements()[element_index].element.mass;

                    self.quadtree.nodes_mut()[node_index].as_mut().unwrap().data =
                        Pseudobody::new(element_position, element_mass);
                }
            }
        }
    }
}
