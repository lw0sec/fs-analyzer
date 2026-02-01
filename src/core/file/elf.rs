use crate::core::node::TreeNode;

use crate::core::fstree::node::Node;

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use crate::core::file::FileType;
use crate::core::tree::TreeData;

use xmas_elf::symbol_table::Binding;
use xmas_elf::symbol_table::DynEntry32;
use xmas_elf::symbol_table::Entry;
use xmas_elf::ElfFile;
use xmas_elf::{header, program, program::Flags, sections, sections::SectionHeader};

use std::sync::{Arc, Mutex, RwLock};

use log::warn;
// #[repr(C)]
// pub struct ElfHeader {
//     // Magic value 0x7fELF
//     pub magic: [u8; 4],
//     pub arch_bits: u8,
//     pub endianess: u8,
//     pub header_version: u8,
//     pub abi: u8,
//     pub version_abi: u8,
//     pub _a: [u8; 6],
//     pub ident_size: u8,
//     pub file_type: u16,
//     pub arch: u16,
// }

pub struct ElfCmpResult {}

#[derive(Debug)]
pub struct ElfData {
    //pub libs: Vec<String>,
    pub size: u64,
    pub dyn_libs: HashMap<String, Node>,
    pub dyn_funcs: Vec<String>,
}

impl ElfData {
    pub fn new() -> Self {
        Self {
            size: 0,
            dyn_libs: HashMap::new(),
            dyn_funcs: Vec::new(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ElfHeader32 {
    // Magic value : ELF + 0x7f
    pub magic: [u8; 4],

    // 0 = None
    // 1 = 32 bits
    // 2 = 64 bits
    pub class: u8,

    // 0 = None
    // 1 = LSB (little)
    // 2 = MSB (big)
    pub endianess: u8,

    // Should be equal to 1
    pub header_version: u8,

    // ABI
    // 3 = Linux
    // 64 = ARM EABI
    // 97 = ARM
    pub abi: u8,

    pub abi_version: u8,

    // Reserved
    pub reserved: [u8; 7],

    // 0 = Unknown
    // 1 = Relocatable file
    // 2 = Executable file
    // 3 = Shared object
    // 4 = Core file
    pub file_type: u16,

    // Target architecture
    // 7 = Intel i860
    // 40 = ARM
    // 50 = Intel IA64
    // 62 = X64
    // 243 = RISC-V
    pub machine: u16,

    // Should be equal to 1
    pub version: u32,

    // Entry point
    // 0 if no entrypoint
    pub entry_point: u32,

    // Offset of the program header table
    pub ph_offset: u32,

    // Offset of the section header table
    pub sh_offset: u32,

    // Flags
    pub flags: u32,

    // Size of this header
    pub header_size: u16,

    // Size of a program header table entry
    pub ph_entry_size: u16,

    // Number of entries in the program header table
    pub ph_num: u16,

    // Size of a section heade table entry
    pub sh_entry_size: u16,

    // Number of entries in the section header table
    pub sh_num: u16,

    // Index of the entry in the section table header that contains the section names
    pub sh_str_index: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct ElfHeader {
    // Magic value : ELF + 0x7f
    pub magic: [u8; 4],

    // 0 = None
    // 1 = 32 bits
    // 2 = 64 bits
    pub class: u8,

    // 0 = None
    // 1 = LSB (little)
    // 2 = MSB (big)
    pub endianess: u8,

    // Should be equal to 1
    pub header_version: u8,

    // ABI
    // 3 = Linux
    // 64 = ARM EABI
    // 97 = ARM
    pub abi: u8,

    pub abi_version: u8,

    // Reserved
    pub reserved: [u8; 7],

    // 0 = Unknown
    // 1 = Relocatable file
    // 2 = Executable file
    // 3 = Shared object
    // 4 = Core file
    pub file_type: u16,

    // Target architecture
    // 7 = Intel i860
    // 40 = ARM
    // 50 = Intel IA64
    // 62 = X64
    // 243 = RISC-V
    pub machine: u16,

    // Should be equal to 1
    pub version: u32,

    // Entry point
    // 0 if no entrypoint
    pub entry_point: usize,

    // Offset of the program header table
    pub ph_offset: usize,

    // Offset of the section header table
    pub sh_offset: usize,

    // Flags
    pub flags: u32,

    // Size of this header
    pub header_size: u16,

    // Size of a program header table entry
    pub ph_entry_size: u16,

    // Number of entries in the program header table
    pub ph_num: u16,

    // Size of a section heade table entry
    pub sh_entry_size: u16,

    // Number of entries in the section header table
    pub sh_num: u16,

    // Index of the entry in the section table header that contains the section names
    pub sh_str_index: u16,
}

pub fn check_elf(ptr: *const u8, file_type: &mut FileType) {
    unsafe {
        if *(ptr as *const u32) == 0x464c457f {
            let elf_header = ptr as *const ElfHeader;

            // Valid the elf file
            if (*elf_header).class <= 2
                && (*elf_header).class >= 0
                && (*elf_header).endianess <= 2
                && (*elf_header).endianess >= 0
            {
                match file_type {
                    FileType::Data => {
                        *file_type = FileType::Elf(None);
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn simple_elf_cmp(ptr1: *const u8, ptr2: *const u8) -> bool {
    // Return false if same
    let mut res = false;

    let elf_header1 = ptr1 as *const ElfHeader;
    let elf_header2 = ptr2 as *const ElfHeader;

    // unsafe { println!("{}", (*elf_header1).class) };

    // 32 bits
    if unsafe { (*elf_header1).class == 1 } {
        let elf_header1 = ptr1 as *const ElfHeader32;
        let elf_header2 = ptr2 as *const ElfHeader32;

        if unsafe {
            (*elf_header1).ph_num != (*elf_header2).ph_num
                || (*elf_header1).sh_num != (*elf_header2).sh_num
        } {
            res = true;
        }
    } else if unsafe { (*elf_header1).class == 2 } {
        // unsafe {
        //     println!(
        //         "elf_header1: header_size={} ph_num={} sh_num={}",
        //         (*elf_header1).header_size,
        //         (*elf_header1).ph_num,
        //         (*elf_header1).sh_num,
        //     );

        //     println!(
        //         "elf_header2: header_size={} ph_num={} sh_num={}",
        //         (*elf_header2).header_size,
        //         (*elf_header2).ph_num,
        //         (*elf_header2).sh_num
        //     );
        // }
    }

    // unsafe {
    //     println!("{:?}", *(ptr1 as *const u32));
    //     println!("{:?}", *(ptr2 as *const u32));
    // }
    res
}

pub fn analyse_dynamic(
    tree_data: &TreeData,
    elf: &ElfFile,
    section: SectionHeader,
    elf_data: &mut ElfData,
) {
    //println!("Analyse ELF");
    // Get data from dynamic table
    let data = section.get_data(&elf).unwrap();

    match data {
        sections::SectionData::Dynamic32(entries) => {
            // Check all entries
            for entry in entries {
                // If the type is needed check if a string is found
                if entry.get_tag().unwrap() == xmas_elf::dynamic::Tag::Needed {
                    let res = elf.get_dyn_string(entry.get_val().unwrap());

                    match res {
                        Ok(v) => {
                            //let nodes = tree_data.find_nodes_by_name(v);
                            //elf_data.libs.insert(v.to_string());
                        }
                        Err(_) => {}
                    }
                    //println!("{:?}", res);
                }
            }
        }
        _ => {}
    }
}

pub fn analyse_elf(tree_data: &TreeData, path: &str) -> ElfData {
    let mut elf_data = ElfData::new();

    let file = File::open(path).unwrap();
    let file_size = file.metadata().unwrap().len();
    elf_data.size = file_size;

    let mut binary_data = std::fs::read(path).unwrap();
    let elf = ElfFile::new(&mut binary_data).unwrap();

    for section in elf.section_iter() {
        match section.get_type().unwrap() {
            sections::ShType::DynSym => {
                //analyse_dyn_sym(tree_data, &elf, section, &mut node_elf_data)
            }
            sections::ShType::Dynamic => analyse_dynamic(tree_data, &elf, section, &mut elf_data),
            _ => {}
        }
    }

    elf_data
}

pub fn get_dynamic_libs(elf: &ElfFile, section: SectionHeader) -> Vec<String> {
    let mut libs: Vec<String> = Vec::new();

    let data_section = section.get_data(&elf).unwrap();

    match data_section {
        sections::SectionData::Dynamic32(entries) => {
            for entry in entries {
                if entry.get_tag().unwrap() == xmas_elf::dynamic::Tag::Needed {
                    let res = elf.get_dyn_string(entry.get_val().unwrap());
                    match res {
                        Ok(v) => libs.push(v.to_string()),
                        Err(_) => {}
                    }
                }
            }
        }
        sections::SectionData::Dynamic64(entries) => {
            for entry in entries {
                if entry.get_tag().unwrap() == xmas_elf::dynamic::Tag::Needed {
                    let res = elf.get_dyn_string(entry.get_val().unwrap() as u32);
                    match res {
                        Ok(v) => libs.push(v.to_string()),
                        Err(_) => {}
                    }
                }
            }
        }
        _ => {}
    };

    //println!("{:?}", libs);
    libs
}

fn get_dyn_func(elf: &ElfFile, section: SectionHeader) -> Vec<String> {
    let mut dyn_funcs: Vec<String> = Vec::new();

    let data = section.get_data(&elf).unwrap();

    match data {
        sections::SectionData::DynSymbolTable32(entries) => {
            for entry in entries {
                let name = entry.get_name(&elf).unwrap();
                let binding = entry.get_binding().unwrap();
                let entry_type = entry.get_type().unwrap();

                //println!("{} {:?} {:?}", name, binding, entry_type);
                
                /*match binding {
                    Binding::Global => {
                        println!("{} {:?} {:?}", name, binding, entry_type);
                    }
                    _ => {}
                }*/
            }
        }
        _ => {}
    };
    
    dyn_funcs
}

pub fn analyse_elf2(head_node: Node, path: &str) -> ElfData {
    let mut elf_data = ElfData::new();

    // Get size of file
    let metadata = fs::metadata(path).unwrap();
    elf_data.size = metadata.len();

    // Open file using xmas_elf
    let mut binary_data = std::fs::read(path).unwrap();
    let elf = ElfFile::new(&mut binary_data).unwrap();

    let mut dyn_libs: Vec<String> = Vec::new();
    let mut dyn_funcs: Vec<String> = Vec::new();

    for section in elf.section_iter() {
        match section.get_type().unwrap() {
            /*
                Get list of dynamic symbols
            */
            sections::ShType::DynSym => {
                dyn_funcs = get_dyn_func(&elf, section);
            }
            sections::ShType::SymTab => {
                let data = section.get_data(&elf).unwrap();
                match data {
                    sections::SectionData::SymbolTable32(entries) => {
                        /*for entry in entries {

                            let name = entry.get_name(&elf).unwrap();
                            let binding = entry.get_binding().unwrap();
                            let entry_type = entry.get_type().unwrap();
                            //println!("{} {:?}", name, binding);

                            match binding {
                                Binding::Global => {
                                    println!("{} {:?} {:?}", name, binding, entry_type);
                                }
                                _ => {
                                }
                            }
                            /*let res = entry.get_name(&elf);
                            match res {
                                Ok(v) => println!("{:?}", res),
                                Err(_) => {}
                            }*/
                        }*/
                    }
                    _ => {}
                }
            }
            /*
                Get list of dynamic libraries
            */
            sections::ShType::Dynamic => {
                dyn_libs = get_dynamic_libs(&elf, section);
            }
            _ => {}
        }
    }
    
    /*
        Create dynamic libraries hash map
    */
    let mut dyn_libs_map: HashMap<String, Node> = HashMap::new();
    for dyn_lib in dyn_libs {
        let node_list = head_node.find_node_by_name_rec(&dyn_lib);
        //println!("{} {}", dyn_lib, node_list.len());
        if node_list.len() == 1 {
            //println!("{} {}", dyn_lib, node_list.len());
            dyn_libs_map.insert(dyn_lib, node_list[0].clone());
        }
        else if node_list.len() == 0 {
            warn!("Dynamic library {} not found", dyn_lib);
        }
    }
    
    elf_data.dyn_libs = dyn_libs_map;
    elf_data.dyn_funcs = dyn_funcs;
    
    elf_data
}
