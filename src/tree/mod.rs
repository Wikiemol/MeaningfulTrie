use std::collections::HashMap;
use std::collections::HashSet;
use crate::lazy::LazySupplier;
use crate::lazy::Lazy;

type NodeRef = usize;
#[derive(Clone)]
pub struct BiDirectionalTreeNode<T> {
    pub value: T,
    pub parents: HashSet<NodeRef>,
    pub children: HashSet<NodeRef>,
}

pub struct BiDirectionalTree<T> {
    nodes: Vec<BiDirectionalTreeNode<T>>,
    reference_counts: HashMap<NodeRef, usize>,
    pub root: NodeRef
}

pub enum EditNodeError {
    NodeDeleted(NodeRef)
}


pub type EditNodeResult = Result<NodeRef, EditNodeError>;


impl<T> BiDirectionalTree<T> {

    pub fn new(value: T) -> BiDirectionalTree<T> {
        BiDirectionalTree {
            nodes: vec![BiDirectionalTreeNode {
                value,
                parents: HashSet::new(),
                children: HashSet::new()
            }],
            reference_counts: HashMap::new(),
            root: 0
        }
    }

    #[inline(always)]
    pub fn get_value_mut(&mut self, idx: NodeRef) -> &mut T {
        &mut self.get_node_mut(idx).value
    }

    #[inline(always)]
    pub fn get_value(&self, idx: NodeRef) -> &T {
        &self.get_node(idx).value
    }

    #[inline(always)]
    pub fn remove_child(&mut self, parent: NodeRef, child: NodeRef) {
        self.get_node_mut(parent).children.remove(&child);
        let child_node = self.get_node_mut(child);
        child_node.parents.remove(&parent);

        if child_node.parents.len() == 0 {
            for grand_child in child_node.children.clone() {
                self.remove_child(child, grand_child);
            }
        }
    }


    #[inline(always)]
    pub fn add_child(&mut self, parent: NodeRef, child: T) -> EditNodeResult {
        let child = self.create_node(child);
        self.add_child_ref(parent, child)
    }


    #[inline(always)]
    /// Returns Some if the parent node exists, otherwise, returns None
    pub fn add_child_ref(&mut self, parent: NodeRef, child: NodeRef) -> EditNodeResult {
        if parent == self.root || self.get_node(parent).parents.len() > 0 {
            self.add_parent(child, parent);
            self.get_node_mut(parent).children.insert(child);
            Ok(child)
        } else {
            Err(EditNodeError::NodeDeleted(parent))
        }
    }

    #[inline(always)]
    fn get_node_mut(&mut self, idx: NodeRef) -> &mut BiDirectionalTreeNode<T> {
        &mut self.nodes[idx]
    }

    #[inline(always)]
    pub fn get_node(&self, idx: NodeRef) -> &BiDirectionalTreeNode<T> {
        &self.nodes[idx]
    }

    #[inline(always)]
    fn add_parent(&mut self, child: NodeRef, parent: NodeRef) {
        self.get_node_mut(child).parents.insert(parent);
    }

    fn create_node(&mut self, value: T) -> NodeRef {
        self.nodes.push(BiDirectionalTreeNode {
            value,
            parents: HashSet::from([]),
            children: HashSet::new()
        });
        self.nodes.len() - 1
    }

    pub fn get_children(&self, parent: NodeRef) -> Vec<&BiDirectionalTreeNode<T>> {
        self.get_node_children(self.get_node(parent))
    }

    pub fn get_node_children(&self, parent: &BiDirectionalTreeNode<T>) -> Vec<&BiDirectionalTreeNode<T>> {
        parent.children.iter()
            .map(|&child| self.get_node(child))
            .collect::<Vec<_>>()
    }

}

impl<T : Clone> BiDirectionalTree<T> {
    pub fn clone(&mut self, idx: NodeRef) -> NodeRef {
        let cloned = self.get_node(idx).clone();
        self.nodes.push(cloned);
        let cloned_ref = self.nodes.len() - 1;
        for existing_child in self.get_node(cloned_ref).children.clone() {
            self.add_parent(existing_child, cloned_ref);
        }
        cloned_ref
    }
}

pub type TreePath = Vec<usize>;

pub struct Tree<T> {
    pub value: T,
    pub children: Vec<Tree<T>>
}

impl<T> Tree<T> {
    pub fn get_node(&self, path: &TreePath) -> &T {
        let mut child = self;
        for &i  in path.iter() {
            child = &child.children[i];
        }
        &child.value
    }
}

impl<T: Clone> Clone for Tree<T> {
    fn clone(&self) -> Self { 
        Tree {
            value: self.value.clone(),
            children: self.children.clone()
        }
    }
}


pub struct LazyTree<T> {
    pub value: T, 
    children: Lazy<Vec<LazyTree<T>>>
}

impl<T> LazyTree<T> {
    pub fn new(value: T, children: LazySupplier<Vec<LazyTree<T>>>) -> LazyTree<T> {
        LazyTree {
            value, 
            children: Lazy::new(children)
        }
    }

    pub fn child(&self, idx: usize) -> &LazyTree<T> {
        &self.children.as_ref()[idx]
    }
    pub fn child_mut(&mut self, idx: usize) -> &mut LazyTree<T> {
        &mut self.children.as_mut()[idx]
    }

    pub fn children(&self) -> &Vec<LazyTree<T>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> &mut Vec<LazyTree<T>> {
        self.children.as_mut()
    }

    pub fn get_node(&self, path: TreePath) -> &T {
        let mut child = self;
        for &i in path.iter() {
            child = child.child(i);
        }
        &child.value
    }
}


pub struct LazyTreeZipper<T> {
    parent: Option<Box<LazyTreeZipper<T>>>,
    node: LazyTree<T>,
    index: usize
}

impl<T> LazyTreeZipper<T> {
    pub fn new(tree: LazyTree<T>) -> LazyTreeZipper<T> {
        LazyTreeZipper {
            parent: None,
            node: tree,
            index: 0
        }
    }

    pub fn child(mut self, index: usize) -> LazyTreeZipper<T> {
        LazyTreeZipper {
            node: self.node.children.as_mut().remove(index),
            parent: Some(Box::new(self)),
            index
        }
    }



    pub fn parent(self) -> Option<LazyTreeZipper<T>> {
        match self.parent {
            None => None,
            Some(mut parent) => {
                parent.as_mut().node.children.as_mut().insert(self.index, self.node);
                Some(*parent)
            }
        }
    }

    pub fn number_of_children(&self) -> usize {
        self.node.children().len()
    }
    pub fn children(&self) -> Vec<&T> {
        self.node.children().iter()
            .map(|child| &child.value)
            .collect::<Vec<_>>()
    }

    pub fn follow_path(self, path: &Vec<usize>) -> LazyTreeZipper<T> {
        let mut zipper = self;
        for &i in path.iter() {
            zipper = zipper.child(i);
        }
        zipper
    }

    #[inline(always)]
    pub fn value(&self) -> &T {
        &self.node.value
    }

    #[inline(always)]
    pub fn replace(&mut self, t: T) {
        self.node.value = t;
    }

    pub fn build(self) -> LazyTree<T> {
        let mut child = self;
        while child.parent.is_some() {
            child = child.parent().unwrap();
        }
        child.node
    }

}




