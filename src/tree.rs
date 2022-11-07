//! A SAH-based Bounding Volume Hierarchy

use std::sync::Arc;

use crate::{
    bounds::BoundingBox,
    hittables::Hittable,
    utils::arena::{Arena, ArenaIndex},
};

/// The discrete element making up the [Tree]
pub enum TreeNode {
    /// A terminal node containing `T` values
    Leaf {
        bbox: Option<BoundingBox>,
        items: Vec<Arc<dyn Hittable>>,
    },
    /// A node that holds the indices into its Tree's [Arena]
    Interior {
        bbox: Option<BoundingBox>,
        left: ArenaIndex,
        right: ArenaIndex,
    },
}

impl TreeNode {
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
pub struct Tree {
    arena: Arena<TreeNode>,
    root: ArenaIndex,
}

/// Holds the precomputed information of an item necessary to calculate the SAH
///
/// Contains:
/// - the item itself
/// - the item's bounding box
/// - the bounding box's centroid
#[derive(Clone)]
struct ItemInfo {
    bbox: Option<BoundingBox>,
    centroid: Option<glam::Vec3A>,
    item: Arc<dyn Hittable>,
}

/// Holds the metadata of items being binned for SAH splitting
#[derive(Debug, Default, Clone, Copy)]
struct Bin {
    bbox: BoundingBox,
    count: usize,
}

impl std::fmt::Display for Bin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items in {:?}", self.count, self.bbox)
    }
}

impl Tree {
    /// Adds a new leaf node to the Tree, returning the index for use in creation and intersection
    #[inline]
    fn new_leaf(&mut self, info: Vec<ItemInfo>) -> ArenaIndex {
        self.arena.add(TreeNode::Leaf {
            items: info.iter().map(|info| info.item.clone()).collect(),
            bbox: info
                .iter()
                .filter_map(|info| info.bbox)
                .reduce(|acc, b| acc.union(b)),
        })
    }

    /// Returns the [BoundingBox] of the node at the given index `idx`, if it has one
    #[inline]
    fn get_bbox(&self, idx: ArenaIndex) -> Option<BoundingBox> {
        self.arena[idx].get_bbox()
    }

