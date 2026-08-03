use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::abstraction::program::{GeneralizedProgram, AbstractStep, TargetSpec, Cardinality, GridCorner, SpatialRelation};
use manhattan_kernel::sandbox::operators::Transformation;
use manhattan_kernel::predicate::builtin;
use manhattan_kernel::predicate::Predicate;
use std::fs;
use serde_json::{Value, json};

fn main() {
    let task_path = "ARC-AGI-master/data/training/05f2a901.json";
    let task_content = fs::read_to_string(task_path).expect("Failed to read task");
    let task: Value = serde_json::from_str(&task_content).expect("Invalid JSON");

    let train: &Vec<Value> = task["train"].as_array().unwrap();
    let train_inputs: Vec<ArcGrid> = train.iter().map(|ex| grid_from_json(&ex["input"])).collect();
    let train_outputs: Vec<ArcGrid> = train.iter().map(|ex| grid_from_json(&ex["output"])).collect();

    let mut diff_text = String::new();
    for (i, (input, output)) in train_inputs.iter().zip(train_outputs.iter()).enumerate() {
        let ik = ArcAdapter::grid_to_ksg(input);
        let ok = ArcAdapter::grid_to_ksg(output);
        let diffs = manhattan_kernel::structure::topology::graph_diff(&ik, &ok);
        diff_text.push_str(&format!("Pair {}: {:?}\n", i, diffs));
    }

    let prompt = format!(
        "Available predicates: Largest, Smallest, Leftmost, Rightmost, Topmost, Bottommost, ColorEquals(N), UniqueColor, MajorityColor, MinorityColor. Available transformations: SemanticTranslateToTarget, SemanticRecolorToTarget, SemanticMirrorHorizontal, SemanticMirrorVertical, SemanticGravitate. TargetSpec types: RelativeToNode(relation: Above/Below/LeftOf/RightOf/TouchingTop/TouchingBottom/TouchingLeft/TouchingRight/CenteredX/CenteredY), Constant(value), GridAnchor(corner: TopLeft/TopRight/BottomLeft/BottomRight), GravitateAnchor(anchor_predicate). Cardinalities: All, ExactlyOne, AtMostOne.\n\nGraph diffs:\n{}\n\nOutput ONLY valid JSON, no extra text: {{\"steps\":[{{\"condition\":{{\"type\":\"Largest\"}},\"transformation\":\"SemanticTranslateToTarget\",\"target_spec\":{{\"type\":\"RelativeToNode\",\"anchor_predicate\":{{\"type\":\"Smallest\"}},\"relation\":\"TouchingTop\"}},\"cardinality\":\"All\"}}]}}",
        diff_text
    );

    // Use gemini-3.6-flash by default, or GEMINI_MODEL env var
    let model_name = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.6-flash".to_string());
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let client = reqwest::blocking::Client::new();
    let resp = client.post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": model_name,
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()
        .expect("API request failed");

    let resp_text = resp.text().expect("Cannot read response body");
    println!("=== FULL API RESPONSE ===\n{}\n=== END API RESPONSE ===", resp_text);

    let body: Value = match serde_json::from_str(&resp_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse API response JSON: {}", e);
            return;
        }
    };

    let content = body["choices"][0]["message"]["content"].as_str().unwrap_or("");
    println!("=== LLM GENERATED PROGRAM ===\n{}\n=== END PROGRAM ===", content);

    if content.is_empty() {
        eprintln!("Empty response content. Cannot continue.");
        return;
    }

    let prog_json: Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(e) => { eprintln!("Parse error: {}", e); return; }
    };
    let steps_arr = prog_json["steps"].as_array().expect("Missing steps");

    let mut steps = Vec::new();
    for step in steps_arr {
        let condition = step["condition"].as_object().and_then(|c| {
            Some(parse_predicate(c["type"].as_str()?))
        });

        let transformation = match step["transformation"].as_str().unwrap_or("") {
            "SemanticTranslateToTarget" => Transformation::SemanticTranslateToTarget,
            "SemanticRecolorToTarget" => Transformation::SemanticRecolorToTarget,
            "SemanticMirrorHorizontal" => Transformation::SemanticMirrorHorizontal,
            "SemanticMirrorVertical" => Transformation::SemanticMirrorVertical,
            "SemanticGravitate" => Transformation::SemanticGravitate,
            other => { eprintln!("Unknown transformation: {}", other); continue; }
        };

        let target_spec = step["target_spec"].as_object().and_then(|ts| {
            let ts_type = ts["type"].as_str()?;
            match ts_type {
                "RelativeToNode" => {
                    let anchor = parse_predicate(ts["anchor_predicate"]["type"].as_str()?);
                    let relation = match ts["relation"].as_str()? {
                        "Above" => SpatialRelation::Above,
                        "Below" => SpatialRelation::Below,
                        "LeftOf" => SpatialRelation::LeftOf,
                        "RightOf" => SpatialRelation::RightOf,
                        "TouchingTop" => SpatialRelation::TouchingTop,
                        "TouchingBottom" => SpatialRelation::TouchingBottom,
                        "TouchingLeft" => SpatialRelation::TouchingLeft,
                        "TouchingRight" => SpatialRelation::TouchingRight,
                        "CenteredX" => SpatialRelation::CenteredX,
                        "CenteredY" => SpatialRelation::CenteredY,
                        _ => { eprintln!("Unknown relation"); return None; }
                    };
                    Some(TargetSpec::RelativeToNode {
                        condition: Box::new(manhattan_kernel::abstraction::transform::Condition::Predicate(anchor)),
                        relation,
                    })
                },
                _ => { eprintln!("Unknown TargetSpec"); None }
            }
        });

        let cardinality = match step["cardinality"].as_str().unwrap_or("All") {
            "ExactlyOne" => Cardinality::ExactlyOne,
            "AtMostOne" => Cardinality::AtMostOne,
            _ => Cardinality::All,
        };

        steps.push(AbstractStep { condition, transformation, target_spec, cardinality });
    }

    if steps.is_empty() {
        eprintln!("No steps parsed.");
        return;
    }

    let program = GeneralizedProgram::new(steps, 1.0, train_inputs.len());

    let mut covered = 0;
    for (input, expected) in train_inputs.iter().zip(train_outputs.iter()) {
        let ik = ArcAdapter::grid_to_ksg(input);
        let rk = program.apply(&ik, input.width, input.height);
        let pred = ArcAdapter::ksg_to_grid(&rk, input.width, input.height, 0);
        if pred.pixels == expected.pixels {
            covered += 1;
        }
    }

    let coverage = covered as f64 / train_inputs.len() as f64;
    println!("\n=== RESULT ===");
    println!("Parse: OK");
    println!("Coverage: {:.1}% ({}/{})", coverage * 100.0, covered, train_inputs.len());
}

