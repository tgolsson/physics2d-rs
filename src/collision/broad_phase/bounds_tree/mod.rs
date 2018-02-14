extern crate core;

#[cfg(test)]
mod tests;

use math::{Bounds, Vec2};
use util::pool;
use world::{Body, BodyMap, BodyId, BodyPair};

use std;
use collision::broad_phase::{BroadPhase, BodyPairSet, ProxyId};

trait Nullable {
    const NULL: Self;
}

type NodeId = pool::PoolId;

impl Nullable for NodeId {
    const NULL: NodeId = std::usize::MAX;
}

#[derive(Clone)]
struct Node<T: Default> {
    bounds: Bounds,
    
    parent: NodeId,
    left: NodeId,
    right: NodeId,
    
    data: T,
    
    height: u32,
}

impl<T: Default> Default for Node<T> {
    fn default() -> Node<T> {
        Node {
            bounds: Bounds::new(Vec2::ZERO, Vec2::ZERO),
            parent: NodeId::NULL,
            left: NodeId::NULL,
            right: NodeId::NULL,
            data: T::default(),
            height: 0,
        }
    }
}

impl<T: Default> Node<T> {
    fn new(bounds: Bounds, data: T) -> Node<T> {
        Node {
            bounds,
            parent: NodeId::NULL,
            left: NodeId::NULL,
            right: NodeId::NULL,
            data,
            height: 0,
        }
    }
    
    fn is_leaf(&self) -> bool {
        self.left == NodeId::NULL
    }
}

struct BoundsTree<T: Default> {
    pool: pool::Pool<Node<T>>,
    pub root_id: NodeId,
}

impl<T: Default> BoundsTree<T> {
    pub fn new() -> BoundsTree<T> {
        let mut pool = pool::Pool::default();
        BoundsTree {
            root_id: NodeId::NULL,
            pool,
        }
    }
    
    pub fn get_root(&self) -> &Node<T> {
        self.get_node(self.root_id)
    }
    
    pub fn get_root_mut(&mut self) -> &mut Node<T> {
        let root_id = self.root_id;
        self.get_node_mut(root_id)
    }
    
    pub fn get_node(&self, node_id: NodeId) -> &Node<T> {
        self.pool.get(node_id)
    }
    
    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut Node<T> {
        self.pool.get_mut(node_id)
    }
    
    /// Walks up the tree from `node_id`, balancing subtrees and fixing ancestor heights and bounds.
    fn update_ancestors(&mut self, mut node_id: NodeId) {
        while node_id != NodeId::NULL {
            // Balance
        
            let left_id = self.get_node(node_id).left;
            let right_id = self.get_node(node_id).right;
        
            let new_height = 1 + self.get_node(left_id).height.max(self.get_node(right_id).height);
            let new_bounds = self.get_node(left_id).bounds.union(&self.get_node(right_id).bounds);
        
            let node = self.get_node_mut(node_id);
            node.height = new_height;
            node.bounds = new_bounds;
        
            node_id = node.parent;
        }
    }
    
    /// Inserts a leaf node with the given `data` into the tree.
    ///
    /// When inserting the leaf node in a new branch with an existing leaf, the existing leaf is
    /// made the `left` child of the new branch, and the new leaf node is made the `right` child.
    ///
    /// # Returns
    /// The `NodeId` of the inserted leaf node.
    fn insert_leaf(&mut self, bounds: Bounds, data: T) -> NodeId {
        let leaf_id = self.pool.allocate_with(Node::new(bounds, data));
        
        if self.root_id == NodeId::NULL {
            self.root_id = leaf_id;
            return leaf_id;
        }
    
        let leaf_bounds = self.get_node(leaf_id).bounds;
        let mut sibling_id = self.root_id;
        
        while !self.get_node(sibling_id).is_leaf() {
            let sibling = self.get_node(sibling_id);
            
            let left = self.get_node(sibling.left);
            let right = self.get_node(sibling.right);
            
            let cost_left = left.bounds.union(&leaf_bounds).perimeter() - left.bounds.perimeter();
            let cost_right = right.bounds.union(&leaf_bounds).perimeter() - right.bounds.perimeter();
            
            sibling_id = if cost_left < cost_right {
                sibling.left
            } else {
                sibling.right
            }
        }
        
        // Create new node that will become parent and replace sibling's position in the tree
        
        let parent_id = self.pool.allocate();
        
        let sibling_parent_id = self.get_node(sibling_id).parent;
        let sibling_bounds = self.get_node(sibling_id).bounds;
        let sibling_height = self.get_node(sibling_id).height;
        
        {
            let parent = self.get_node_mut(parent_id);
            parent.parent = sibling_parent_id;
            parent.bounds = leaf_bounds.union(&sibling_bounds);
            parent.height = sibling_height + 1;
            
            parent.left = sibling_id;
            parent.right = leaf_id;
        }
        
        self.get_node_mut(sibling_id).parent = parent_id;
        self.get_node_mut(leaf_id).parent = parent_id;
        
        // Place new parent in sibling's position in tree
        
        if sibling_id == self.root_id {
            self.root_id = parent_id;
        } else {
            let is_sibling_left_child = self.get_node(sibling_parent_id).left == sibling_id;
            
            if is_sibling_left_child {
                self.get_node_mut(sibling_parent_id).left = parent_id;
            } else {
                self.get_node_mut(sibling_parent_id).right = parent_id;
            }
        }
        
        let parent_id = self.get_node(leaf_id).parent;
        self.update_ancestors(parent_id);
        
        leaf_id
    }
    
