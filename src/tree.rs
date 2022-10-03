//! A SAH-based Bounding Volume Hierarchy

use crate::{
    bounds::BoundingBox,
    hittables::Hittable,
    utils::arena::{Arena, ArenaIndex},
};

/// The discrete element making up the [Tree]
#[derive(Debug)]
pub enum TreeNode<T> {
    /// A terminal node containing `T` values
    Leaf {
        bbox: Option<BoundingBox>,
        items: Vec<T>,
    },
    /// A node that holds the indices into its Tree's [Arena]
    Interior {
        bbox: Option<BoundingBox>,
        left: ArenaIndex,
        right: ArenaIndex,
    },
}

impl<T> TreeNode<T> {
    #[inline]
    pub fn get_bbox(&self) -> Option<BoundingBox> {
        *match self {
            TreeNode::Leaf { bbox, .. } => bbox,
            TreeNode::Interior { bbox, .. } => bbox,
        }
    }
}

/// An acceleration structure using arena allocation and the surface area hueristic (SAH) splitting method.
///
/// See [Arena] for more information on the allocator.
///
#[derive(Debug)]
pub struct Tree<T> {
    arena: Arena<TreeNode<T>>,
    root: Option<ArenaIndex>,
}

/// Holds the precomputed information of an item necessary to calculate the SAH
///
/// Contains:
/// - the item itself
/// - the item's bounding box
/// - the bounding box's centroid
#[derive(Debug, Clone, Copy)]
struct ItemInfo<T> {
    item: T,
    bbox: Option<BoundingBox>,
    centroid: Option<glam::Vec3A>,
}