fn grid_from_json(g: &Value) -> ArcGrid {
    let rows: Vec<Vec<u8>> = g.as_array().unwrap().iter()
        .map(|r| r.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect())
        .collect();
    let h = rows.len() as u8;
    let w = if h > 0 { rows[0].len() as u8 } else { 0 };
    ArcGrid { width: w, height: h, pixels: rows.concat() }
}

fn parse_predicate(name: &str) -> Box<dyn Predicate> {
    match name {
        "Largest" => Box::new(builtin::LargestPredicate),
        "Smallest" => Box::new(builtin::SmallestPredicate),
        "Leftmost" => Box::new(builtin::LeftmostPredicate),
        "Rightmost" => Box::new(builtin::RightmostPredicate),
        "Topmost" => Box::new(builtin::TopmostPredicate),
        "Bottommost" => Box::new(builtin::BottommostPredicate),
        "UniqueColor" => Box::new(builtin::UniqueColorPredicate),
        "MajorityColor" => Box::new(builtin::MajorityColorPredicate),
        "MinorityColor" => Box::new(builtin::MinorityColorPredicate),
        _ if name.starts_with("ColorEquals(") => {
            let color = name.trim_start_matches("ColorEquals(").trim_end_matches(")").to_string();
            Box::new(builtin::ColorPredicate { color })
        },
        _ => { eprintln!("Unknown predicate '{}', using Largest", name); Box::new(builtin::LargestPredicate) }
    }
}
