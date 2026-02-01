use crate::core::file;
use crate::core::file::FileType;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use std::fmt;

#[derive(Debug)]
pub enum NodeType {
    UNKNOWN,
    File(FileType),
    Dir,
    SYMLINK,
    ELF,
    SH,
    PE,
}

pub enum NodeCmp {
    NotModified,
    Modified,
}

#[derive(Debug)]
pub struct TreeNode {
    // Type of node
    pub node_type: NodeType,
    // Name of the file or directory
    pub node_name: String,
    // Path of the file/dir in the root fs
    pub node_local_path: String,
    // Path of the file/dir on the system (root fs + local_path)
    pub node_path: String,
    // Hash of the node
    pub node_hash: Option<u64>,
    // Length of the node
    pub node_len: Option<u64>,
    // Childrens of the node
    pub childrens: Arc<Mutex<Vec<Arc<Mutex<TreeNode>>>>>,
    // Parent of the node
    pub parent: Option<Arc<Mutex<TreeNode>>>,
}

impl fmt::Display for TreeNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Customize so only `x` and `y` are denoted.
        write!(
            f,
            "{} [{:?}] \n *{}",
            self.node_local_path, self.node_type, self.node_path
        )
    }
}

impl TreeNode {
    pub fn new(
        node_type: NodeType,
        node_name: String,
        node_local_path: String,
        node_path: String,
        parent: Option<Arc<Mutex<TreeNode>>>,
    ) -> Self {
        TreeNode {
            node_type: node_type,
            node_name: node_name,
            node_local_path: node_local_path,
            node_path: node_path,
            node_hash: None,
            node_len: None,

            childrens: Arc::new(Mutex::new(Vec::new())),
            parent: None,
        }
    }

    /*
        Set type of node
    */
    pub fn set_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }

    /*
        Find a node by name in childrens
    */
    pub fn find_node_by_name(&self, name: &str) -> Option<Arc<Mutex<TreeNode>>> {
        let childrens = self.childrens.lock().unwrap();

        for child in &(*childrens) {
            let c = child.lock().unwrap();

            match &c.node_type {
                NodeType::File(ft) => {
                    if c.node_path.ends_with(name) {
                        drop(c);
                        return Some(child.clone());
                    }
                }
                NodeType::Dir => {
                    if c.node_path.ends_with(name) {
                        drop(c);
                        return Some(child.clone());
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn cmp_node(&self, other_node: Arc<Mutex<TreeNode>>) -> NodeCmp {
        let mut res = NodeCmp::NotModified;

        let o_node = other_node.lock().unwrap();

        match self.node_type {
            NodeType::File(FileType::Elf(_)) => {
                let path1 = &self.node_path;
                let path2 = &o_node.node_path;

                let f1 = std::fs::File::open(path1).unwrap();
                let f2 = std::fs::File::open(path2).unwrap();

                let b1 = std::fs::read(path1).unwrap();
                let b2 = std::fs::read(path2).unwrap();

                if file::elf::simple_elf_cmp(
                    b1.as_slice() as *const _ as *const u8,
                    b2.as_slice() as *const _ as *const u8,
                ) || f1.metadata().unwrap().len() != f2.metadata().unwrap().len()
                {
                    res = NodeCmp::Modified;
                }
            }
            _ => {
                if let Some(hash) = self.node_hash {
                    if let Some(o_hash) = o_node.node_hash {
                        if hash != o_hash {
                            res = NodeCmp::Modified;
                        }
                    }
                }
            }
        }
        res
    }
}
