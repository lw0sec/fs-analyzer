
use crate::core::tree::TreeData;

pub fn from_path(path: &str) -> TreeData {
    TreeData::new(path)
}