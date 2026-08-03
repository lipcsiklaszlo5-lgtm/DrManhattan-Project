use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::abstraction::program::{GeneralizedProgram, AbstractStep, TargetSpec, Cardinality, GridCorner, SpatialRelation};
use manhattan_kernel::sandbox::operators::Transformation;
use manhattan_kernel::predicate::builtin;
use manhattan_kernel::predicate::Predicate;
use std::fs;
use serde_json::Value;

fn main() {
    // 1. Feladat betöltése
    let task_path = "ARC-AGI-master/data/training/05f2a901.json";
    let task_content = fs::read_to_string(task_path).expect("Failed to read task");
    let task: Value = serde_json::from_str(&task_content).expect("Invalid JSON");

    let train: &Vec<Value> = task["train"].as_array().unwrap();
    let train_inputs: Vec<ArcGrid> = train.iter().map(|ex| grid_from_json(&ex["input"])).collect();
    let train_outputs: Vec<ArcGrid> = train.iter().map(|ex| grid_from_json(&ex["output"])).collect();

    // 2. Kézi hipotézisek beolvasása
    let hypotheses_path = "phase0_manual_hypotheses.json";
    let hypotheses_content = fs::read_to_string(hypotheses_path).expect("Failed to read hypotheses file");
    let hypotheses: Vec<Value> = serde_json::from_str(&hypotheses_content).expect("Invalid hypotheses JSON");

    println!("=== PHASE 0/A – Manual Hypotheses Test ===");
    println!("Task: 05f2a901.json ({} train pairs)\n", train_inputs.len());

    for (idx, hyp) in hypotheses.iter().enumerate() {
        let comment = hyp["comment"].as_str().unwrap_or("no comment");
        println!("--- Hypothesis {}: {} ---", idx + 1, comment);

        // Parse
        let condition = Some(parse_predicate(&hyp["condition"]));

        let transformation = match hyp["transformation"].as_str().unwrap_or("") {
            "SemanticTranslateToTarget" => Transformation::SemanticTranslateToTarget,
            "SemanticRecolorToTarget" => Transformation::SemanticRecolorToTarget,
            "SemanticMirrorHorizontal" => Transformation::SemanticMirrorHorizontal,
            "SemanticMirrorVertical" => Transformation::SemanticMirrorVertical,
            "SemanticGravitate" => Transformation::SemanticGravitate,
            other => { eprintln!("  Unknown transformation: {}", other); continue; }
        };

        let target_spec = hyp["target_spec"].as_object().and_then(|ts| {
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
                        _ => { eprintln!("  Unknown relation"); return None; }
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
                _ => { eprintln!("  Unknown TargetSpec"); None }
            }
        });

        let cardinality = match hyp["cardinality"].as_str().unwrap_or("All") {
            "ExactlyOne" => Cardinality::ExactlyOne,
            "AtMostOne" => Cardinality::AtMostOne,
            _ => Cardinality::All,
        };

        let step = AbstractStep { condition, transformation, target_spec, cardinality };
        let program = GeneralizedProgram::new(vec![step], 1.0, train_inputs.len());

        // Futtatás
        let mut covered = 0;
        for (i, (input, expected)) in train_inputs.iter().zip(train_outputs.iter()).enumerate() {
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
        println!("  Coverage: {:.1}% ({}/{})", coverage * 100.0, covered, train_inputs.len());
        println!();
    }

    println!("=== DONE ===");
}

fn grid_from_json(g: &Value) -> ArcGrid {
    let rows: Vec<Vec<u8>> = g.as_array().unwrap().iter()
        .map(|r| r.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect())
        .collect();
    let h = rows.len() as u8;
    let w = if h > 0 { rows[0].len() as u8 } else { 0 };
    ArcGrid { width: w, height: h, pixels: rows.concat() }
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
