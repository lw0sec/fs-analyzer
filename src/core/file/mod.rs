pub mod elf;
pub mod extension;

#[derive(Debug)]
pub enum FileType {
    Data,
    Text,
    SymLink,
    Sh,

    Elf(Option<elf::ElfData>),
    Driver,
    Pe,

    Header,
    Source,

    Markdown,

    Html,
    XHtml,
    Js,

    Php,
    Cgi,

    Asp,
    Aspx,
}

pub fn check_type(file_name: &str, ptr: &[u8]) -> FileType {
    let mut file_type = FileType::Data;
    elf::check_elf(ptr as *const _ as *const u8, &mut file_type);
    extension::check_extension(file_name, ptr as *const _ as *const u8, &mut file_type);

    match file_type {
        FileType::Php => {
            //*file_type = FileType::Elf;
            // println!("{file_name} == {:?}", file_type);
        }
        _ => {
            //println!("{file_name} == {:?}", file_type);
        }
    }
    file_type
}
