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

const MAX_DEPTH: u32 = 64;

pub trait Positioned {
    fn position(&self) -> Point2<f32>;
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
        element_position: Point2<f32>,
        node_position: Point2<f32>,
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
        point: Point2<f32>,
        extent: f32,
        depth: u32,
    ) -> Point2<f32> {
        let half_extent = 0.5 * extent * 0.5f32.powi(depth.try_into().unwrap());

        match self {
            Quadrant::NW => Point2::new(point.x - half_extent, point.y + half_extent),
            Quadrant::SW => Point2::new(point.x - half_extent, point.y - half_extent),
            Quadrant::NE => Point2::new(point.x + half_extent, point.y + half_extent),
            Quadrant::SE => Point2::new(point.x + half_extent, point.y - half_extent),
        }
    }
}

#[derive(Debug)]
pub enum QuadtreeNode<U>
where
    U: Default,
{
    Twig {
        children_index: NonZeroU32,
        parent: NonZeroU32,
        data: U,
    },
    Leaf {
        element_index: NonZeroU32,
        parent_index: NonZeroU32,
    },
}

#[derive(Debug)]
struct QuadtreeElement<T>
where
    T: Positioned + Debug,
{
    element: T,
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
    U: Default + Debug,
{
    // the size of the root quadrants
    extent: f32,
    nodes: Vec<Option<QuadtreeNode<U>>>,
    elements: Vec<QuadtreeElement<T>>,
}

impl<T, U> Quadtree<T, U>
where
    T: Positioned + Debug,
    U: Default + Debug,
{
    pub fn new(extent: f32) -> Self {
        Self {
            extent,
            nodes: vec![None],
            elements: vec![],
        }
    }

    pub fn elements(&self) -> &[QuadtreeElement<T>] {
        &self.elements
    }

    pub fn nodes(&self) -> &[Option<QuadtreeNode<U>>] {
        &self.nodes
    }

    pub fn extent(&self) -> f32 {
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
        F: FnMut(&QuadtreeNode<U>, Point2<f32>, u32) -> (),
    {
        self.traverse_at_node(func, 0, Point2::new(0.0, 0.0), 0)
    }

    fn traverse_at_node<F>(
        &self,
        func: &mut F,
        node_index: usize,
        node_position: Point2<f32>,
        depth: u32,
    ) -> Result<(), String>
    where
        F: FnMut(&QuadtreeNode<U>, Point2<f32>, u32) -> (),
    {
        if depth > MAX_DEPTH {
            Err(format!("Maximum stack depth exceeded while traversing over node {:?}", self.nodes[node_index]))?;
        }

        if let Some(node) = &self.nodes[node_index] {
            func(node, node_position, depth);

            if let QuadtreeNode::Twig {
                children_index,
                parent: _,
                data: _,
            } = node
            {
                let children_index = usize::try_from(children_index.get() - 1).unwrap();
                let half_extent = 0.5 * self.extent * 0.5f32.powi(depth.try_into().unwrap());

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
        node_position: Point2<f32>,
        depth: u32,
        element_index: usize,
    ) -> Result<(), String> {
        if depth > MAX_DEPTH {
            Err(format!("Maximum stack depth exceeded while inserting {:?}", self.elements[element_index]))?;
        }

        match self.nodes[node_index] {
            None => {
                // use empty slot
                let parent_index = NonZeroU32::try_from(u32::try_from(parent_index + 1).unwrap()).unwrap();
                let new_leaf = QuadtreeNode::Leaf {
                    element_index: NonZeroU32::try_from(u32::try_from(element_index + 1).unwrap()).unwrap(),
                    parent_index,
                };
                self.nodes[node_index] = Some(new_leaf);

                let leaf_index = NonZeroU32::try_from(u32::try_from(node_index + 1).unwrap()).unwrap();
                self.elements[element_index as usize].leaf_index = Some(leaf_index);
            }
            Some(QuadtreeNode::Twig {
                children_index,
                parent: _,
                data: _,
            }) => {
                // find correct quadrant and insert there
                let element_pos = self.elements[element_index as usize].element.position();
                let children_index = usize::try_from(children_index.get() - 1).unwrap();

                let quadrant = Quadrant::from_comparison(element_pos, node_position);

                let child_index = children_index + quadrant as usize;
                let node_position = Quadrant::from_comparison(element_pos, node_position).apply_offset(
                    node_position,
                    self.extent,
                    depth,
                );

                self.insert_at_node(child_index, node_index, node_position, depth + 1, element_index)?;
            }
            Some(QuadtreeNode::Leaf {
                element_index: reinsert_index,
                parent_index,
            }) => {
                // subdivide leaf into twig and reinsert into self
                let children_index = NonZeroU32::try_from(u32::try_from(self.nodes.len() + 1).unwrap()).unwrap();
                for _ in 0..4 {
                    self.nodes.push(None);
                }

                let node_as_twig = QuadtreeNode::Twig {
                    children_index,
                    parent: NonZeroU32::try_from(node_index as u32 + 1).unwrap(),
                    data: U::default(),
                };

                self.nodes[node_index] = Some(node_as_twig);
                self.insert_at_node(
                    node_index,
                    (parent_index.get() - 1).try_into().unwrap(),
                    node_position,
                    depth,
                    element_index,
                )?;
                self.insert_at_node(
                    node_index,
                    (parent_index.get() - 1).try_into().unwrap(),
                    node_position,
                    depth,
                    usize::try_from(reinsert_index.get() - 1).unwrap(),
                )?;
            }
        }

        Ok(())
    }
}
