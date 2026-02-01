

pub mod node;

use node::Node;

use std::sync::{Arc, Mutex};

use std::fs::metadata;
use std::path::Path;

pub struct FsTree {
    // Path of the FS on the local system
    pub path: String,
    
    // Root directory node
    pub head_node: Node,
}

impl FsTree {
    pub fn build_from_path(path: &str) -> Self {
        let path_metadata = metadata(path).unwrap();
        
        if !path_metadata.is_dir() {
            panic!("The root path is not a directory");
        } else {
            // Get the directory name
            let dir_name = Path::new(path).file_name().unwrap().to_str().unwrap();
            
            // Build the tree
            let head_node = Node::new_dir(path, dir_name, path, path, None);
            
            let fstree = Self {
                path: path.to_string(),
                head_node,
            };
            fstree
        }
    }
    
    pub fn count_dirs(&self) -> u64 {
        self.head_node.count_dirs_rec()
    }
    
    pub fn count_files(&self) -> u64 {
        self.head_node.count_files_rec()
    }
    
    pub fn analyse_files_type(&self) {
        self.head_node.analyse_files_type_rec();
    }
    
    pub fn calc_files_hash(&self) {
        self.head_node.calc_files_hash_rec();
    }
    
    pub fn list_files(&self) {
        self.head_node.list_files_rec();
    }
    
    pub fn list_path(&self, path: &str) {
            
    }
    
    pub fn analyse_binaries(&self) {
        self.head_node.analyse_binaries_rec(self.head_node.clone());
    }
    
}
