use super::geometry::Rectangle;

#[derive(Clone, Debug)]
enum NodeType {
    Free { next_free: Option<u8> },
    Leaf { item: Rectangle },
    Internal { left: u8, right: u8 },
}

#[derive(Clone, Debug)]
struct Node {
    aabb: Rectangle,
    parent: Option<u8>,
    node_type: NodeType,
}

const DUMMY_RECT: Rectangle = Rectangle::from_bounds(0, 0, 0, 0);
const DUMMY_NODE: Node = Node {
    aabb: DUMMY_RECT,
    parent: None,
    node_type: NodeType::Free { next_free: None },
};

#[derive(Debug)]
pub struct StaticAABBTree<const CAP: usize> {
    nodes: [Node; CAP],
    root: Option<u8>,
    free_head: Option<u8>,
    size: u8,
}

impl<const CAP: usize> StaticAABBTree<CAP> {
    pub const fn new() -> Self {
        let mut nodes = [DUMMY_NODE; CAP];
        let mut i = 0u8;
        while i < CAP as u8 {
            let next = if i + 1 < CAP as u8 { Some(i + 1) } else { None };
            nodes[i as usize].node_type = NodeType::Free { next_free: next };
            i += 1;
        }

        Self {
            nodes,
            root: None,
            free_head: if CAP > 0 { Some(0) } else { None },
            size: 0,
        }
    }

    pub fn len(&self) -> u8 {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn alloc_node(&mut self) -> Option<u8> {
        if let Some(idx) = self.free_head
            && let NodeType::Free { next_free } = self.nodes[idx as usize].node_type
        {
            self.free_head = next_free;
            return Some(idx);
        }
        None
    }

    fn free_node(&mut self, idx: u8) {
        self.nodes[idx as usize].node_type = NodeType::Free {
            next_free: self.free_head,
        };
        self.nodes[idx as usize].parent = None;
        self.free_head = Some(idx);
    }

    /// Inserts a new rectangle into the tree in O(log N) time.
    pub fn insert(&mut self, rect: Rectangle) -> Result<(), &'static str> {
        if self.root.is_none() {
            let r = self.alloc_node().ok_or("Tree is at maximum capacity")?;
            self.nodes[r as usize] = Node {
                aabb: rect.clone(),
                parent: None,
                node_type: NodeType::Leaf { item: rect },
            };
            self.root = Some(r);
            self.size += 1;
            return Ok(());
        }

        // We need two free nodes for an insert: an internal node and a new leaf node.
        let internal_idx = self.alloc_node().ok_or("Tree is at maximum capacity")?;
        let leaf_idx = self.alloc_node().unwrap_or_else(|| {
            // Rollback if we can only get one node
            self.free_node(internal_idx);
            panic!("Tree is at maximum capacity");
        });

        let target_leaf = self.choose_leaf(&rect);
        let parent_idx = self.nodes[target_leaf as usize].parent;

        // Setup the new leaf
        self.nodes[leaf_idx as usize] = Node {
            aabb: rect.clone(),
            parent: Some(internal_idx),
            node_type: NodeType::Leaf { item: rect.clone() },
        };

        // Setup the new internal node, taking the place of the old leaf
        self.nodes[internal_idx as usize] = Node {
            aabb: self.nodes[target_leaf as usize].aabb.union(&rect),
            parent: parent_idx,
            node_type: NodeType::Internal {
                left: target_leaf,
                right: leaf_idx,
            },
        };

        self.nodes[target_leaf as usize].parent = Some(internal_idx);

        if let Some(p) = parent_idx {
            if let NodeType::Internal { left, right } = &mut self.nodes[p as usize].node_type {
                if *left == target_leaf {
                    *left = internal_idx;
                } else if *right == target_leaf {
                    *right = internal_idx;
                }
            }
        } else {
            self.root = Some(internal_idx);
        }

        self.sync_hierarchy(internal_idx);
        self.size += 1;
        Ok(())
    }

    /// Removes a specific rectangle from the tree in O(log N) time.
    pub fn remove(&mut self, rect: &Rectangle) -> bool {
        let target = match self.find_exact(rect) {
            Some(idx) => idx,
            None => return false,
        };

        let parent_idx = self.nodes[target as usize].parent;

        if let Some(p) = parent_idx {
            let sibling =
                if let NodeType::Internal { left, right } = self.nodes[p as usize].node_type {
                    if left == target { right } else { left }
                } else {
                    unreachable!()
                };

            let grand_parent = self.nodes[p as usize].parent;
            self.nodes[sibling as usize].parent = grand_parent;

            if let Some(gp) = grand_parent {
                if let NodeType::Internal { left, right } = &mut self.nodes[gp as usize].node_type {
                    if *left == p {
                        *left = sibling;
                    } else {
                        *right = sibling;
                    }
                }
                self.sync_hierarchy(gp);
            } else {
                self.root = Some(sibling);
            }

            self.free_node(target);
            self.free_node(p);
        } else {
            // It was the root
            self.free_node(target);
            self.root = None;
        }

        self.size -= 1;
        true
    }

    /// Removes and returns an arbitrary rectangle from the tree.
    /// Returns `None` if the tree is empty. Operates in O(log N) time.
    pub fn pop(&mut self) -> Option<Rectangle> {
        let root_idx = self.root?;
        let mut curr = root_idx;

        // Traverse down the left side of the tree to find an arbitrary leaf
        let target_rect = loop {
            match &self.nodes[curr as usize].node_type {
                NodeType::Leaf { item } => break item.clone(),
                NodeType::Internal { left, .. } => curr = *left,
                _ => unreachable!(),
            }
        };

        // Remove the leaf using the existing balanced removal logic
        self.remove(&target_rect);

        Some(target_rect)
    }