    /// Removes the leaf identified by `leaf_id` from the tree.
    ///
    /// The removal process involves replacing the parent of the leaf with its sibling.
    fn remove_leaf(&mut self, leaf_id: NodeId) {
        if self.root_id == leaf_id {
            self.root_id = NodeId::NULL;
            self.pool.free(leaf_id);
            return;
        }
        
        let parent_id = self.get_node(leaf_id).parent;
        let sibling_id = {
            let parent = self.get_node(parent_id);
            if parent.left == leaf_id {
                parent.right
            } else {
                parent.left
            }
        };
        
        if self.root_id == parent_id {
            // Root is parent, replace root
            
            self.pool.free(parent_id);
            
            self.root_id = sibling_id;
            self.get_node_mut(sibling_id).parent = NodeId::NULL;
            
            self.pool.free(leaf_id);
            
            return;
        }
    
        // Delete parent and connect sibling to grandparent in its place
        
        let grandparent_id = self.get_node(parent_id).parent;
    
        let is_parent_left_child = self.get_node(grandparent_id).left == parent_id;
        
        self.pool.free(parent_id);
    
        if is_parent_left_child {
            self.get_node_mut(grandparent_id).left = sibling_id;
        } else {
            self.get_node_mut(grandparent_id).right = sibling_id;
        }
        
        self.get_node_mut(sibling_id).parent = grandparent_id;
        
        self.update_ancestors(grandparent_id);
        
        self.pool.free(leaf_id);
    }
    
    fn query<F>(&self, bounds: Bounds, mut f: F)
        where F: FnMut(&Node<T>) -> bool {
        let mut stack = Vec::with_capacity(self.pool.object_count);
        stack.push(self.root_id);
    
        while stack.len() > 0 {
            let node_id = stack.pop().unwrap();
            let node = self.get_node(node_id);
            
            if !bounds.intersects(&node.bounds) {
                continue;
            }
            
            if node.is_leaf() {
                if !f(node) {
                    return;
                }
            } else {
                stack.push(node.left);
                stack.push(node.right);
            }
        }
    }
}

pub struct BoundsTreeBroadPhase {
    tree: BoundsTree<BodyId>,
    reinserted_bodies: Vec<BodyId>,
}

impl BoundsTreeBroadPhase {
    pub fn new() -> BoundsTreeBroadPhase {
        BoundsTreeBroadPhase {
            tree: BoundsTree::new(),
            reinserted_bodies: Vec::new(),
        }
    }
    
    pub fn post_update(&mut self) {
        self.reinserted_bodies.clear();
    }
}

const MARGIN: f32 = 0.05;

impl BroadPhase for BoundsTreeBroadPhase {
    fn new_potential_pairs(&self, bodies: &BodyMap) -> BodyPairSet {
        let mut pairs = BodyPairSet::default();
    
        for body_id in self.reinserted_bodies.iter() {
            self.tree.query(bodies[&body_id].borrow().bounds, |node| {
                if node.data == *body_id {
                    return true;
                }
                
                pairs.insert(BodyPair::new(node.data, *body_id));
                true
            });
        }
        
        pairs
    }
    
    fn create_proxy(&mut self, body: &Body) -> ProxyId {
        self.tree.insert_leaf(body.bounds.expand(MARGIN), body.id)
    }
    
    fn destroy_proxy(&mut self, proxy_id: ProxyId) {
        self.tree.remove_leaf(proxy_id);
    }
    
    fn update_proxy(&mut self, proxy_id: ProxyId, body: &Body) {
        // TODO: Explore rotation based method instead
        
        if self.tree.get_node(proxy_id).bounds.contains(&body.bounds) {
            return;
        }
        
        // Proxy (node) id is guaranteed to remain the same after the reinsert
        
        self.destroy_proxy(proxy_id);
        
        self.create_proxy(body);
        
        self.reinserted_bodies.push(body.id);
    }
}
