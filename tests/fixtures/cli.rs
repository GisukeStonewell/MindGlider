//! Offline subprocess fixture. Not included in the portable distribution.
use serde_json::{Value, json};
use std::io::Read;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args
        .iter()
        .any(|a| a == "app-server" || a == "--input-format")
    {
        use std::io::{BufRead, Write};
        let mut input = std::io::stdin().lock().lines();
        let request: serde_json::Value =
            serde_json::from_str(&input.next().unwrap().unwrap()).unwrap();
        if args.iter().any(|a| a == "app-server") {
            assert_eq!(request["method"], "initialize");
            println!("{{\"id\":0,\"result\":{{}}}}");
            std::io::stdout().flush().unwrap();
            let notify: serde_json::Value =
                serde_json::from_str(&input.next().unwrap().unwrap()).unwrap();
            assert_eq!(notify["method"], "initialized");
            let request: serde_json::Value =
                serde_json::from_str(&input.next().unwrap().unwrap()).unwrap();
            assert_eq!(request["method"], "model/list");
            println!(
                "{{\"id\":1,\"result\":{{\"data\":[{{\"model\":\"fixture-model\",\"isDefault\":true,\"supportedReasoningEfforts\":[{{\"reasoningEffort\":\"high\"}}]}}],\"nextCursor\":null}}}}"
            );
        } else {
            assert_eq!(request["request"]["subtype"], "initialize");
            println!(
                "{{\"type\":\"control_response\",\"response\":{{\"subtype\":\"success\",\"request_id\":\"init\",\"response\":{{\"models\":[{{\"value\":\"fixture-model\",\"supportedEffortLevels\":[\"high\"]}}]}}}}}}"
            );
        }
        return;
    }
    let mode = args
        .windows(2)
        .find(|a| a[0] == "--model")
        .map(|a| a[1].as_str())
        .unwrap_or("ok");
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    if mode == "custom-prompt" && !input.starts_with("CUSTOM FIXED PROMPT") {
        eprintln!("custom fixed prompt missing");
        std::process::exit(2);
    }
    std::thread::sleep(std::time::Duration::from_millis(30));
    match mode {
        "timeout" => {
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
        "auth-error" => {
            eprintln!("fixture: authentication required");
            std::process::exit(1);
        }
        "invalid" => {
            println!("not json");
        }
        "provider-rejection" => {
            println!(r#"{{"is_error":true,"result":"HTTP 429 Too Many Requests"}}"#);
        }
        "title-ok" | "title-result-envelope" => {
            let schema = if args.iter().any(|arg| arg == "-p") {
                option_value(&args, "--json-schema").to_owned()
            } else {
                std::fs::read_to_string(option_value(&args, "--output-schema")).unwrap()
            };
            let schema: Value = serde_json::from_str(&schema).unwrap();
            assert_eq!(schema["type"], "object");
            assert_eq!(schema["required"], json!(["title"]));
            assert_eq!(schema["additionalProperties"], false);
            assert_eq!(schema["properties"].as_object().unwrap().len(), 1);
            assert_eq!(schema["properties"]["title"]["type"], "string");
            let response = json!({"title": "改善後のタイトル"});
            if mode == "title-result-envelope" {
                println!(
                    "{}",
                    json!({"is_error": false, "result": response.to_string()})
                );
            } else {
                write_title_response(&args, response);
            }
        }
        "title-extra-control" => {
            write_title_response(
                &args,
                json!({"title": "new title", "id": "outside", "op": "delete"}),
            );
        }
        "title-old-operation" => {
            write_title_response(
                &args,
                json!({"operations": [{"op": "update", "id": "outside", "text": "new title",
                    "note": null, "details": null}]}),
            );
        }
        "title-blank" => write_title_response(&args, json!({"title": " \t "})),
        "title-multiline" => write_title_response(&args, json!({"title": "line one\nline two"})),
        _ => {
            if args.iter().any(|a| a == "-p") {
                println!("{{\"is_error\":false,\"structured_output\":{{\"operations\":[]}}}}");
            } else {
                println!("{{\"operations\":[]}}");
            }
        }
    }
}

fn option_value<'a>(args: &'a [String], option: &str) -> &'a str {
    &args.windows(2).find(|pair| pair[0] == option).unwrap()[1]
}

fn write_title_response(args: &[String], response: Value) {
    if args.iter().any(|arg| arg == "-p") {
        println!(
            "{}",
            json!({"is_error": false, "structured_output": response})
        );
    } else {
        println!("{response}");
    }
}