    pub fn insert_ensured(&mut self, rect: Rectangle) -> Rectangle {
        let mut result = rect.clone();

        loop {
            if let Ok(_) = self.insert(result.clone()) {
                return result;
            }

            if let Some(popped) = self.pop() {
                result = result.union(&popped);
                continue;
            } else {
                panic!("Couldn't pop but we also couldn't insert?");
            }
        }
    }

    pub fn query_intersects<F: FnMut(&Rectangle)>(&self, bounds: &Rectangle, mut callback: F) {
        if self.root.is_none() {
            return;
        }

        let mut stack = [0u8; 64];
        let mut top = 0;
        stack[top] = self.root.unwrap();
        top += 1;

        while top > 0 {
            top -= 1;
            let curr = stack[top];
            let node = &self.nodes[curr as usize];

            if !node.aabb.intersects(bounds) {
                continue;
            }

            match &node.node_type {
                NodeType::Leaf { item } => {
                    callback(&item);
                }
                NodeType::Internal { left, right } => {
                    stack[top] = *left;
                    top += 1;
                    stack[top] = *right;
                    top += 1;
                }
                _ => {}
            }
        }
    }

    /// Queries all rectangles strictly contained within the given bounds. O(log N) avg.
    pub fn query_contains<F: FnMut(&Rectangle)>(&self, bounds: &Rectangle, mut callback: F) {
        if self.root.is_none() {
            return;
        }

        let mut stack = [0u8; 64];
        let mut top = 0;
        stack[top] = self.root.unwrap();
        top += 1;

        while top > 0 {
            top -= 1;
            let curr = stack[top];
            let node = &self.nodes[curr as usize];

            if !bounds.intersects(&node.aabb) {
                continue;
            }

            match &node.node_type {
                NodeType::Leaf { item } => {
                    if bounds.contains_rect(&item) {
                        callback(&item);
                    }
                }
                NodeType::Internal { left, right } => {
                    stack[top] = *left;
                    top += 1;
                    stack[top] = *right;
                    top += 1;
                }
                _ => {}
            }
        }
    }

    pub fn drain_intersects<F: FnMut(Rectangle)>(&mut self, bounds: &Rectangle, mut callback: F) {
        loop {
            let mut target_rect = None;

            // 1. Find a single intersecting leaf
            if let Some(root_idx) = self.root {
                let mut stack = [0u8; 64];
                let mut top = 0;
                stack[top] = root_idx as u8;
                top += 1;

                while top > 0 {
                    top -= 1;
                    let curr = stack[top];
                    let node = &self.nodes[curr as usize];

                    if !node.aabb.intersects(bounds) {
                        continue;
                    }

                    match &node.node_type {
                        NodeType::Leaf { item } => {
                            target_rect = Some(item.clone());
                            break;
                        }
                        NodeType::Internal { left, right } => {
                            stack[top] = *left;
                            top += 1;
                            stack[top] = *right;
                            top += 1;
                        }
                        _ => {}
                    }
                }
            }

            // 2. If an intersecting rectangle is found, remove it from the tree
            // and trigger the callback. Otherwise, we are done.
            if let Some(rect) = target_rect {
                self.remove(&rect);
                callback(rect);
            } else {
                break;
            }
        }
    }

    fn choose_leaf(&self, rect: &Rectangle) -> u8 {
        let mut curr = self.root.unwrap();
        loop {
            match self.nodes[curr as usize].node_type {
                NodeType::Leaf { .. } => return curr,
                NodeType::Internal { left, right } => {
                    let aabb_l = &self.nodes[left as usize].aabb;
                    let aabb_r = &self.nodes[right as usize].aabb;

                    let cost_l = aabb_l.union(rect).area() - aabb_l.area();
                    let cost_r = aabb_r.union(rect).area() - aabb_r.area();

                    curr = if cost_l < cost_r { left } else { right };
                }
                _ => unreachable!(),
            }
        }
    }

    fn sync_hierarchy(&mut self, mut curr: u8) {
        loop {
            if let NodeType::Internal { left, right } = self.nodes[curr as usize].node_type {
                self.nodes[curr as usize].aabb = self.nodes[left as usize]
                    .aabb
                    .union(&self.nodes[right as usize].aabb);
            }
            if let Some(p) = self.nodes[curr as usize].parent {
                curr = p;
            } else {
                break;
            }
        }
    }

    fn find_exact(&self, rect: &Rectangle) -> Option<u8> {
        self.root?;

        let mut stack = [0u8; 64];
        let mut top = 0;
        stack[top] = self.root.unwrap();
        top += 1;

        while top > 0 {
            top -= 1;
            let curr = stack[top];
            let node = &self.nodes[curr as usize];

            if !node.aabb.contains_rect(rect) {
                continue;
            }

            match &node.node_type {
                NodeType::Leaf { item } => {
                    if item == rect {
                        return Some(curr);
                    }
                }
                NodeType::Internal { left, right } => {
                    stack[top] = *left;
                    top += 1;
                    stack[top] = *right;
                    top += 1;
                }
                _ => {}
            }
        }
        None
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
