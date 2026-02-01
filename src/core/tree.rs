use crate::core::file;
use crate::core::file::FileType;
use crate::core::node::{NodeType, TreeNode};

use std::cell::RefCell;
use std::rc::Rc;

use std::fs::{self, DirEntry, File};

use std::sync::{Arc, Mutex, RwLock};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::collections::HashMap;

use colored::Colorize;

use siphasher::sip::{SipHasher, SipHasher13, SipHasher24};

use super::node::NodeCmp;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum FsUpdate {
    New,
    Removed,
    Moved,
    Modified,
}

pub struct TreeCmpResult {
    pub fs_updates: HashMap<FsUpdate, Vec<Arc<Mutex<TreeNode>>>>,

    pub new_files: Vec<Arc<Mutex<TreeNode>>>,
    pub new_dirs: Vec<Arc<Mutex<TreeNode>>>,
    pub removed_files: Vec<Arc<Mutex<TreeNode>>>,
    pub modified_files: Vec<Arc<Mutex<TreeNode>>>,
}

impl TreeCmpResult {
    pub fn new() -> Self {
        let fs_updates = HashMap::from([
            (FsUpdate::New, Vec::new()),
            (FsUpdate::Removed, Vec::new()),
            (FsUpdate::Moved, Vec::new()),
            (FsUpdate::Modified, Vec::new()),
        ]);

        TreeCmpResult {
            fs_updates: fs_updates,
            new_files: Vec::new(),
            new_dirs: Vec::new(),
            removed_files: Vec::new(),
            modified_files: Vec::new(),
        }
    }

    pub fn display_new_elf() {    
    }
    
    pub fn display_data(&self) {
        for (fs_update, vec) in &self.fs_updates {
            println!("{:?}", fs_update);
            for node in vec {
                let n = node.lock().unwrap();
                println!("{}", n);
            }
        }
    }

    pub fn display_count(&self) {
        for (fs_update, vec) in &self.fs_updates {
            println!("{:?} {}", fs_update, vec.len());
        }
    }

    // pub fn display_data(&self) {
    //     println!("{} new dirs", self.new_dirs.len());

    //     for new_dir in &self.new_dirs {
    //         let c = new_dir.lock().unwrap();
    //         println!("{}", c.node_local_path);
    //     }

    //     println!("{} new files", self.new_files.len());

    //     for new_file in &self.new_files {
    //         let c = new_file.lock().unwrap();
    //         println!("{}", c.node_local_path);
    //     }

    //     println!("{} deleted files", self.removed_files.len());

    //     for removed_file in &self.removed_files {
    //         let c = removed_file.lock().unwrap();
    //         println!("{}", c.node_local_path);
    //     }
    // }

    // pub fn display_count(&self) {
    //     println!("{} new dirs", self.new_dirs.len());
    //     println!("{} new files", self.new_files.len());
    //     println!("{} deleted files", self.removed_files.len());
    //     println!("{} modified files", self.modified_files.len());
    // }
}

/*
*/
pub struct TreeData {
    pub path: String,
    pub head_node: Option<Arc<Mutex<TreeNode>>>,
}

impl TreeData {
    pub fn new(path: &str) -> Self {
        let mut td = TreeData {
            path: path.to_string(),
            head_node: None,
        };
        let head_node = td.create_node(path, path);
        td.head_node = Some(head_node);
        td.analyse_nodes_type();
        td
    }
    
    /*
        Analyse the type of all nodes
    */
    pub fn analyse_nodes_type(&self) {
        match &self.head_node {
            Some(node) => {
                self.analyse_node_type(node.clone());
            }
            None => {}
        }
    }
    
    
    /*
        Analyse all nodes
    */
    pub fn analyse_tree(&self) {
        match &self.head_node {
            Some(node) => {
                self.analyse_node(node.clone());
            }
            None => {}
        }
    }
    

    fn handle_dir_entry(&self, parent: Arc<Mutex<TreeNode>>, entry: DirEntry) {
        let entry_path = entry.path();
        let file_type = entry.file_type().unwrap();
        let entry_path_str = entry_path.as_path().to_str().unwrap().to_string();

        if file_type.is_dir() {
            let node = self.create_node(&self.path, &entry_path_str);
            let p = parent.lock().unwrap();
            p.childrens.lock().unwrap().push(node.clone());
        } else if file_type.is_file() {
            let node_name = entry_path_str.split("/").last().unwrap();
            let node_local_path =
                "/".to_owned() + &entry_path_str.strip_prefix(&self.path).unwrap().to_string();

            let node = TreeNode::new(
                NodeType::File(FileType::Data),
                node_name.to_string(),
                node_local_path,
                entry_path_str.to_string(),
                Some(parent.clone()),
            );

            let p = parent.lock().unwrap();
            p.childrens.lock().unwrap().push(Arc::new(Mutex::new(node)));
        }
    }

