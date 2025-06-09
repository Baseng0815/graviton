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

use crate::{new_map_key, new_map_key_16, new_map_key_32};
use crate::utility::index_map::PrimaryMap;
use crate::utility::index_map::MapKey;

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
#[derive(Eq, PartialEq, Clone, Copy)]
enum Quadrant {
    NE = 0b00,
    NW = 0b01,
    SE = 0b10,
    SW = 0b11,
}

/* NW (0b01) | NE (0b00)
 * ----------+----------
 * SW (0b11) | SE (0b10)
 */

impl TryFrom<u32> for Quadrant {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(match value {
            0b00 => {
                Self::NE
            },
            0b01 => {
                Self::NW
            },
            0b10 => {
                Self::SE
            },
            0b11 => {
                Self::SW
            }
            _ => Err(())?
        })
    }
}

impl Quadrant {
    fn from_comparison(
        node_position: Point2<SimFloat>,
        element_position: Point2<SimFloat>,
    ) -> Self {
        let cmp_x = u32::from(element_position.x < node_position.x);
        let cmp_y = u32::from(element_position.y < node_position.y);

        Self::try_from((cmp_x << 0) | (cmp_y << 1)).unwrap()
    }

    fn apply_offset(
        &self,
        position: Point2<SimFloat>,
        extent: SimFloat,
    ) -> Point2<SimFloat> {
        let half_extent = 0.5 * extent;

        match self {
            Quadrant::NW => Point2::new(position.x - half_extent, position.y + half_extent),
            Quadrant::SW => Point2::new(position.x - half_extent, position.y - half_extent),
            Quadrant::NE => Point2::new(position.x + half_extent, position.y + half_extent),
            Quadrant::SE => Point2::new(position.x + half_extent, position.y - half_extent),
        }
    }
}

new_map_key_32! { pub struct NodeKey; "NODE"; }
new_map_key_32! { pub struct ElementKey; "NODE"; }

#[derive(Debug, Copy, Clone)]
pub(super) enum QuadtreeChild {
    Node(NodeKey),
    Element(ElementKey),
}

#[derive(Debug, Copy, Clone)]
pub struct QuadtreeNode<U>
where
    U: Default + Debug + Copy + Clone,
{
    pub child_key: QuadtreeChild,
    pub position: Point2<SimFloat>,
    pub extent: SimFloat,
    pub data: U,
}

#[derive(Debug)]
pub struct Quadtree<T, U>
where
    T: Positioned + Debug,
    U: Default + Debug + Copy + Clone,
{
    // the size of the root quadrants
    extent: SimFloat,
    nodes: PrimaryMap<NodeKey, Option<QuadtreeNode<U>>>,
    elements: PrimaryMap<ElementKey, T>,
}

impl<T, U> Quadtree<T, U>
where
    T: Positioned + Debug,
    U: Default + Debug + Copy + Clone,
{
    pub fn new(extent: SimFloat) -> Self {
        let mut slf = Self {
            extent,
            nodes: Default::default(),
            elements: Default::default(),
        };

        slf.nodes.insert(None);
        slf
    }

    pub fn clear(&mut self) {
        self.nodes = Default::default();
        self.elements = Default::default();

        self.nodes.insert(None);
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

        // insert new element
        let element_key = self.elements.insert(element);

        // find existing leaf quadrant the element belongs to
        let mut leaf_node_key = self.nodes.keys().next().expect("A root must exist");
        let mut position = Point2::new(0.0, 0.0);
        let mut extent = self.extent;

        while let Some(QuadtreeNode { child_key: QuadtreeChild::Node(children), .. }) = self.nodes[leaf_node_key] {
            let quadrant = Quadrant::from_comparison(position, self.elements[element_key].position());
            let child_index = quadrant as usize;
            leaf_node_key = NodeKey::try_from_index(children.to_index() + child_index).unwrap();

            position = quadrant.apply_offset(position, extent);
            extent *= 0.5;
        }

        match self.nodes[leaf_node_key] {
            None => {
                // empty leaf => insert directly
                let new_leaf = QuadtreeNode {
                    child_key: QuadtreeChild::Element(element_key),
                    position,
                    extent,
                    data: U::default(),
                };
                self.nodes[leaf_node_key] = Some(new_leaf);
            }
            Some(existing) => {
                // non-empty leaf => split until quadrants are different
                let QuadtreeChild::Element(existing_element_key) = existing.child_key else {
                    panic!("We checked for this above");
                };

                // convert leaf to empty twig
                let children_index = self.nodes.next_key();
                for _ in 0..4 {
                    self.nodes.insert(None);
                }

                self.nodes[leaf_node_key].as_mut().unwrap().child_key = QuadtreeChild::Node(children_index);

                loop {
                    let q_0 = Quadrant::from_comparison(position, self.elements[existing_element_key].position());
                    let q_1 = Quadrant::from_comparison(position, self.elements[element_key].position());

                    if q_0 == q_1 {
                        position = q_0.apply_offset(position, extent);
                        extent *= 0.5;

                        // same quadrants => split further by creating new twig node at the child's position
                        let QuadtreeChild::Node(children_key) = self.nodes[leaf_node_key].unwrap().child_key else {
                            panic!("We converted the parent to a twig");
                        };

                        let child_key = NodeKey::try_from_index(children_key.to_index() + q_0 as usize).unwrap();
                        let new_children_key = self.nodes.next_key();
                        for _ in 0..4 {
                            self.nodes.insert(None);
                        }

                        self.nodes[child_key] = Some(QuadtreeNode {
                            child_key: QuadtreeChild::Node(new_children_key), position, extent, data: Default::default(),
                        });

                        leaf_node_key = child_key;
                    } else {
                        // different quadrants => insert elements and finish
                        let position_0 = q_0.apply_offset(position, extent);
                        let position_1 = q_1.apply_offset(position, extent);
                        extent *= 0.5;

                        let QuadtreeChild::Node(children_key) = self.nodes[leaf_node_key].unwrap().child_key else {
                            panic!("We converted the parent to a twig");
                        };

                        let child_key_0 = NodeKey::try_from_index(children_key.to_index() + q_0 as usize).unwrap();
                        let child_key_1 = NodeKey::try_from_index(children_key.to_index() + q_1 as usize).unwrap();

                        self.nodes[child_key_0] = Some(QuadtreeNode {
                            child_key: QuadtreeChild::Element(existing_element_key),
                            position: position_0,
                            extent,
                            data: Default::default(),
                        });

                        self.nodes[child_key_1] = Some(QuadtreeNode {
                            child_key: QuadtreeChild::Element(element_key),
                            position: position_1,
                            extent,
                            data: Default::default(),
                        });

                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn nodes(&self) -> &PrimaryMap<NodeKey, Option<QuadtreeNode<U>>> {
        &self.nodes
    }
}
