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
    centroid: Option<glam::Vec3A>,
}

#[derive(Debug, Clone, Copy)]
struct Bin {
    count: usize,
    bbox: BoundingBox,
}

impl std::fmt::Display for Bin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items in {:?}", self.count, self.bbox)
    }
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
        match self.arena.get(idx) {
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

        if num_items == 2 {
            let left = Some(self.new_leaf(items[0].clone()));
            let right = Some(self.new_leaf(items[1].clone()));
            let bbox = self.compute_bbox(left, right, time0, time1);
            return self.arena.add(TreeNode::Interior { bbox, left, right });
        }

        // Get bounding_box for all item under this node
        // as well as the bbox for all items' centroids
        let (total_bbox, centroid_bbox) = items
            .iter()
            // For each item, take the bbox and centroid Options -> (bbox, centroid) tuples
            .filter_map(|item| match (item.bbox, item.centroid) {
                (Some(bbox), Some(centroid)) => Some((bbox, centroid)),
                _ => None,
            })
            // reduce the tuples into two bboxes
            .fold(
                (BoundingBox::default(), BoundingBox::default()),
                // Destructs the tuples for init and current for readability
                |(total_bbox, centroid_bbox), (bbox, centroid)| {
                    (total_bbox.union(&bbox), centroid_bbox.add_point(centroid))
                },
            );

        // choose axis based on lengths of the surrounding bbox
        let axis_idx = total_bbox.longest_axis();

        // set up bins
        const NUM_BINS: usize = 8;
        let mut bins = [Bin {
            count: 0,
            bbox: BoundingBox::default(),
        }; NUM_BINS];

        fn comp_bin_idx(off: f32) -> usize {
            let idx = (NUM_BINS as f32 * off) as usize;
            idx.clamp(0, NUM_BINS - 1)
        }

        // compute bin info
        items.iter().for_each(|item| {
            // Compute which bin based on how far the item's centroid
            // is the start of the centroid bbox
            let off = centroid_bbox.offset(item.centroid.unwrap())[axis_idx];
            let bin_idx = comp_bin_idx(off);
            let bin = &mut bins[bin_idx];
            bin.count += 1;
            bin.bbox = bin.bbox.union(&item.bbox.unwrap());
        });

        // set up costs
        let mut costs = [0.0; NUM_BINS - 1];

        // Using two scans of the items, we can compute the SAH cost
        // SurfArea_Left * Num_Left + SurfArea_Right * Num_Right
        // by splitting on the addition, such that the left operand of
        // the addition is computed on the forward scan, and the right
        // operand using the backward scan.

        let mut left_bin_acc = Bin {
            count: 0,
            bbox: BoundingBox::default(),
        };
        for bin in 0..(NUM_BINS - 1) {
            left_bin_acc.bbox = left_bin_acc.bbox.union(&(bins[bin].bbox));
            left_bin_acc.count += bins[bin].count;
            costs[bin] += left_bin_acc.count as f32 * left_bin_acc.bbox.surface_area();
        }

        let mut right_bin_acc = Bin {
            count: 0,
            bbox: BoundingBox::default(),
        };
        for bin in (1..=(NUM_BINS - 1)).rev() {
            right_bin_acc.bbox = right_bin_acc.bbox.union(&(bins[bin].bbox));
            right_bin_acc.count += bins[bin].count;
            costs[bin - 1] += right_bin_acc.count as f32 * right_bin_acc.bbox.surface_area();
        }

        costs
            .iter_mut()
            .for_each(|cost| *cost /= total_bbox.surface_area());

        // Find smallest split cost and its index into the bins array
        let (min_bin_idx, min_cost) = costs
            .iter()
            .enumerate()
            .min_by(|(_, a_cost), (_, b_cost)| a_cost.total_cmp(b_cost))
            .unwrap();

        let leaf_cost = num_items as f32;
        let min_cost = 0.5 + min_cost/*  / total_bbox.surface_area() */;

        // if its better to split, do SAH split
        let (mut left_items, mut right_items) = if min_cost < leaf_cost {
            let (left_items, right_items): (Vec<_>, Vec<_>) = items.iter().partition(|item| {
                let off = centroid_bbox.offset(item.centroid.unwrap())[axis_idx];
                let bin_idx = comp_bin_idx(off);
                bin_idx <= min_bin_idx
            });

            // partition returns a vec of refs, go back to cloned values
            let left_items: Vec<_> = left_items.into_iter().cloned().collect();
            let right_items: Vec<_> = right_items.into_iter().cloned().collect();

            (left_items, right_items)
        } else {
            items.sort_by(|a, b| crate::bvh::box_cmp(&a.bbox, &b.bbox, axis_idx));

            let (left_items, right_items) = items.split_at_mut(num_items / 2);

            // if we take the slice ref now, the ref won't live long enough,
            // so capture values into a Vec then re-slice later :/
            let left_items: Vec<_> = left_items.to_vec();
            let right_items: Vec<_> = right_items.to_vec();

            (left_items, right_items)
        };

        let left_node = self.new_interior(&mut left_items, time0, time1);
        let right_node = self.new_interior(&mut right_items, time0, time1);

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
        // We hope for a best case full binary tree and allocate enough space
        // for such a case, minimizing allocations per-insertion
        // See [Properties of binary trees from the Binary Tree Wikipedia article](https://en.wikipedia.org/wiki/Binary_tree#Properties_of_binary_trees)
        tree.arena = Arena::with_capacity((items.len() * 2) - 1);

        // Compute info per item
        let mut added_info: Vec<ItemInfo<T>> = items
            .into_iter()
            .map(|item| {
                let bbox = item.bounding_box(time0, time1);
                let centroid = bbox.map(|bbox| bbox.centroid());
                ItemInfo {
                    item,
                    bbox,
                    centroid,
                }
            })
            .collect();

        // create tree and get root index
        let root = tree.new_interior(&mut added_info, time0, time1);

        // finish modifying the formerly empty tree
        tree.root = Some(root);
        tree
    }

    fn hit_impl(
        &self,
        idx: ArenaIndex,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<crate::hittables::HitRecord> {
        // need a private impl because we need recursion w/ indices
        let node = self.arena.get(idx);
        match node {
            Some(node) => match node {
                // a leaf node delegates to its contained item
                TreeNode::Leaf(item) => item.hit(ray, t_min, t_max),
                TreeNode::Interior { bbox, left, right } => {
                    // if there's a box, check against it first
                    if let Some(bbox) = bbox {
                        if !bbox.hit(ray, t_min, t_max) {
                            return None;
                        }
                    }

                    // recurse into children
                    match (left, right) {
                        // no children, no intersection
                        (None, None) => None,
                        // one child uses the child's hit result
                        (None, Some(right)) => self.hit_impl(*right, ray, t_min, t_max),
                        (Some(left), None) => self.hit_impl(*left, ray, t_min, t_max),
                        // use the closer of the two children's hit results
                        (Some(left), Some(right)) => {
                            let left_hit = self.hit_impl(*left, ray, t_min, t_max);

                            let t_max = match &left_hit {
                                Some(rec) => rec.t,
                                None => t_max,
                            };

                            let right_hit = self.hit_impl(*right, ray, t_min, t_max);
                            match (left_hit, right_hit) {
                                (None, None) => None,
                                (None, Some(r_rec)) => Some(r_rec),
                                (Some(l_rec), None) => Some(l_rec),
                                (Some(l_rec), Some(r_rec)) => {
                                    if l_rec.t < r_rec.t {
                                        Some(l_rec)
                                    } else {
                                        Some(r_rec)
                                    }
                                }
                            }
                        }
                    }
                }
            },
            // no node, no intersection
            None => None,
        }
    }
}

impl<T> Hittable for Tree<T>
where
    T: Clone + Hittable + Sized,
{
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<crate::hittables::HitRecord> {
        match self.root {
            Some(root_idx) => self.hit_impl(root_idx, ray, t_min, t_max),
            None => None,
        }
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<BoundingBox> {
        match self.root {
            Some(root_idx) => self.get_bbox(root_idx, time0, time1),
            None => None,
        }
    }
}
