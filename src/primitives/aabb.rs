use super::geometry::Rectangle;

#[derive(Debug)]
pub struct StaticAABBTree<const CAP: usize> {
    nodes: [Node; CAP],
    root: u16,
    free_head: u16,
}

impl<const CAP: usize> core::fmt::Display for StaticAABBTree<CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.root == NULL_NODE {
            return writeln!(f, "<Empty Tree>");
        }

        // Recursive helper to format lines using a bitmask for indentation
        fn print_node<const C: usize>(
            tracker: &StaticAABBTree<C>,
            node_idx: u16,
            f: &mut core::fmt::Formatter<'_>,
            depth: usize,
            is_last: bool,
            path_mask: u64,
        ) -> core::fmt::Result {
            if node_idx == NULL_NODE {
                return Ok(());
            }

            let node = &tracker.nodes[node_idx as usize];
            let is_leaf = tracker.is_leaf(node_idx);

            // Print the tree structure branches based on depth and ancestry mask
            if depth > 0 {
                for i in 0..(depth - 1) {
                    // Check if the ancestor at this depth level requires a continuation line
                    if (path_mask & (1_u64.wrapping_shl(i as u32))) != 0 {
                        write!(f, "│   ")?;
                    } else {
                        write!(f, "    ")?;
                    }
                }

                if is_last {
                    write!(f, "└── ")?;
                } else {
                    write!(f, "├── ")?;
                }
            }

            let label = if depth == 0 {
                "Root"
            } else if is_leaf {
                "Leaf"
            } else {
                "Node"
            };

            writeln!(f, "{} {}", label, node.rect)?;

            // Recurse for children, updating the path bitmask
            if !is_leaf {
                // Left child is never the last child. Ancestors below will need a '|' line.
                let left_mask = path_mask | 1_u64.wrapping_shl(depth as u32);
                print_node(tracker, node.left, f, depth + 1, false, left_mask)?;

                // Right child is the last child. Ancestors below won't need a '|' line.
                let right_mask = path_mask & !1_u64.wrapping_shl(depth as u32);
                print_node(tracker, node.right, f, depth + 1, true, right_mask)?;
            }

            Ok(())
        }

        print_node(self, self.root, f, 0, true, 0)
    }
}

const NULL_NODE: u16 = u16::MAX;

#[derive(Clone, Debug)]
struct Node {
    rect: Rectangle,
    left: u16,
    right: u16,
    parent: u16,
}

impl Node {
    const EMPTY: Self = Self {
        rect: Rectangle::from_bounds(0, 0, 0, 0),
        left: NULL_NODE,
        right: NULL_NODE,
        parent: NULL_NODE,
    };
}

impl<const CAP: usize> StaticAABBTree<CAP> {
    /// Creates a new damage tracker with an initialized internal free-list.
    #[must_use] 
    pub const fn new() -> Self {
        let mut nodes = [Node::EMPTY; CAP];
        let mut free_head = NULL_NODE;

        if CAP > 0 {
            free_head = 0;
            let mut i = 0;
            while i < CAP - 1 {
                nodes[i].left = (i + 1) as u16;
                i += 1;
            }
            nodes[CAP - 1].left = NULL_NODE;
        }

        Self {
            nodes,
            root: NULL_NODE,
            free_head,
        }
    }

    /// Inserts a new rectangle. If capacity is hit, it defensively unions with the
    /// closest leaf to avoid allocation failure.
    pub fn insert(&mut self, rect: Rectangle) {
        if self.root == NULL_NODE {
            if let Some(node_idx) = self.allocate() {
                self.nodes[node_idx as usize] = Node {
                    rect,
                    left: NULL_NODE,
                    right: NULL_NODE,
                    parent: NULL_NODE,
                };
                self.root = node_idx;
            }
            return;
        }

        // 1. Find best leaf via minimal area expansion
        let mut curr = self.root;
        while !self.is_leaf(curr) {
            let left = self.nodes[curr as usize].left;
            let right = self.nodes[curr as usize].right;

            let area_left = self.nodes[left as usize].rect.area();
            let area_right = self.nodes[right as usize].rect.area();

            let cost_left = self.nodes[left as usize].rect.union(&rect).area() - area_left;
            let cost_right = self.nodes[right as usize].rect.union(&rect).area() - area_right;

            if cost_left < cost_right {
                curr = left;
            } else if cost_right < cost_left {
                curr = right;
            } else {
                curr = if area_left < area_right { left } else { right };
            }
        }

        // 2. Expand hierarchy if we have capacity, otherwise fallback to union
        let new_leaf = self.allocate();
        let new_internal = self.allocate();

        if let (Some(leaf_idx), Some(internal_idx)) = (new_leaf, new_internal) {
            let old_parent = self.nodes[curr as usize].parent;
            let old_rect = self.nodes[curr as usize].rect.clone();

            self.nodes[leaf_idx as usize] = Node {
                rect: rect.clone(),
                left: NULL_NODE,
                right: NULL_NODE,
                parent: internal_idx,
            };

            self.nodes[curr as usize].parent = internal_idx;

            self.nodes[internal_idx as usize] = Node {
                rect: old_rect.union(&rect),
                left: curr,
                right: leaf_idx,
                parent: old_parent,
            };

            if old_parent == NULL_NODE {
                self.root = internal_idx;
            } else {
                let parent_node = &mut self.nodes[old_parent as usize];
                if parent_node.left == curr {
                    parent_node.left = internal_idx;
                } else {
                    parent_node.right = internal_idx;
                }
            }

            self.fix_upwards(internal_idx);
        } else {
            // Return nodes to free list if partially allocated
            if let Some(idx) = new_leaf { self.free(idx); }
            if let Some(idx) = new_internal { self.free(idx); }

            // Capacity Resistant Edge-case: Force a union on the leaf.
            self.nodes[curr as usize].rect = self.nodes[curr as usize].rect.union(&rect);
            self.fix_upwards(curr);
        }
    }

