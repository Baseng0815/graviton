use std::any::Any;
use std::fmt::Debug;
use std::iter;
use std::num::{
    NonZeroU8,
    NonZeroU32,
};

use cgmath::{
    Array,
    Point2,
    Vector2,
};

use super::SimFloat;

const MAX_DEPTH: u32 = 64;

pub trait Positioned {
    fn position(&self) -> Point2<SimFloat>;
}

#[derive(Debug)]
pub enum ContinueTraverse {
    Continue,
    Stop,
}

#[repr(usize)]
enum Quadrant {
    NW = 0,
    SW = 1,
    NE = 2,
    SE = 3,
}

impl Quadrant {
    fn from_comparison(
        element_position: Point2<SimFloat>,
        node_position: Point2<SimFloat>,
    ) -> Self {
        if element_position.x < node_position.x {
            if element_position.y < node_position.y {
                Self::SW
            } else {
                Self::NW
            }
        } else {
            if element_position.y < node_position.y {
                Self::SE
            } else {
                Self::NE
            }
        }
    }

    fn apply_offset(
        &self,
        point: Point2<SimFloat>,
        extent: SimFloat,
        depth: u32,
    ) -> Point2<SimFloat> {
        let half_extent = 0.5 * extent * SimFloat::from(0.5).powi(depth.try_into().unwrap());

        match self {
            Quadrant::NW => Point2::new(point.x - half_extent, point.y + half_extent),
            Quadrant::SW => Point2::new(point.x - half_extent, point.y - half_extent),
            Quadrant::NE => Point2::new(point.x + half_extent, point.y + half_extent),
            Quadrant::SE => Point2::new(point.x + half_extent, point.y - half_extent),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) enum QuadtreeChild {
    Node(NonZeroU32),
    Element(NonZeroU32),
}

#[derive(Debug, Copy, Clone)]
pub struct QuadtreeNode<U>
where
    U: Default + Debug + Copy + Clone,
{
    pub child_index: QuadtreeChild,
    pub parent_index: NonZeroU32,
    pub data: U,
}

#[derive(Debug)]
pub(super) struct QuadtreeElement<T>
where
    T: Positioned + Debug,
{
    pub(super) element: T,
    leaf_index: Option<NonZeroU32>,
}

impl<T> QuadtreeElement<T>
where
    T: Positioned + Debug,
{
    fn new(
        element: T,
        leaf_index: Option<NonZeroU32>,
    ) -> Self {
        Self { element, leaf_index }
    }
}

#[derive(Debug)]
pub struct Quadtree<T, U>
where
    T: Positioned + Debug,
    U: Default + Debug + Copy + Clone,
{
    // the size of the root quadrants
    extent: SimFloat,
    nodes: Vec<Option<QuadtreeNode<U>>>,
    elements: Vec<QuadtreeElement<T>>,
}

impl<T, U> Quadtree<T, U>
where
    T: Positioned + Debug,
    U: Default + Debug + Copy + Clone,
{
    pub fn new(extent: SimFloat) -> Self {
        Self {
            extent,
            nodes: vec![None],
            elements: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.elements.clear();

        self.nodes.push(None);
    }

    pub fn elements(&self) -> &[QuadtreeElement<T>] {
        &self.elements
    }

    pub fn nodes(&self) -> &[Option<QuadtreeNode<U>>] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [Option<QuadtreeNode<U>>] {
        &mut self.nodes
    }

    pub fn extent(&self) -> SimFloat {
        self.extent
    }

    pub fn insert(
        &mut self,
        element: T,
    ) -> Result<(), String> {
        if element.position().x.abs() > self.extent || element.position().y.abs() > self.extent {
            panic!("Can't insert element with position {:?} into self with extent {}", element.position(), self.extent);
        }

        let element_index = self.elements.len();
        self.elements.push(QuadtreeElement::new(element, None));
        self.insert_at_node(0, 0, Point2::new(0.0, 0.0), 0, element_index)?;

        Ok(())
    }

    pub fn traverse<F>(
        &self,
        func: &mut F,
    ) -> Result<(), String>
    where
        F: FnMut(&QuadtreeNode<U>, Point2<SimFloat>, u32) -> ContinueTraverse,
    {
        self.traverse_at_node(func, 0, Point2::new(0.0, 0.0), 0)
    }

    fn traverse_at_node<F>(
        &self,
        func: &mut F,
        node_index: usize,
        node_position: Point2<SimFloat>,
        depth: u32,
    ) -> Result<(), String>
    where
        F: FnMut(&QuadtreeNode<U>, Point2<SimFloat>, u32) -> ContinueTraverse,
    {
        if depth > MAX_DEPTH {
            Err(format!("Maximum stack depth exceeded while traversing over node {:?}", self.nodes[node_index]))?;
        }

        if let Some(node) = &self.nodes[node_index] {
            match func(node, node_position, depth) {
                ContinueTraverse::Continue => {}
                ContinueTraverse::Stop => return Ok(()),
            };

            if let QuadtreeChild::Node(child_node_index) = node.child_index {
                let children_index = usize::try_from(child_node_index.get() - 1).unwrap();
                let half_extent = 0.5 * self.extent * SimFloat::from(0.5).powi(depth.try_into().unwrap());

                let child_positions = [
                    node_position + Vector2::new(-half_extent, half_extent),
                    node_position + Vector2::new(-half_extent, -half_extent),
                    node_position + Vector2::new(half_extent, half_extent),
                    node_position + Vector2::new(half_extent, -half_extent),
                ];

                for child_index in 0..4 {
                    self.traverse_at_node(func, children_index + child_index, child_positions[child_index], depth + 1)?;
                }
            }
        }

        Ok(())
    }

    fn insert_at_node(
        &mut self,
        node_index: usize,
        parent_index: usize,
        node_position: Point2<SimFloat>,
        depth: u32,
        element_index: usize,
    ) -> Result<(), String> {
        if depth > MAX_DEPTH {
            Err(format!("Maximum stack depth exceeded while inserting {:?}", self.elements[element_index]))?;
        }

        match self.nodes[node_index].as_ref().copied() {
            None => {
                // use empty slot
                let parent_index = NonZeroU32::try_from(u32::try_from(parent_index + 1).unwrap()).unwrap();
                let new_leaf = QuadtreeNode {
                    child_index: QuadtreeChild::Element(
                        NonZeroU32::try_from(u32::try_from(element_index + 1).unwrap()).unwrap(),
                    ),
                    parent_index,
                    data: U::default(),
                };
                self.nodes[node_index] = Some(new_leaf);

                let leaf_index = NonZeroU32::try_from(u32::try_from(node_index + 1).unwrap()).unwrap();
                self.elements[element_index as usize].leaf_index = Some(leaf_index);
            }
            Some(node) => {
                match node.child_index {
                    QuadtreeChild::Node(child_node_index) => {
                        // find correct quadrant and insert there
                        let element_pos = self.elements[element_index as usize].element.position();
                        let children_index = usize::try_from(child_node_index.get() - 1).unwrap();

                        let quadrant = Quadrant::from_comparison(element_pos, node_position);

                        let child_index = children_index + quadrant as usize;
                        let node_position = Quadrant::from_comparison(element_pos, node_position).apply_offset(
                            node_position,
                            self.extent,
                            depth,
                        );

                        self.insert_at_node(child_index, node_index, node_position, depth + 1, element_index)?;
                    }
                    QuadtreeChild::Element(child_element_index) => {
                        // subdivide leaf into twig and reinsert into self
                        let children_index =
                            NonZeroU32::try_from(u32::try_from(self.nodes.len() + 1).unwrap()).unwrap();
                        for _ in 0..4 {
                            self.nodes.push(None);
                        }

                        let updated_node = QuadtreeNode {
                            child_index: QuadtreeChild::Node(children_index),
                            ..self.nodes[node_index].expect("We checked this above")
                        };

                        self.nodes[node_index] = Some(updated_node);

                        self.insert_at_node(node_index, parent_index, node_position, depth, element_index)?;
                        self.insert_at_node(
                            node_index,
                            parent_index,
                            node_position,
                            depth,
                            usize::try_from(child_element_index.get() - 1).unwrap(),
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}
