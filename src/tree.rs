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

#[derive(Debug, Clone)]
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

    fn new_leaf(&mut self, info: ItemInfo<T>) -> ArenaIndex {
        self.arena.add(TreeNode::Leaf(info.item))
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

    fn compute_bbox(
        &self,
        left_idx: Option<usize>,
        right_idx: Option<usize>,
        time0: f32,
        time1: f32,
    ) -> Option<BoundingBox> {
        match (left_idx, right_idx) {
            // no children
            (None, None) => None,
            // use box of only child
            (None, Some(r_idx)) => self.get_bbox(r_idx, time0, time1),
            (Some(l_idx), None) => self.get_bbox(l_idx, time0, time1),
            // combine boxes of both children
            (Some(l_idx), Some(r_idx)) => {
                match (
                    self.get_bbox(l_idx, time0, time1),
                    self.get_bbox(r_idx, time0, time1),
                ) {
                    (None, None) => None,
                    (None, Some(r_bbox)) => Some(r_bbox),
                    (Some(l_bbox), None) => Some(l_bbox),
                    (Some(l_bbox), Some(r_bbox)) => Some(l_bbox.union(&r_bbox)),
                }
            }
        }
    }

    fn new_interior(&mut self, items: &mut [ItemInfo<T>], time0: f32, time1: f32) -> ArenaIndex {
        assert!(!items.is_empty(), "Given empty scene!");
        let num_items = items.len();

        if num_items == 1 {
            return self.new_leaf(items[0].clone());
        }

        let axis_idx = 0; // temporary
        items.sort_by(|a, b| crate::bvh::box_cmp(&a.bbox, &b.bbox, axis_idx));

        let (left_items, right_items) = items.split_at_mut(num_items / 2);
        let left_node = self.new_interior(left_items, time0, time1);
        let right_node = self.new_interior(right_items, time0, time1);

        let bbox = self.compute_bbox(Some(left_node), Some(right_node), time0, time1);

        self.arena.add(TreeNode::<T>::Interior {
            bbox,
            left: Some(left_node),
            right: Some(right_node),
        })
    }

    pub fn with_items(items: Vec<T>, time0: f32, time1: f32) -> Self {
        // TODO find way to create Tree without making an empty one first
        let mut tree = Self::new();
        tree.arena = Arena::with_capacity((items.len() * 2) - 1);

        // Compute info per item
        let mut added_info: Vec<ItemInfo<T>> = items
            .into_iter()
            .map(|item| {
                let bbox = item.bounding_box(time0, time1);
                ItemInfo { item, bbox }
            })
            .collect();

        // create tree and get root index
        let root = tree.new_interior(&mut added_info, time0, time1);

        // finish modifying the formerly empty tree
        tree.root = Some(root);
        tree
    }
}
