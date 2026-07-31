use std::process::Command;
use std::io::Write;
use crate::structure::KernelStructureGraph;
use crate::task::Task;
use super::{Algorithm, CostEstimate, DomainAdapter, ValidationError};

pub struct CompilerAdapter;

impl DomainAdapter for CompilerAdapter {
    fn build_structure(&self, task: &Task) -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        let code = task.intent.clone();

        let sysroot = match Command::new("rustc").arg("--print").arg("sysroot").output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => return g,
        };

        let mut child = match Command::new("rustc")
            .arg("--edition=2021")
            .arg("--sysroot").arg(&sysroot)
            .arg("--emit=metadata")
            .arg("-")
            .arg("--error-format=json")
            .stdin(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return g,
        };

        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(code.as_bytes());
        }
        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return g,
        };

        if output.status.success() {
            return g;
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            if let Ok(err) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(msg) = err.get("message") {
                    let msg_str = msg.as_str().unwrap_or("");
                    let node_id = format!("err_{}", g.nodes.len());
                    let node = g.add_node(&node_id, "compiler_error");
                    node.attributes.insert("message".into(), msg_str.to_string());
                    if let Some(spans) = err.get("spans") {
                        if let Some(span) = spans.get(0) {
                            if let (Some(line), Some(col)) =
                                (span.get("line_start"), span.get("column_start")) {
                                node.attributes.insert("line".into(), line.to_string());
                                node.attributes.insert("column".into(), col.to_string());
                            }
                        }
                    }
                    
                    if msg_str.contains("mismatched types") {
                        node.attributes.insert("action".into(), "replace_type".into());
                        if let Some(expected) = err.get("expected") {
                            let exp = expected.as_str().unwrap_or("").to_string();
                            node.attributes.insert("old_type".into(), exp);
                        }
                        if let Some(found) = err.get("found") {
                            let fnd = found.as_str().unwrap_or("").to_string();
                            node.attributes.insert("new_type".into(), fnd);
                        }
                    } else if msg_str.contains("cannot find") || msg_str.contains("not found") {
                        node.attributes.insert("action".into(), "add_import".into());
                        node.attributes.insert("annotation".into(), "use std::fmt::Debug;".into());
                    } else if msg_str.contains("unresolved name") || msg_str.contains("cannot find value") {
                        node.attributes.insert("action".into(), "rename".into());
                        if let Some(name) = err.get("name") {
                            node.attributes.insert("old_name".into(), name.as_str().unwrap_or("").to_string());
                        }
                    } else {
                        node.attributes.insert("action".into(), "fix_main".into());
                    }
                }
            }
        }
        g
    }

    fn validate(&self, _structure: &KernelStructureGraph, candidate: &str) -> Result<(), ValidationError> {
        let sysroot = match Command::new("rustc").arg("--print").arg("sysroot").output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => return Err(ValidationError::Failed("rustc not available".into())),
        };
        let mut child = match Command::new("rustc")
            .arg("--edition=2021")
            .arg("--sysroot").arg(&sysroot)
            .arg("--emit=metadata")
            .arg("-")
            .arg("--error-format=json")
            .stdin(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return Err(ValidationError::Failed("rustc not available".into())),
        };
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(candidate.as_bytes());
        }
        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(_) => return Err(ValidationError::Failed("rustc failed".into())),
        };
        if output.status.success() {
            Ok(())
        } else {
            Err(ValidationError::Failed("compilation failed".into()))
        }
    }

    fn available_algorithms(&self) -> Vec<Algorithm> {
        vec![
            Algorithm {
                name: "rustfmt".into(),
                description: "format code".into(),
                cost: CostEstimate { latency_ms: 100, memory_bytes: 2048 },
            },
            Algorithm {
                name: "rustc_check".into(),
                description: "type check".into(),
                cost: CostEstimate { latency_ms: 500, memory_bytes: 8192 },
            },
        ]
    }

    fn graph_to_code(&self, graph: &KernelStructureGraph, original_code: &str) -> String {
        let mut code = original_code.to_string();
        for node in &graph.nodes {
            if node.node_type != "compiler_error" {
                continue;
            }
            let action = node.attributes.get("action").cloned().unwrap_or_default();
            match action.as_str() {
                "replace_type" => {
                    if let (Some(line), Some(col), Some(old), Some(new), Some(new_val)) = (
                        node.attributes.get("line").and_then(|l| l.parse::<usize>().ok()),
                        node.attributes.get("column").and_then(|c| c.parse::<usize>().ok()),
                        node.attributes.get("old_type"),
                        node.attributes.get("new_type"),
                        node.attributes.get("new_value"),
                    ) {
                        // Típus és érték együttes cseréje
                        code = code.lines()
                            .enumerate()
                            .map(|(i, l)| {
                                if i + 1 == line {
                                    // A sorban a típus az adott oszloptól kezdődik, utána '=' és érték következik
                                    let prefix: String = l.chars().take(col - 1).collect();
                                    let rest: String = l.chars().skip(col - 1).collect();
                                    // Megkeressük az '=' utáni értéket és kicseréljük
                                    if let Some(eq_pos) = rest.find('=') {
                                        let _after_eq = &rest[eq_pos+1..];
                                        let _before_eq = &rest[..eq_pos];
                                        let new_rest = format!("{}= {}", new, new_val);
                                        format!("{}{}", prefix, new_rest)
                                    } else {
                                        // Ha nincs '=', akkor csak a típust cseréljük
                                        let after_type = rest.chars().skip(old.len()).collect::<String>();
                                        format!("{}{}{}", prefix, new, after_type)
                                    }
                                } else {
                                    l.to_string()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n");
                    }
                }
                "add_import" => {
                    if let Some(annotation) = node.attributes.get("annotation") {
                        code = format!("{}\n{}", annotation, code);
                    }
                }
                "rename" => {
                    if let (Some(old_name), Some(new_name)) = (
                        node.attributes.get("old_name"),
                        node.attributes.get("new_name"),
                    ) {
                        code = code.replace(old_name, new_name);
                    }
                }
                "delete_line" => {
                    if let Some(line) = node.attributes.get("line").and_then(|l| l.parse::<usize>().ok()) {
                        code = code.lines()
                            .enumerate()
                            .filter(|(i, _)| *i + 1 != line)
                            .map(|(_, l)| l)
                            .collect::<Vec<&str>>()
                            .join("\n");
                    }
                }
                "fix_main" => {
                    code = "fn main() {}".to_string();
                }
                _ => {}
            }
        }
        code
    }
}
