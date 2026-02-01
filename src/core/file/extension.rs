use crate::core::file::FileType;

pub fn check_extension(file_name: &str, ptr: *const u8, file_type: &mut FileType) {
    match file_type {
        FileType::Data => {
            let s: Vec<&str> = file_name.split(".").collect();

            let extension = s.last().unwrap();

            match extension {
                &"txt" => *file_type = FileType::Text,
                &"sh" => *file_type = FileType::Sh,
                &"h" | &"hpp" => *file_type = FileType::Header,
                &"c" | &"cpp" => *file_type = FileType::Source,
                &"ko" => *file_type = FileType::Driver,
                &"md" | &"markdown" => *file_type = FileType::Markdown,
                &"html" | &"htm" => *file_type = FileType::Html,
                &"xhtml" | &"xht" => *file_type = FileType::XHtml,
                &"php" => *file_type = FileType::Php,
                &"js" => *file_type = FileType::Js,
                &"asp" => *file_type = FileType::Asp,
                &"aspx" => *file_type = FileType::Aspx,
                _ => {}
            }
        }
        _ => {}
    }
}