    /*
       Create tree nodes from path
    */
    pub fn create_node(&self, root_path: &str, path: &str) -> Arc<Mutex<TreeNode>> {
        let node_name = path.split("/").last().unwrap();
        let node_local_path = "/".to_owned() + &path.strip_prefix(root_path).unwrap().to_string();
        let dir_node = TreeNode::new(
            NodeType::Dir,
            node_name.to_string(),
            node_local_path,
            path.to_string(),
            None,
        );
        let parent = Arc::new(Mutex::new(dir_node));

        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            self.handle_dir_entry(parent.clone(), entry);
        }
        parent
    }

    pub fn find_nodes_by_name(&self, name: &str) -> Vec<Arc<Mutex<TreeNode>>> {
        let mut res: Vec<Arc<Mutex<TreeNode>>> = Vec::new();

        let head_node = self.head_node.as_ref().unwrap();
        self.find_nodes_by_name_rec(head_node.clone(), name, &mut res);
        res
    }

    pub fn find_nodes_by_name_rec(
        &self,
        node: Arc<Mutex<TreeNode>>,
        name: &str,
        vec: &mut Vec<Arc<Mutex<TreeNode>>>,
    ) {
        let node = node.lock().unwrap();
        let childrens = node.childrens.lock().unwrap();

        for child in &(*childrens) {
            let mut c = child.lock().unwrap();

            if c.node_name == name {
                vec.push(child.clone());
            }
        }
    }

    /*
        Counter number of childrens of type File
    */
    pub fn count_node_files(&self, node: Arc<Mutex<TreeNode>>) -> u64 {
        let mut count: u64 = 0;
        let node = node.lock().unwrap();
        let childrens = node.childrens.lock().unwrap();

        let mut s = DefaultHasher::new();

        for child in &(*childrens) {
            let mut c = child.lock().unwrap();

            match c.node_type {
                NodeType::File(_) => {
                    count += 1;
                }
                NodeType::Dir => {
                    drop(c);
                    count += self.count_node_files(child.clone());
                }
                _ => {}
            }
        }
        count
    }

    /*
        Count number of node of type File
    */
    pub fn count_files(&self) -> u64 {
        let head_node = self.head_node.as_ref().unwrap();
        self.count_node_files(head_node.clone())
    }

    pub fn analyse_node_type(&self, node: Arc<Mutex<TreeNode>>) {
        let node = node.lock().unwrap();
        let childrens = node.childrens.lock().unwrap();

        let mut s = DefaultHasher::new();
        let mut s = SipHasher24::new();

        for child in &(*childrens) {
            let mut c = child.lock().unwrap();

            match c.node_type {
                NodeType::File(FileType::Data) => {
                    let f = std::fs::File::open(&c.node_path).unwrap();
                    let file_size = f.metadata().unwrap().len();

                    if file_size <= 50000000 {
                        let bytes = std::fs::read(&c.node_path).unwrap();
                        let mut s = DefaultHasher::new();
                        s.write(&bytes);
                        let hash = s.finish();
                        c.node_hash = Some(hash);

                        let file_type = file::check_type(&c.node_name, bytes.as_slice());
                        c.node_type = NodeType::File(file_type);
                    }
                }
                NodeType::Dir => {
                    drop(c);
                    self.analyse_node_type(child.clone());
                }
                _ => {}
            }
        }
    }

    pub fn analyse_node(&self, node: Arc<Mutex<TreeNode>>) {
        let node = node.lock().unwrap();
        let childrens = node.childrens.lock().unwrap();

        let mut s = DefaultHasher::new();
        let mut s = SipHasher24::new();

        for child in &(*childrens) {
            let mut c = child.lock().unwrap();
            let path = c.node_path.to_string();
            
            /*
                Set the len of the node
            */
            match c.node_type {
                NodeType::Dir => {
                }
                NodeType::File(_) => {
                    let file = File::open(&path).unwrap();
                    let file_size = file.metadata().unwrap().len();
                    c.node_len = Some(file_size);
                }
                _ => {}
            }
            
            /*
                Analaysis based on the file type
            */
            match c.node_type {
                NodeType::File(FileType::Elf(None)) => {
                    //let path = c.node_path.to_string();
                    drop(c);
                    //drop(childrens);
                    let elf_data = file::elf::analyse_elf(self, &path);

                    let mut c = child.lock().unwrap();
                    //let node = node.lock().unwrap();
                    //let childrens = node.childrens.lock().unwrap();

                    c.node_type = NodeType::File(FileType::Elf(Some(elf_data)));
                }
                NodeType::Dir => {
                    drop(c);
                    self.analyse_node(child.clone());
                }
                _ => {}
            }
        }
    }

    /*
        Compare nodes with another TreeData
    */
    pub fn compare_tree_data(&self, other: TreeData) -> TreeCmpResult {
        let mut result = TreeCmpResult::new();

        let head_node = self.head_node.as_ref().unwrap();
        let other_head_node = other.head_node.as_ref().unwrap();

        self.compare_node(&mut result, head_node.clone(), other_head_node.clone());
        result
    }

    /*
        Compare 2 given nodes
    */
    pub fn compare_node(
        &self,
        result: &mut TreeCmpResult,
        node: Arc<Mutex<TreeNode>>,
        other_node: Arc<Mutex<TreeNode>>,
    ) {
        let node = node.lock().unwrap();
        let childrens = node.childrens.lock().unwrap();

        // Used to track all found file/directories
        let mut node_names: Vec<String> = Vec::new();

        for child in &(*childrens) {
            let mut c = child.lock().unwrap();

            match c.node_type {
                // If the current child note type is file
                NodeType::File(_) => {
                    // Get the other_node lock to check if the file exists
                    let o_node = other_node.lock().unwrap();
                    let exists = o_node.find_node_by_name(&c.node_name);
                    match exists {
                        // File exists
                        Some(node) => {
                            node_names.push(c.node_name.clone());

                            //let n = node.lock().unwrap();
                            // match c.node_hash {
                            //     Some(hash) => {
                            //         if let Some
                            //     }
                            //     None => {}
                            // }

                            match c.cmp_node(node.clone()) {
                                NodeCmp::Modified => {
                                    let n = node.lock().unwrap();
                                    // println!("{} {:?}", c.node_path, c.node_type);
                                    // println!("{} {:?}", n.node_path, n.node_type);

                                    let mut fs_updates_vec =
                                        result.fs_updates.get_mut(&FsUpdate::Modified).unwrap();
                                    fs_updates_vec.push(child.clone());
                                }
                                // Nothing to do if the file is not modified
                                NodeCmp::NotModified => {}
                            };

                            // let n = node.lock().unwrap();

                            // if let Some(hash) = c.node_hash {
                            //     if let Some(o_hash) = n.node_hash {
                            //         //println!("{:?}", c.node_hash);
                            //         //println!("Ok");
                            //         if hash != o_hash {
                            //             let mut fs_updates_vec =
                            //                 result.fs_updates.get_mut(&FsUpdate::Modified).unwrap();
                            //             fs_updates_vec.push(child.clone());

                            //             result.modified_files.push(node.clone());
                            //             //println!("New hash for {} {}", c.node_path, n.node_path);
                            //             // println!(
                            //             //     "New hash: {}, old hash: {} {} {}",
                            //             //     hash, o_hash, c.node_path, n.node_path
                            //             // );
                            //         }
                            //     }
                            // }

                            // drop(c);
                            // drop(o_node);
                            // self.compare_node(child.clone(), other_node.clone());
                            //println!("{} {:?} found", "FILE".green(), c.node_path);
                        }
                        // File not found (deleted/moved)
                        None => {
                            let mut fs_updates_vec =
                                result.fs_updates.get_mut(&FsUpdate::Removed).unwrap();
                            fs_updates_vec.push(child.clone());

                            result.removed_files.push(child.clone());
                            //println!("{} {:?} not found", "FILE".red(), c.node_local_path);
                        }
                    }
                }
                // If the current child node type is dir
                NodeType::Dir => {
                    // Get the other_node lock to check if the dir exists
                    let o_node = other_node.lock().unwrap();
                    let dir_path = c.node_path.replace(&self.path, "/");
                    let exists = o_node.find_node_by_name(&dir_path);

                    match exists {
                        // Directory exists. Recursive call with the 2 child nodes
                        Some(node) => {
                            node_names.push(c.node_name.clone());
                            let c_o_node = node.lock().unwrap();
                            drop(c);
                            drop(o_node);
                            drop(c_o_node);
                            self.compare_node(result, child.clone(), node.clone());
                        }
                        // Directory not found (deleted/moved)
                        None => {
                            //println!("{} {:?} not found", "DIR".red(), c.node_local_path);
                        }
                    }
                }
                _ => {}
            }
        }

        // Check with node_names to detect new files/directories
        let mut fs_updates_vec = result.fs_updates.get_mut(&FsUpdate::New).unwrap();
        let o_node = other_node.lock().unwrap();
        let o_childrens = o_node.childrens.lock().unwrap();
        for child in &(*o_childrens) {
            let mut c = child.lock().unwrap();
            if !node_names.contains(&c.node_name) {
                // let node_type = c.node_type;
                // drop(c);

                match c.node_type {
                    NodeType::File(_) => result.new_files.push(child.clone()),
                    NodeType::Dir => result.new_dirs.push(child.clone()),
                    _ => {}
                }

                match c.node_type {
                    NodeType::File(_) => {
                        fs_updates_vec.push(child.clone());
                    }
                    NodeType::Dir => fs_updates_vec.push(child.clone()),
                    _ => {}
                }

                //println!("{} {:?} is new", "NEW".green(), c.node_local_path);
            }
        }
    }

    pub fn display_files(&self) {
        let head_node = self.head_node.as_ref().unwrap();

        self.display_node_files(head_node.clone());
    }

    pub fn display_node_files(&self, node: Arc<Mutex<TreeNode>>) {
        let node = node.lock().unwrap();
        let childrens = node.childrens.lock().unwrap();

        let mut s = DefaultHasher::new();

        for child in &(*childrens) {
            let mut c = child.lock().unwrap();

            match c.node_type {
                NodeType::File(_) => {
                    println!("{:?} {:?}", c.node_name, c.node_hash);
                }
                NodeType::Dir => {
                    drop(c);
                    self.display_node_files(child.clone());
                }
                _ => {}
            }
        }
    }
}
