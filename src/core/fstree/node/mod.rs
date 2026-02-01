
use crate::core::file;
use crate::core::file::FileType;

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use std::fs;
use std::fmt;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

type NbDir = u64;
type NbFile = u64;

#[derive(Debug)]
pub enum NodeType {
    File(Option<FileType>),
    Dir,
}

//pub type Node = Arc<Mutex<FsTreeNode>>;

#[derive(Debug)]
pub struct NodeInner {
    // Type of node
    pub node_type: NodeType,
    // Name of the file or directory
    pub name: String,
    // Path of the file/dir in the root fs
    pub local_path: String,
    // Path of the file/dir on the system (root fs + local_path)
    pub fs_path: String,
    // Hash of the node
    pub hash: Option<u64>,
    // Length of the node
    pub len: u64,
    // Childrens of the node
    pub childrens: Arc<RwLock<Vec<Node>>>,
    // Parent of the node
    pub parent: Option<Node>,
}

impl NodeInner {
    pub fn new(name: &str, node_type: NodeType, local_path: &str, fs_path: &str, parent: Option<Node>) -> Self {
        let metadata = fs::metadata(local_path).unwrap();
        
        Self {
            node_type,
            name: name.to_string(),
            local_path: local_path.to_string(),
            fs_path: fs_path.to_string(),
            hash: None,
            len: metadata.len(),
            childrens: Arc::new(RwLock::new(Vec::new())),
            parent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    inner: Arc<RwLock<NodeInner>>,
}

impl Node {
    pub fn new_file(root_path: &str, name: &str, local_path: &str, fs_path: &str, parent: Option<Node>) -> Self {
        let node_inner = NodeInner::new(name, NodeType::File(None), local_path, fs_path, parent);
        Self {
            inner: Arc::new(RwLock::new(node_inner)),
        }
    }
    
    pub fn new_dir(root_path: &str, name: &str, local_path: &str, fs_path: &str, parent: Option<Node>) -> Self {
        let node_inner = NodeInner::new(name, NodeType::Dir, local_path, fs_path, parent);
        
        let node = Self {
            inner: Arc::new(RwLock::new(node_inner)),
        };
        
        let mut childrens: Vec<Node> = Vec::new();
        
        for entry in fs::read_dir(local_path).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let entry_type = entry.file_type().unwrap();
            
            let file_name = entry_path.file_name().unwrap().to_str().unwrap();
            let local_path = entry_path.as_path().to_str().unwrap().to_string();
            let fs_path = local_path.replace(root_path, "/");
            
            if entry_type.is_dir() {
                let dir_node = Self::new_dir(root_path, file_name, &local_path, &fs_path, Some(node.clone()));
                childrens.push(dir_node);
            }
            else if entry_type.is_file() {
                let file_node = Self::new_file(root_path, file_name, &local_path, &fs_path, Some(node.clone()));
                childrens.push(file_node);
            }
        }
        
        node.set_childrens(childrens);
        node
    }
    
    pub fn inner(&self) -> RwLockReadGuard<NodeInner> {
        self.inner.read().unwrap()
    }
    
    pub fn is_dir(&self) -> bool {
        let inner = self.inner.read().unwrap();
        match inner.node_type {
            NodeType::Dir => true,
            _ => false,
        }
    }
    
    pub fn is_file(&self) -> bool {
        let inner = self.inner.read().unwrap();
        match inner.node_type {
            NodeType::File(_) => true,
            _ => false,
        }
    }
    
    pub fn is_elf(&self) -> bool {
        let inner = self.inner.read().unwrap();
        match inner.node_type {
            NodeType::File(Some(FileType::Elf(_))) => true,
            _ => false,
        }
    }
    
    pub fn len(&self) -> u64 {
        let inner = self.inner.read().unwrap();
        inner.len
    }
    
    pub fn local_path(&self) -> String {
        let inner = self.inner.read().unwrap();
        inner.local_path.clone()
    }
    
    pub fn name(&self) -> String {
        let inner = self.inner.read().unwrap();
        inner.name.clone()
    }
    
    fn set_type(&self, node_type: NodeType) {
        let mut inner = self.inner.write().unwrap();
        inner.node_type = node_type;
    }
    
    fn set_hash(&self, hash: u64) {
        let mut inner = self.inner.write().unwrap();
        inner.hash = Some(hash);
    }
    
    fn set_childrens(&self, childrens: Vec<Node>) -> bool {
        let mut inner = self.inner.write().unwrap();
        inner.childrens = Arc::new(RwLock::new(childrens));
        true
    }
    
    pub fn find_node_by_name_rec(&self, name: &str) -> Vec<Node> {
        
        let mut node_list: Vec<Node> = Vec::new();
        
        let inner = self.inner.read().unwrap();
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            if child.is_dir() {
                let mut nodes = child.find_node_by_name_rec(name);
                node_list.append(&mut nodes);
            }
            else if child.is_file() {
                if name == child.name() {
                    node_list.push(child.clone());
                }
            }
        }
        
        node_list
    }
    
    pub fn count_dirs_rec(&self) -> u64 {
        let mut count = 0;
        let inner = self.inner.read().unwrap();
        
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            if child.is_dir() {
                count += child.count_dirs_rec();
                count += 1;
            }
        }
        
        count
    }
    
    pub fn count_files_rec(&self) -> u64 {
        let mut count = 0;
        let inner = self.inner.read().unwrap();
        
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            if child.is_dir() {
                count += child.count_files_rec();
            }
            else if child.is_file() {
                count += 1;
            }
        }
        
        count
    }
    
    pub fn analyse_files_type_rec(&self) {
        let inner = self.inner.read().unwrap();
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            if child.is_dir() {
                child.analyse_files_type_rec();
            }
            else if child.is_file() {
                if child.len() <= 50000000 {
                    let bytes = fs::read(child.local_path()).unwrap();
                    let file_type = file::check_type(&child.name(), bytes.as_slice());
                    child.set_type(NodeType::File(Some(file_type)));
                }
            }
        }
    }
    
    pub fn analyse_binaries_rec(&self, head_node: Node) {
        let inner = self.inner.read().unwrap();
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            
            //let node_type = &child.inner().node_type;
            
            if child.is_dir() {
                child.analyse_binaries_rec(head_node.clone());
            }
            else if child.is_elf() {
            //else if let NodeType::File(Some(FileType::Elf(None))) = child.inner().node_type {
                let elf_data = file::elf::analyse_elf2(head_node.clone(), &child.local_path());
                child.set_type(NodeType::File(Some(FileType::Elf(Some(elf_data)))));
            }
        }
    }
    
    pub fn calc_files_hash_rec(&self) {
        let inner = self.inner.read().unwrap();
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            if child.is_dir() {
                child.calc_files_hash_rec();
            }
            else if child.is_file() {
                if child.len() <= 50000000 {
                    let bytes = fs::read(child.local_path()).unwrap();
                    let mut s = DefaultHasher::new();
                    s.write(&bytes);
                    let hash = s.finish();
                    child.set_hash(hash);
                }
            }
        }
    }
    
    pub fn list_files_rec(&self) {
        let inner = self.inner.read().unwrap();
        let childrens = inner.childrens.read().unwrap();
        
        for child in &(*childrens) {
            if child.is_dir() {
                child.list_files_rec();
            }
            else if child.is_file() {
                println!("{}", child);
            }
        }
    }
    
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.inner.read().unwrap();
        
        write!(
            f,
            "{:?} {} - {} {} bytes ({})",
            inner.node_type, inner.fs_path, inner.name, inner.len, inner.local_path,
        )
    }
}