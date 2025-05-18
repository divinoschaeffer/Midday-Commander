use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

#[derive(Debug, PartialEq)]
pub enum FsNodeType {
    File,
    Directory,
}

#[derive(Debug)]
pub struct FsNode {
    pub name: String,
    pub path: PathBuf,
    pub node_type: FsNodeType,
    pub parent: Option<Weak<FsNode>>,
    pub children: Vec<Rc<RefCell<FsNode>>>,
}

impl FsNode {
    pub fn new(
        name: String,
        path: PathBuf,
        fs_node_type: FsNodeType,
        parent: Option<Weak<FsNode>>,
        children: Vec<Rc<RefCell<FsNode>>>
    ) -> FsNode {
        FsNode {
            name,
            path,
            node_type: fs_node_type,
            parent,
            children,
        }
    }

    /// add child to a node
    pub fn add_child(&mut self, child:FsNode) {
        let child = Rc::new(RefCell::new(child));
        self.children.push(child);
    }

    /// find node amongst the direct children of a node
    pub fn find_node(
        &mut self,
        path: PathBuf,
        fs_node_type: Option<FsNodeType>
    ) -> Option<Rc<RefCell<FsNode>>> {
        self.children.iter().find_map(|child| {
            let node = child.borrow();
            if node.path == path
                && (fs_node_type.is_none() || (fs_node_type.as_ref() == Some(&node.node_type))) {
                Some(Rc::clone(child))
            }
            else {
                None
            }
        })
    }

    /// remove node if there is one and return it
    pub fn remove_node(&mut self, path: PathBuf, fs_node_type: Option<FsNodeType>) -> Option<Rc<RefCell<FsNode>>> {
        let position = self.children
            .iter()
            .position(|child|
                child.borrow().path == path
                    && (fs_node_type.is_none() || (fs_node_type.as_ref() == Some(&child.borrow().node_type)))
            );

        if let Some(position) = position {
            Some(Rc::clone(&self.children.remove(position)))
        } else {
            None
        }
    }
}