use std::fmt::Debug;

use cgmath::{
    Point2,
    Vector2,
};
use wgpu::Color;

use crate::simulation::{quadtree::Positioned, Body};
use crate::simulation::quadtree::{ContinueTraverse, Quadtree};

use super::generic::{
    GenericVertex,
    Mesh,
    push_line,
};

pub(super) fn generate_quadtree_mesh<T, U>(quadtree: &Quadtree<T, U>) -> Mesh
where T: Positioned + Debug,
      U: Default + Debug + Copy + Clone
{
    let mut quadtree_mesh = Mesh::default();

    quadtree.traverse(&mut |node, node_position, depth| {
        let extent = quadtree.extent() * 0.5f32.powi(depth.try_into().unwrap());
        let p0 = node_position + Vector2::new(-extent, extent);
        let p1 = node_position + Vector2::new(-extent, -extent);
        let p2 = node_position + Vector2::new(extent, extent);
        let p3 = node_position + Vector2::new(extent, -extent);

        push_line(&mut quadtree_mesh, p0, p1, 0.003, Color::GREEN);
        push_line(&mut quadtree_mesh, p1, p3, 0.003, Color::GREEN);
        push_line(&mut quadtree_mesh, p3, p2, 0.003, Color::GREEN);
        push_line(&mut quadtree_mesh, p2, p0, 0.003, Color::GREEN);

        ContinueTraverse::Continue
    }).unwrap();

    quadtree_mesh
}
