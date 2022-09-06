use crate::{
    bounds::BoundingBox,
    hittables::Hittable,
    utils::arena::{Arena, ArenaIndex},
};

#[derive(Debug)]
pub enum TreeNode<T> {
    Leaf(T),
    Interior {
        bbox: Option<BoundingBox>,
        left: Option<ArenaIndex>,
        right: Option<ArenaIndex>,
    },
}

#[derive(Debug)]
pub struct Tree<T> {
    arena: Arena<TreeNode<T>>,
    root: Option<ArenaIndex>,
}

#[derive(Debug)]
struct ItemInfo<T> {
    item: T,
    bbox: Option<BoundingBox>,
}

impl<T> Tree<T>
where
    T: Clone + Hittable + Sized,
{
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            root: None,
        }
    }

    fn new_leaf(&mut self, item: T) -> ArenaIndex {
        self.arena.add(TreeNode::Leaf(item))
    }

    fn get_bbox(&self, idx: ArenaIndex, time0: f32, time1: f32) -> Option<BoundingBox> {
        match &self.arena[idx] {
            Some(node) => match node {
                TreeNode::Leaf(item) => item.bounding_box(time0, time1),
                TreeNode::Interior { bbox, .. } => *bbox,
            },
            None => None,
        }
    }

    fn set_bbox(&self, node: &mut TreeNode<T>, time0: f32, time1: f32) {
        // only interior nodes need to set their bboxes
        if let TreeNode::Interior { bbox, left, right } = node {
            if bbox.is_none() {
                *bbox = match (left, right) {
                    // no children
                    (None, None) => None,
                    // use box of only child
                    (None, Some(r_idx)) => self.get_bbox(*r_idx, time0, time1),
                    (Some(l_idx), None) => self.get_bbox(*l_idx, time0, time1),
                    // combine boxes of both children
                    (Some(l_idx), Some(r_idx)) => {
                        match (
                            self.get_bbox(*l_idx, time0, time1),
                            self.get_bbox(*r_idx, time0, time1),
                        ) {
                            (None, None) => None,
                            (None, Some(r_bbox)) => Some(r_bbox),
                            (Some(l_bbox), None) => Some(l_bbox),
                            (Some(l_bbox), Some(r_bbox)) => Some(l_bbox.union(&r_bbox)),
                        }
                    }
                }
            }
        }
    }

    fn new_interior(&mut self, items: &mut [T]) -> ArenaIndex {
        let time0 = 0.0;
        let time1 = 1.0;
        assert!(!items.is_empty(), "Given empty scene!");
        let num_items = items.len();

        if num_items == 1 {
            return self.new_leaf(items[0].clone());
        }

        let axis_idx = 0; // temporary
        items.sort_by(|a, b| {
            crate::bvh::box_cmp(
                &a.bounding_box(time0, time1),
                &b.bounding_box(time0, time1),
                axis_idx,
            )
        });

        let (left_items, right_items): (&mut [T], &mut [T]) = items.split_at_mut(num_items / 2);
        let left_node = self.new_interior(left_items);
        let right_node = self.new_interior(right_items);
        let mut node = TreeNode::<T>::Interior {
            bbox: None,
            left: Some(left_node),
            right: Some(right_node),
        };

        self.set_bbox(&mut node, time0, time1);
        self.arena.add(node)
    }

    pub fn with_items(items: &mut [T]) -> Self {
        let mut tree = Self::new();
        tree.arena = Arena::with_capacity((items.len() * 2) - 1);
        let root = tree.new_interior(items);
        tree.root = Some(root);
        tree
    }
}
