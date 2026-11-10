use std::{env, error::Error, fs, path::Path};

use rustsheet_wasm::{Engine, EngineOptions, FillOptions, FillRect};
use serde_json::{json, Value};

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("Missing string {key}").into())
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 3 {
        return Err("usage: predict-fill <probes.json> <engine-commit>".into());
    }
    let manifest_path = Path::new(&arguments[1]);
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path)?)?;
    let directory = manifest_path.parent().ok_or("Missing probe directory")?;
    let entries = manifest["probes"].as_array().ok_or("Missing probes")?;
    let options = EngineOptions {
        clock_ms: Some(1_794_268_800_000),
        seed: Some(1),
        timezone: Some("UTC".into()),
        ..EngineOptions::default()
    };
    let mut predictions = Vec::new();
    for entry in entries {
        let mut engine = Engine::new(options.clone())?;
        engine.load_xlsx(&fs::read(directory.join(required(entry, "file")?))?)?;
        let source: FillRect = serde_json::from_value(entry["sourceRect"].clone())?;
        let target: FillRect = serde_json::from_value(entry["targetRect"].clone())?;
        let fill_options: FillOptions = serde_json::from_value(entry["engineOptions"].clone())?;
        let outcome = match engine.fill_plan(
            required(entry, "sheet")?,
            source,
            target,
            fill_options,
        ) {
            Ok(plan) => json!({"status": "plan", "plan": plan}),
            Err(error) => {
                json!({"status": "error", "message": error.to_string(), "debug": format!("{error:?}")})
            }
        };
        predictions.push(json!({"id": required(entry, "id")?, "sha256": required(entry, "sha256")?, "outcome": outcome}));
    }
    let document = json!({
        "schema": "rustsheet.fill-predictions/1",
        "engineCommit": arguments[2],
        "entry": "rustsheet_wasm::Engine::load_xlsx then Engine::fill_plan (native)",
        "engineOptions": options,
        "isExcelObservation": false,
        "predictions": predictions,
    });
    println!("{}", serde_json::to_string_pretty(&document)?);
    Ok(())
}