    /// Queries rectangles intersecting the target and calls the visitor function
    pub fn query_intersects<F: FnMut(&Rectangle)>(&self, target: &Rectangle, mut visitor: F) {
        if self.root == NULL_NODE { return; }

        let mut stack = [NULL_NODE; 64];
        let mut top = 0;
        stack[top] = self.root;
        top += 1;

        while top > 0 {
            top -= 1;
            let curr = stack[top];
            let node = &self.nodes[curr as usize];

            if node.rect.intersects(target) {
                if self.is_leaf(curr) {
                    visitor(&node.rect);
                } else if top + 2 <= 64 {
                    stack[top] = node.left; top += 1;
                    stack[top] = node.right; top += 1;
                }
            }
        }
    }

    /// Recursively drains (deletes) all rectangles fully contained within `target`
    /// calling `visitor` on the exact rectangles deleted.
    pub fn drain_contained<F: FnMut(Rectangle)>(&mut self, target: &Rectangle, mut visitor: F) {
        while let Some(leaf_idx) = self.find_first_match(target, true) {
            let rect = self.nodes[leaf_idx as usize].rect.clone();
            self.remove_leaf(leaf_idx);
            visitor(rect);
        }
    }

    /// Recursively drains (deletes) all rectangles intersecting with `target`
    /// calling `visitor` on the exact rectangles deleted.
    pub fn drain_intersects<F: FnMut(Rectangle)>(&mut self, target: &Rectangle, mut visitor: F) {
        while let Some(leaf_idx) = self.find_first_match(target, false) {
            let rect = self.nodes[leaf_idx as usize].rect.clone();
            self.remove_leaf(leaf_idx);
            visitor(rect);
        }
    }

    // --- Private Helper Methods ---

    fn is_leaf(&self, idx: u16) -> bool {
        let node = &self.nodes[idx as usize];
        node.left == NULL_NODE && node.right == NULL_NODE
    }

    fn allocate(&mut self) -> Option<u16> {
        if self.free_head == NULL_NODE {
            None
        } else {
            let idx = self.free_head;
            self.free_head = self.nodes[idx as usize].left;
            Some(idx)
        }
    }

    fn free(&mut self, idx: u16) {
        self.nodes[idx as usize].left = self.free_head;
        self.free_head = idx;
    }

    fn fix_upwards(&mut self, mut node_idx: u16) {
        while node_idx != NULL_NODE {
            let left = self.nodes[node_idx as usize].left;
            let right = self.nodes[node_idx as usize].right;

            if left != NULL_NODE && right != NULL_NODE {
                let rect = self.nodes[left as usize].rect.union(&self.nodes[right as usize].rect);
                self.nodes[node_idx as usize].rect = rect;
            }
            node_idx = self.nodes[node_idx as usize].parent;
        }
    }

    fn find_first_match(&self, target: &Rectangle, require_containment: bool) -> Option<u16> {
        if self.root == NULL_NODE { return None; }

        let mut stack = [NULL_NODE; 64];
        let mut top = 0;
        stack[top] = self.root;
        top += 1;

        while top > 0 {
            top -= 1;
            let curr = stack[top];
            let node = &self.nodes[curr as usize];

            if node.rect.intersects(target) {
                if self.is_leaf(curr) {
                    if !require_containment || target.contains_rect(&node.rect) {
                        return Some(curr);
                    }
                } else if top + 2 <= 64 {
                    stack[top] = node.left; top += 1;
                    stack[top] = node.right; top += 1;
                }
            }
        }
        None
    }

