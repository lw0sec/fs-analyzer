use fs_analyzer_v2::core::analyzer;

use fs_analyzer_v2::core::fstree::FsTree;

use std::env;

use std::time::{Duration, Instant};

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let fs_0 = &args[1];
    let fs_1 = &args[2];

    let start = Instant::now();
    
    
    let fstree = FsTree::build_from_path(fs_0);
    fstree.analyse_files_type();
    fstree.calc_files_hash();
    //fstree.list_files();
    fstree.analyse_binaries();
    println!("{} dirs and {} files in the tree", fstree.count_dirs(), fstree.count_files());
    
    
    let duration = start.elapsed();
    println!("{:?} to build the fstree", duration);

    /*let tree_data = analyzer::from_path(fs_0);
    tree_data.analyse_tree();
    //tree_data.display_files();

    let duration = start.elapsed();
    println!(
        "{:?} to create the TreeData. {} files",
        duration,
        tree_data.count_files()
    );

    let start = Instant::now();

    let tree_data2 = analyzer::from_path(fs_1);
    tree_data2.analyse_tree();
    //tree_data.display_files();

    let duration = start.elapsed();
    println!(
        "{:?} to create the TreeData. {} files",
        duration,
        tree_data2.count_files()
    );

    let res = tree_data.compare_tree_data(tree_data2);
    res.display_data();
    res.display_count();*/
}
