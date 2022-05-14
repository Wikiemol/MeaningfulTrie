use crate::lazy::LazySupplier;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use crate::lazy::Lazy;

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