    /// Creates a new interior node by splitting the given items into child nodes.
    ///
    /// TODO more explanation
    fn new_interior(&mut self, mut items: Vec<ItemInfo>) -> ArenaIndex {
        let num_items = items.len();

        // given few items, make leaf
        if num_items <= 4 {
            return self.new_leaf(items);
        }

        // Get bounding_box for all item under this node
        let total_bbox = items
            .iter()
            .filter_map(|item| item.bbox)
            .reduce(|acc, b| acc.union(b));

        if let Some(total_bbox) = total_bbox {
            // If we have some bounding box, then we have some centroids
            let centroid_bbox = items
                .iter()
                .filter_map(|item| item.centroid)
                .fold(BoundingBox::default(), |bbox, centroid| {
                    bbox.add_point(centroid)
                });

            // choose axis based on lengths of the surrounding bbox
            let axis_idx = total_bbox.longest_axis();

            // set up bins
            const NUM_BINS: usize = 16;
            let mut bins = [Bin::default(); NUM_BINS];

            /// helper functions to correctly compute index into bins
            fn comp_bin_idx(off: f32) -> usize {
                let idx = (NUM_BINS as f32 * off) as usize;
                idx.clamp(0, NUM_BINS - 1)
            }

            // Compute bin based on how far the item's centroid
            // is from the min of the bbox of centroids
            for item in items.iter() {
                let off = match item.centroid {
                    Some(centroid) => centroid_bbox.offset(centroid)[axis_idx],
                    None => (NUM_BINS / 2) as f32,
                };

                let bin_idx = comp_bin_idx(off);
                let bin = &mut bins[bin_idx];
                bin.count += 1;
                // bin.bbox = bin.bbox.union(item.bbox.unwrap_or_default());
                if let Some(bbox) = item.bbox {
                    bin.bbox = bin.bbox.union(bbox);
                }
            }

            // set up costs
            let mut costs = [f32::MAX; NUM_BINS - 1];

            // Using two scans of the items, we can compute the SAH cost
            // `SurfaceArea_Left * Num_Left + SurfaceArea_Right * Num_Right`
            // by splitting on the addition, such that the left operand of
            // the addition is computed on the forward scan, and the right
            // operand using the backward scan. We reuse the [Bin] struct
            // as it holds exactly the info needed for cost computation

            let mut acc = Bin::default();
            // forward scan uses the first bin up to second-to-last bin
            for bin_idx in 0..(NUM_BINS - 1) {
                let bin = bins[bin_idx];
                if bin.count > 0 {
                    acc.bbox = acc.bbox.union(bin.bbox);
                    acc.count += bin.count;
                    costs[bin_idx] += bin.count as f32 * bin.bbox.surface_area();
                }
            }

            acc = Bin::default();
            // backward scan uses the last bin down to second bin
            for bin_idx in (1..=(NUM_BINS - 1)).rev() {
                let bin = bins[bin_idx];
                if bin.count > 0 {
                    acc.bbox = acc.bbox.union(bin.bbox);
                    acc.count += bin.count;
                    costs[bin_idx - 1] += bin.count as f32 * bin.bbox.surface_area();
                }
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

            // init arena space before children
            let new_idx = self.arena.add(TreeNode::Interior {
                bbox: None,
                left: 0,
                right: 0,
            });

            // if its better to split, do SAH split
            let (left_items, right_items) = if min_cost < leaf_cost {
                items.into_iter().partition(|item| match item.centroid {
                    Some(centroid) => {
                        let off = centroid_bbox.offset(centroid)[axis_idx];
                        let bin_idx = comp_bin_idx(off);
                        bin_idx <= min_bin_idx
                    }
                    None => true,
                })
            } else {
                // otherwise split items based on total_bbox cmp
                items.sort_by(|a, b| match (a.bbox, b.bbox) {
                    (None, None) => unreachable!(),
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (Some(a), Some(b)) => a.min[axis_idx].total_cmp(&(b.min[axis_idx])),
                });

                let halves = items.split_at(num_items / 2);
                (halves.0.to_owned(), halves.1.to_owned())
            };

            let left_idx = self.new_interior(left_items);
            let right_idx = self.new_interior(right_items);

            self.arena[new_idx] = TreeNode::Interior {
                bbox: Some(total_bbox),
                left: left_idx,
                right: right_idx,
            };
            new_idx
        } else {
            // full of unbounded objects, make leaf
            self.new_leaf(items)
        }
    }

    /// Creates a new Tree using the given items
    pub fn new(items: Vec<Arc<dyn Hittable>>, time0: f32, time1: f32) -> Self {
        debug_assert!(!items.is_empty(), "Given empty scene!");
        // TODO find way to create Tree without making an empty one first
        let mut tree = Self {
            arena: Arena::new(),
            root: 0,
        };
        // We hope for a best case full binary tree and allocate enough space
        // for such a case, minimizing allocations per-insertion
        // See [Properties of binary trees from the Binary Tree Wikipedia article](https://en.wikipedia.org/wiki/Binary_tree#Properties_of_binary_trees)
        tree.arena = Arena::with_capacity((items.len() * 2) - 1);

        // Compute info per item
        let added_info: Vec<ItemInfo> = items
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
        tree.root = tree.new_interior(added_info);
        tree.arena.shrink_to_fit();
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
            TreeNode::Leaf { items, .. } => items.hit(ray, t_min, t_max),
            TreeNode::Interior { left, right, .. } => {
                // recurse into children
                let left_hit = self.hit_impl(*left, ray, t_min, t_max);

                let t_max = left_hit.as_ref().map_or(t_max, |rec| rec.t);

                match self.hit_impl(*right, ray, t_min, t_max) {
                    Some(right_hit) => Some(right_hit),
                    None => left_hit,
                }
            }
        }
    }
}

impl Hittable for Tree {
    fn hit(
        &self,
        ray: &crate::ray::Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<crate::hittables::HitRecord> {
        self.hit_impl(self.root, ray, t_min, t_max)
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<BoundingBox> {
        self.get_bbox(self.root)
    }
}