/// Holds the metadata of items being binned for SAH splitting
#[derive(Debug, Default, Clone, Copy)]
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
    T: Clone + Hittable,
{
    /// Creates an empty tree
    #[inline]
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            root: None,
        }
    }

    /// Adds a new leaf node to the Tree, returning the index for use in creation and intersection
    #[inline]
    fn new_leaf(&mut self, info: &[ItemInfo<T>]) -> ArenaIndex {
        self.arena.add(TreeNode::Leaf {
            items: info.iter().map(|info| info.item.clone()).collect(),
            bbox: info
                .iter()
                .filter_map(|info| info.bbox)
                .reduce(|acc, b| acc.union(&b)),
        })
    }

    /// Returns the [BoundingBox] of the node at the given index `idx` in timeframe [time0..time1], if it has one
    #[inline]
    fn get_bbox(&self, idx: ArenaIndex, time0: f32, time1: f32) -> Option<BoundingBox> {
        self.arena[idx].get_bbox()
    }

    /// Returns the [BoundingBox] surrounding the two child nodes specified by their indices
    #[inline]
    fn compute_bbox(
        &self,
        left_idx: ArenaIndex,
        right_idx: ArenaIndex,
        time0: f32,
        time1: f32,
    ) -> Option<BoundingBox> {
        match (
            self.get_bbox(left_idx, time0, time1),
            self.get_bbox(right_idx, time0, time1),
        ) {
            (None, None) => None,
            (None, Some(r_bbox)) => Some(r_bbox),
            (Some(l_bbox), None) => Some(l_bbox),
            (Some(l_bbox), Some(r_bbox)) => Some(l_bbox.union(&r_bbox)),
        }
    }

    /// Creates a new interior node by splitting the given items into child nodes.
    ///
    /// TODO more explanation
    fn new_interior(&mut self, items: &mut [ItemInfo<T>], time0: f32, time1: f32) -> ArenaIndex {
        assert!(!items.is_empty(), "Given empty scene!");
        let num_items = items.len();

        // given few items, make leaf
        if num_items <= 4 {
            return self.new_leaf(items);
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
        const NUM_BINS: usize = 16;
        let mut bins = [Bin {
            count: 0,
            bbox: BoundingBox::default(),
        }; NUM_BINS];

        /// helper functions to correctly compute index into bins
        fn comp_bin_idx(off: f32) -> usize {
            let idx = (NUM_BINS as f32 * off) as usize;
            idx.clamp(0, NUM_BINS - 1)
        }

        // Compute bin based on how far the item's centroid
        // is from the min of the bbox of centroids
        for item in items.iter() {
            let off = centroid_bbox.offset(item.centroid.unwrap())[axis_idx];
            let bin_idx = comp_bin_idx(off);
            let bin = &mut bins[bin_idx];
            bin.count += 1;
            bin.bbox = bin.bbox.union(&item.bbox.unwrap());
        }

        // set up costs
        let mut costs = [0.0; NUM_BINS - 1];

        // Using two scans of the items, we can compute the SAH cost
        // `SurfaceArea_Left * Num_Left + SurfaceArea_Right * Num_Right`
        // by splitting on the addition, such that the left operand of
        // the addition is computed on the forward scan, and the right
        // operand using the backward scan. We reuse the [Bin] struct
        // as it holds exactly the info needed for cost computation

        let mut left_acc = Bin::default();

        // forward scan uses the first bin up to second-to-last bin
        for bin in 0..(NUM_BINS - 1) {
            left_acc.bbox = left_acc.bbox.union(&(bins[bin].bbox));
            left_acc.count += bins[bin].count;
            costs[bin] += left_acc.count as f32 * left_acc.bbox.surface_area();
        }

        let mut right_acc = Bin::default();

        // backward scan uses the last bin down to second bin
        for bin in (1..=(NUM_BINS - 1)).rev() {
            right_acc.bbox = right_acc.bbox.union(&(bins[bin].bbox));
            right_acc.count += bins[bin].count;
            costs[bin - 1] += right_acc.count as f32 * right_acc.bbox.surface_area();
        }

        // Find smallest split cost and its index into the bins array
        let (min_bin_idx, min_cost) = costs
            .iter()
            .enumerate()
            .min_by(|(_, a_cost), (_, b_cost)| a_cost.total_cmp(b_cost))
            .unwrap();

        // cost to make a node with all items is the # of items
        let leaf_cost = num_items as f32;

        // normalize cost
        let min_cost = 0.5 + min_cost / total_bbox.surface_area();

        let perc = ((min_cost - leaf_cost) / leaf_cost) * 100.0;
        eprintln!(
            "Leaf cost vs Split Cost: {} vs {} ({})",
            leaf_cost, min_cost, perc
        );

        // if its better to split, do SAH split
        if min_cost < leaf_cost {
            // init arena space before children
            let new_idx = self.arena.add(TreeNode::Interior {
                bbox: None,
                left: 0,
                right: 0,
            });

            let (left_items, right_items): (Vec<_>, Vec<_>) = items.iter().partition(|item| {
                let off = centroid_bbox.offset(item.centroid.unwrap())[axis_idx];
                let bin_idx = comp_bin_idx(off);
                bin_idx <= min_bin_idx
            });

            // partition returns a vec of refs, go back to cloned values
            let mut left_items: Vec<_> = left_items.into_iter().cloned().collect();
            let mut right_items: Vec<_> = right_items.into_iter().cloned().collect();

            let left_node = self.new_interior(&mut left_items, time0, time1);
            let right_node = self.new_interior(&mut right_items, time0, time1);

            let bbox = self.compute_bbox(left_node, right_node, time0, time1);

            self.arena[new_idx] = TreeNode::<T>::Interior {
                bbox,
                left: left_node,
                right: right_node,
            };
            new_idx
        } else {
            self.new_leaf(items)
        }
    }

    /// Creates a new Tree using the given items
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

        tree.arena.shrink_to_fit();

        // finish modifying the formerly empty tree
        tree.root = Some(root);
        tree
    }

    /// The underlying intersection routine for use in the [Hittable] trait implementation
    ///
    /// Since [Hittable::hit] doesn't use tree indices, we have to call this routine instead
    fn hit_impl(
        &self,
        idx: ArenaIndex,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<crate::hittables::HitRecord> {
        // need a private impl because we need recursion w/ indices
        let node = &self.arena[idx];

        // if there's a box, check against it first
        if let Some(bbox) = node.get_bbox() {
            if !bbox.hit(ray, t_min, t_max) {
                return None;
            }
        }

        match node {
            // a leaf node delegates to its contained item
            TreeNode::Leaf { items, .. } => {
                let mut t_closest = t_max;
                items
                    .iter()
                    .fold(None, |acc, item| match item.hit(ray, t_min, t_closest) {
                        Some(hit_rec) => {
                            t_closest = hit_rec.t;
                            Some(hit_rec)
                        }
                        None => acc,
                    })
            }
            TreeNode::Interior { left, right, .. } => {
                // recurse into children
                let left_hit = self.hit_impl(*left, ray, t_min, t_max);

                let t_max = left_hit.as_ref().map_or(t_max, |rec| rec.t);

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
        self.root
            .and_then(|root_idx| self.hit_impl(root_idx, ray, t_min, t_max))
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<BoundingBox> {
        self.root
            .and_then(|root_idx| self.get_bbox(root_idx, time0, time1))
    }
}