    fn remove_leaf(&mut self, leaf_idx: u16) {
        let parent_idx = self.nodes[leaf_idx as usize].parent;
        self.free(leaf_idx);

        if parent_idx == NULL_NODE {
            self.root = NULL_NODE;
        } else {
            let parent = self.nodes[parent_idx as usize].clone();
            let sibling_idx = if parent.left == leaf_idx { parent.right } else { parent.left };
            let grandparent_idx = parent.parent;

            self.nodes[sibling_idx as usize].parent = grandparent_idx;

            if grandparent_idx == NULL_NODE {
                self.root = sibling_idx;
            } else {
                let grandparent = &mut self.nodes[grandparent_idx as usize];
                if grandparent.left == parent_idx {
                    grandparent.left = sibling_idx;
                } else {
                    grandparent.right = sibling_idx;
                }
            }

            self.free(parent_idx);
            self.fix_upwards(grandparent_idx);
        }
    }
}

impl<const CAP: usize> Default for StaticAABBTree<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization_and_insertion() {
        let mut tree = StaticAABBTree::<10>::new(); // Capacity of 10 nodes
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);

        // A single insertion takes 1 node
        tree.insert(Rectangle::from_bounds(0, 0, 10, 10)).unwrap();
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());

        // Second insertion creates an internal node + leaf, using 2 more nodes
        tree.insert(Rectangle::from_bounds(15, 15, 20, 20)).unwrap();
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn test_query_intersects() {
        let mut tree = StaticAABBTree::<20>::new();

        let r1 = Rectangle::from_bounds(0, 0, 10, 10);
        let r2 = Rectangle::from_bounds(5, 5, 15, 15);
        let r3 = Rectangle::from_bounds(20, 20, 30, 30);

        tree.insert(r1.clone()).unwrap();
        tree.insert(r2.clone()).unwrap();
        tree.insert(r3.clone()).unwrap();

        let search_area = Rectangle::from_bounds(8, 8, 12, 12);
        let mut match_count = 0;

        tree.query_intersects(&search_area, |rect| {
            // Should match r1 and r2, but not r3
            assert!(*rect == r1 || *rect == r2);
            assert!(*rect != r3);
            match_count += 1;
        });

        assert_eq!(match_count, 2);
    }

    #[test]
    fn test_query_contains() {
        let mut tree = StaticAABBTree::<20>::new();

        let small_rect = Rectangle::from_bounds(2, 2, 8, 8);
        let overlapping_rect = Rectangle::from_bounds(5, 5, 15, 15);
        let large_rect = Rectangle::from_bounds(0, 0, 20, 20);

        tree.insert(small_rect.clone()).unwrap();
        tree.insert(overlapping_rect.clone()).unwrap();
        tree.insert(large_rect.clone()).unwrap();

        let search_area = Rectangle::from_bounds(0, 0, 10, 10);
        let mut match_count = 0;

        tree.query_contains(&search_area, |rect| {
            // Only small_rect is STRICTLY contained inside search_area
            assert_eq!(*rect, small_rect);
            match_count += 1;
        });

        assert_eq!(match_count, 1);
    }

    #[test]
    fn test_removal() {
        let mut tree = StaticAABBTree::<20>::new();

        let r1 = Rectangle::from_bounds(0, 0, 5, 5);
        let r2 = Rectangle::from_bounds(10, 10, 15, 15);

        tree.insert(r1.clone()).unwrap();
        tree.insert(r2.clone()).unwrap();
        assert_eq!(tree.len(), 2);

        // Remove r1
        let was_removed = tree.remove(&r1);
        assert!(was_removed);
        assert_eq!(tree.len(), 1);

        // Verify r1 is gone
        let mut found_r1 = false;
        tree.query_intersects(&r1, |_| {
            found_r1 = true;
        });
        assert!(!found_r1);

        // Verify r2 is still there
        let mut found_r2 = false;
        tree.query_intersects(&r2, |_| {
            found_r2 = true;
        });
        assert!(found_r2);

        // Try removing something that isn't there
        let not_in_tree = Rectangle::from_bounds(100, 100, 105, 105);
        assert!(!tree.remove(&not_in_tree));
    }

    #[test]
    fn test_capacity_limit() {
        // Capacity of 3 nodes allows for exactly 2 insertions.
        // Insertion 1 uses 1 node (root leaf).
        // Insertion 2 uses 2 nodes (1 internal, 1 leaf).
        let mut tree = StaticAABBTree::<3>::new();

        assert!(tree.insert(Rectangle::from_bounds(0, 0, 1, 1)).is_ok());
        assert!(tree.insert(Rectangle::from_bounds(2, 2, 3, 3)).is_ok());

        // The third insertion should fail because it requires 2 more nodes
        let result = tree.insert(Rectangle::from_bounds(4, 4, 5, 5));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tree is at maximum capacity");
    }

    #[test]
    fn test_capacity_reuse_after_removal() {
        // A capacity of 5 nodes allows exactly 3 rectangles to be inserted:
        // - 1st insert: 1 node (becomes root leaf)
        // - 2nd insert: 2 nodes (1 new leaf + 1 new internal parent)
        // - 3rd insert: 2 nodes (1 new leaf + 1 new internal parent)
        // Total nodes used: 5.
        let mut tree = StaticAABBTree::<5>::new();

        let r1 = Rectangle::from_bounds(0, 0, 1, 1);
        let r2 = Rectangle::from_bounds(2, 2, 3, 3);
        let r3 = Rectangle::from_bounds(4, 4, 5, 5);
        let r4_overflow = Rectangle::from_bounds(6, 6, 7, 7);

        // 1. Add nodes until the tree is full
        assert!(tree.insert(r1.clone()).is_ok());
        assert!(tree.insert(r2.clone()).is_ok());
        assert!(tree.insert(r3.clone()).is_ok());
        assert_eq!(tree.len(), 3);

        // Validate the tree is actually full
        assert!(tree.insert(r4_overflow.clone()).is_err());

        // 2. Delete a node (this should free exactly 2 nodes in the underlying array)
        assert!(tree.remove(&r2));
        assert_eq!(tree.len(), 2);

        // 3. Add a new node, which should successfully claim the newly freed space
        let r5_new = Rectangle::from_bounds(8, 8, 9, 9);
        assert!(
            tree.insert(r5_new.clone()).is_ok(),
            "Failed to reuse freed capacity!"
        );
        assert_eq!(tree.len(), 3);

        // Validate it is full once again
        assert!(tree.insert(r4_overflow).is_err());

        // 4. Final sanity check: ensure the tree structure remains valid and
        // contains exactly the expected rectangles (r1, r3, r5_new).
        let mut match_count = 0;
        let global_bounds = Rectangle::from_bounds(0, 0, 10, 10);

        tree.query_intersects(&global_bounds, |rect| {
            assert!(*rect == r1 || *rect == r3 || *rect == r5_new);
            assert!(*rect != r2); // Ensure the deleted one is truly gone
            match_count += 1;
        });

        assert_eq!(match_count, 3);
    }

    #[test]
    fn test_drain_intersects() {
        let mut tree = StaticAABBTree::<20>::new();

        let r1 = Rectangle::from_bounds(0, 0, 5, 5); // Intersects
        let r2 = Rectangle::from_bounds(3, 3, 8, 8); // Intersects
        let r3 = Rectangle::from_bounds(10, 10, 15, 15); // Does NOT intersect
        let r4 = Rectangle::from_bounds(4, 4, 6, 6); // Intersects

        tree.insert(r1.clone()).unwrap();
        tree.insert(r2.clone()).unwrap();
        tree.insert(r3.clone()).unwrap();
        tree.insert(r4.clone()).unwrap();

        assert_eq!(tree.len(), 4);

        let drain_area = Rectangle::from_bounds(2, 2, 7, 7);
        let mut union_rect: Option<Rectangle> = None;
        let mut drain_count = 0;

        // Drain all rectangles intersecting with `drain_area`
        tree.drain_intersects(&drain_area, |rect| {
            drain_count += 1;
            // Build a bounding box covering all the removed rectangles
            union_rect = match &union_rect {
                Some(u) => Some(u.union(&rect)),
                None => Some(rect),
            };
        });

        // Verify exact number of drained items
        assert_eq!(drain_count, 3);
        assert_eq!(tree.len(), 1);

        // Verify the covering union of the drained rectangles:
        // Union of r1 (0,0 to 5,5), r2 (3,3 to 8,8), and r4 (4,4 to 6,6)
        let expected_union = Rectangle::from_bounds(0, 0, 8, 8);
        assert_eq!(union_rect, Some(expected_union));

        // Verify r3 was completely untouched and is still structurally sound
        let mut remaining = 0;
        tree.query_intersects(&Rectangle::from_bounds(-10, -10, 20, 20), |rect| {
            assert_eq!(*rect, r3);
            remaining += 1;
        });
        assert_eq!(remaining, 1);

        // Verify a second drain in the same area yields nothing
        let mut second_drain_count = 0;
        tree.drain_intersects(&drain_area, |_| {
            second_drain_count += 1;
        });
        assert_eq!(second_drain_count, 0);
    }
}
