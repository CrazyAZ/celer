use std::path::PathBuf;
use serde_json::json;
use celer::core;
use super::ErrorState;
use super::file;

pub fn load_unbundled_route_with_metadata(out_errors: &mut ErrorState) -> (serde_json::Value, core::Metadata) {
    let combined_json = load_unbundled_route(out_errors);

    let source_metadata = match combined_json.get("_project") {
        Some(metadata_value) => core::Metadata::from(metadata_value),
        None => core::Metadata::new()
    };

    (combined_json, source_metadata)
}

pub fn load_unbundled_route(out_errors: &mut ErrorState) -> serde_json::Value {
    let mut paths: Vec<PathBuf> = Vec::new();

    super::scan_for_celer_files(&mut paths, out_errors);

    let mut combined_json = json!({});
    for p in paths {
        let file_content = match std::fs::read_to_string(&p) {
            Ok(v) => v,
            Err(e) => {
                out_errors.add(format!("{}", p.display()), format!("Cannot read file: {}", e));
                continue
            }
        };
        let file_json: serde_json::Value = match load_yaml_object(&file_content) {
            Ok(file_json) => file_json,
            Err(e) => {
                out_errors.add(format!("{}", p.display()), format!("Error loading object: {}", e));
                continue
            }
        };
        for (k, v) in file_json.as_object().unwrap() {
            combined_json[k.to_string()] = v.clone();
        }
    }

    combined_json
}

fn load_yaml_object(yaml_str: &str) -> Result<serde_json::Value, String> {
    let yaml_result: serde_yaml::Result<serde_json::Value> = serde_yaml::from_str(yaml_str);
    match yaml_result{
        Ok(value) => {
            if value.is_object() {
                return Ok(value);
            }
            Err(String::from("Yaml must be an object"))
        },
        Err(e) => {
            Err(format!("Yaml load error: {}", e))
        }
    }
}

pub fn add_bundle_errors(bundle_errors: &[core::BundlerError], out_errors: &mut ErrorState) {
    for error in bundle_errors {
        out_errors.add("bundle emitted".to_string(), format!("In {} {}: {}", error.location_type(), error.location_name(), error.message()))
    }
}

pub fn write_bundle_json(bundle: &core::SourceObject, pretty: bool, out_errors: &mut ErrorState) {
    file::write_json_file(&bundle.to_json(), "bundle.json", pretty, out_errors);
}

pub fn write_bundle_yaml(bundle: &core::SourceObject, out_errors: &mut ErrorState) {
    file::write_yaml_file(&bundle.to_json(), "bundle.yaml", out_errors);
}
