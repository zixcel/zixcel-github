use std::fs;
use std::process::ExitCode;

use serde::Serialize;
use serde_json::json;
use zixcel_github::{
    build_plan, capabilities, doctor, parse_api_request_json, parse_config, validate_api_request,
    validation_report,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let error = json!({
                "schema": "zixcel://cli-error/v1",
                "connector": zixcel_github::CONNECTOR,
                "status": "error",
                "message": message
            });
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&error)
                    .unwrap_or_else(|_| "{\"status\":\"error\"}".to_owned())
            );
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let command = arguments.next().ok_or_else(usage)?;
    match command.as_str() {
        "doctor" => {
            expect_no_more(&mut arguments)?;
            print_json(&doctor())
        }
        "capabilities" => {
            expect_no_more(&mut arguments)?;
            print_json(&capabilities())
        }
        "validate" => {
            let path = one_path(&mut arguments)?;
            let source = read_config(&path)?;
            let config = parse_config(&source).map_err(|error| error.to_string())?;
            print_json(&validation_report(&config).map_err(|error| error.to_string())?)
        }
        "plan" => {
            let path = one_path(&mut arguments)?;
            let source = read_config(&path)?;
            let config = parse_config(&source).map_err(|error| error.to_string())?;
            print_json(&build_plan(&config).map_err(|error| error.to_string())?)
        }
        "check-configured-request" => {
            let config_path = arguments.next().ok_or_else(usage)?;
            let request_path = arguments.next().ok_or_else(usage)?;
            expect_no_more(&mut arguments)?;
            let config = parse_config(&read_config(&config_path)?).map_err(|e| e.to_string())?;
            let request = parse_api_request_json(
                &fs::read(request_path).map_err(|_| "request file could not be read")?,
            )
            .map_err(|e| e.to_string())?;
            config
                .authorize_request(&request)
                .map_err(|e| e.to_string())?;
            print_json(&validate_api_request(&request).map_err(|e| e.to_string())?)
        }
        "check-request" => {
            let request_path = arguments.next().ok_or_else(usage)?;
            expect_no_more(&mut arguments)?;
            let request = fs::read(request_path)
                .map_err(|_| "API request file could not be read".to_owned())?;
            let request = parse_api_request_json(&request).map_err(|error| error.to_string())?;
            print_json(&validate_api_request(&request).map_err(|error| error.to_string())?)
        }
        _ => Err(usage()),
    }
}

fn one_path(arguments: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let path = arguments.next().ok_or_else(usage)?;
    expect_no_more(arguments)?;
    Ok(path)
}

fn expect_no_more(arguments: &mut impl Iterator<Item = String>) -> Result<(), String> {
    if arguments.next().is_some() {
        Err(usage())
    } else {
        Ok(())
    }
}

fn read_config(path: &str) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|_| "configuration file could not be read as UTF-8 text".to_owned())
}

fn print_json(value: &impl Serialize) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|_| "JSON serialization failed".to_owned())?
    );
    Ok(())
}

fn usage() -> String {
    "usage: zixcel-github <doctor|capabilities|validate <config>|plan <config>|check-request <request.json>|check-configured-request <config> <request.json>>".to_owned()
}
