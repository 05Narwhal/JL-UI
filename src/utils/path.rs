use std::{env, path::PathBuf};

pub fn get_xml_schema_path() -> PathBuf {
    let mut path = env::current_dir().unwrap();

    path.push("src/modules/ui/schema/ui_schema.xml");

    path
}

pub fn get_xml_schema_path_str() -> String {
    get_xml_schema_path().to_string_lossy().to_string()
}