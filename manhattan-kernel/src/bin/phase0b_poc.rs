use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::abstraction::program::{GeneralizedProgram, AbstractStep, TargetSpec, Cardinality, GridCorner, SpatialRelation};
use manhattan_kernel::sandbox::operators::Transformation;
use manhattan_kernel::predicate::builtin;
use manhattan_kernel::predicate::Predicate;
use std::fs;
use serde_json::Value;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--eval" {
        evaluate_response(&args[2]);
    } else {
        generate_prompt();
    }
}

fn generate_prompt() {
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

    let mut grids_text = String::new();
    for (i, (inp, out)) in train_inputs.iter().zip(train_outputs.iter()).enumerate() {
        grids_text.push_str(&format!("Pair {}:\n{}\n->\n{}\n\n", i, grid_to_string(inp), grid_to_string(out)));
    }

    let operators = vec![
        "Predicates: Largest, Smallest, Leftmost, Rightmost, Topmost, Bottommost, ColorEquals(N), UniqueColor, MajorityColor, MinorityColor",
        "Transformations: SemanticTranslateToTarget, SemanticRecolorToTarget, SemanticMirrorHorizontal, SemanticMirrorVertical, SemanticGravitate",
        "TargetSpec types: RelativeToNode(relation: Above/Below/LeftOf/RightOf/TouchingTop/TouchingBottom/TouchingLeft/TouchingRight/CenteredX/CenteredY), Constant(value), GridAnchor(corner: TopLeft/TopRight/BottomLeft/BottomRight), GravitateAnchor(anchor_predicate)",
        "Cardinalities: All, ExactlyOne, AtMostOne",
        "Output ONLY valid JSON (no extra text): {\"steps\":[{\"condition\":{\"type\":\"Largest\"},\"transformation\":\"SemanticTranslateToTarget\",\"target_spec\":{\"type\":\"RelativeToNode\",\"anchor_predicate\":{\"type\":\"Smallest\"},\"relation\":\"TouchingTop\"},\"cardinality\":\"All\"}]}"
    ].join("\n");

    let prompt = format!(
        "You are an ARC task solver. Given the training examples and available operators, generate ONE JSON program that transforms the input grid into the output grid.\n\nGrids (input -> output):\n{}\nGraph diffs (what changed):\n{}\n\nAvailable operators:\n{}",
        grids_text, diff_text, operators
    );

    fs::write("phase0_prompt.txt", &prompt).expect("Failed to write prompt file");
    println!("=== PROMPT WRITTEN TO phase0_prompt.txt ===");
    println!("Copy the content of this file to an LLM chat (e.g., chat.deepseek.com, gemini.google.com).");
    println!("Save the LLM's response to phase0_response.txt.");
    println!("Then run: cargo run --release --bin phase0b_poc -- --eval phase0_response.txt");
}

fn evaluate_response(response_path: &str) {
    let task_path = "ARC-AGI-master/data/training/05f2a901.json";
    let task_content = fs::read_to_string(task_path).expect("Failed to read task");
    let task: Value = serde_json::from_str(&task_content).expect("Invalid JSON");

    let train: &Vec<Value> = task["train"].as_array().unwrap();
    let train_inputs: Vec<ArcGrid> = train.iter().map(|ex| grid_from_json(&ex["input"])).collect();
    let train_outputs: Vec<ArcGrid> = train.iter().map(|ex| grid_from_json(&ex["output"])).collect();

    let response_text = fs::read_to_string(response_path).expect("Failed to read response file");
    println!("=== RAW LLM RESPONSE ===\n{}\n=== END ===", response_text);

    let json_str = if response_text.contains("```json") {
        let start = response_text.find("```json").unwrap() + 7;
        let end = response_text[start..].find("```").unwrap_or(response_text.len());
        &response_text[start..start+end]
    } else if response_text.contains("```") {
        let start = response_text.find("```").unwrap() + 3;
        let end = response_text[start..].find("```").unwrap_or(response_text.len());
        &response_text[start..start+end]
    } else {
        &response_text
    };

    let prog_json: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => { eprintln!("Parse error: {}", e); return; }
    };

    let steps_arr = prog_json["steps"].as_array().expect("Missing steps");

    let mut steps = Vec::new();
    for step in steps_arr {
        let condition = Some(parse_predicate(&step["condition"]));

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
                    let anchor = parse_predicate(&ts["anchor_predicate"]);
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
                "GravitateAnchor" => {
                    let anchor = parse_predicate(&ts["anchor_predicate"]);
                    Some(TargetSpec::GravitateAnchor { anchor_predicate: anchor })
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
    for (i, (input, expected)) in train_inputs.iter().zip(train_outputs.iter()) {
        let ik = ArcAdapter::grid_to_ksg(input);
        let rk = program.apply(&ik, input.width, input.height);
        let pred = ArcAdapter::ksg_to_grid(&rk, input.width, input.height, 0);
        if pred.pixels == expected.pixels {
            covered += 1;
        } else {
            eprintln!("  Pair {}: mismatch (pred {} nodes, exp {} nodes)", i, rk.nodes.len(), ArcAdapter::grid_to_ksg(expected).nodes.len());
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

fn grid_to_string(g: &ArcGrid) -> String {
    (0..g.height).map(|y| {
        let start = (y as usize) * (g.width as usize);
        g.pixels[start..start + g.width as usize].iter()
            .map(|v| v.to_string()).collect::<Vec<_>>().join(",")
    }).collect::<Vec<_>>().join("\n")
}

fn parse_predicate(obj: &Value) -> Box<dyn Predicate> {
    let type_name = obj["type"].as_str().unwrap_or("Largest");
    match type_name {
        "Largest" => Box::new(builtin::LargestPredicate),
        "Smallest" => Box::new(builtin::SmallestPredicate),
        "Leftmost" => Box::new(builtin::LeftmostPredicate),
        "Rightmost" => Box::new(builtin::RightmostPredicate),
        "Topmost" => Box::new(builtin::TopmostPredicate),
        "Bottommost" => Box::new(builtin::BottommostPredicate),
        "UniqueColor" => Box::new(builtin::UniqueColorPredicate),
        "MajorityColor" => Box::new(builtin::MajorityColorPredicate),
        "MinorityColor" => Box::new(builtin::MinorityColorPredicate),
        "ColorEquals" => {
            let color = obj["color"].as_u64().map(|c| c.to_string()).unwrap_or_else(|| "1".to_string());
            Box::new(builtin::ColorPredicate { color })
        },
        _ => { eprintln!("Unknown predicate type '{}', using Largest", type_name); Box::new(builtin::LargestPredicate) }
    }
}
