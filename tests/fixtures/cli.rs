//! Offline subprocess fixture. Not included in the portable distribution.
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
        _ => {
            if args.iter().any(|a| a == "-p") {
                println!("{{\"is_error\":false,\"structured_output\":{{\"operations\":[]}}}}");
            } else {
                println!("{{\"operations\":[]}}");
            }
        }
    }
}
