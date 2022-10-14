#![allow(dead_code)]
//I am extremely lazy, so
fn add_path(string: &str) -> String {
    format!("./path/to/folder/{}.nobl", string)
}
pub fn db() -> [String; 3] {
    ["data", "doota", "template"]
        .map(|file_name|add_path(file_name))
}