#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# ===== 1. Új solver modul létrehozása =====
cat > src/solver.rs << 'EOF'
use crate::structure::KernelStructureGraph;
use crate::abstraction::program::{GeneralizedProgram, AbstractStep, Cardinality, TargetSpec};
use crate::semantic_hypothesis::hypothesis::SemanticStep;
use crate::sandbox::operators::Transformation;
use crate::predicate::Predicate;

pub struct Solver {
    pub max_offset_variation: i64,  // mennyit próbáljon eltérni a dx/dy
}

impl Solver {
    pub fn new() -> Self {
        Self { max_offset_variation: 2 }
    }

    /// A SHE által elfogadott lépésekből további finomított programokat generál.
    pub fn refine(
        &self,
        common_steps: &[SemanticStep],
        ksg_pairs: &[(KernelStructureGraph, KernelStructureGraph, u8, u8)],
    ) -> Vec<GeneralizedProgram> {
        let mut new_programs = Vec::new();

        for step in common_steps {
            // Csak a SemanticTranslateToTarget lépésekkel foglalkozunk egyelőre
            if !matches!(step.transformation, Transformation::SemanticTranslateToTarget) {
                continue;
            }

            // Ha a target_spec RelativeToNode, akkor próbálgatjuk az eltolásokat
            if let Some(TargetSpec::RelativeToNode { dx_offset, dy_offset, .. }) = &step.target_spec {
                let orig_dx = *dx_offset;
                let orig_dy = *dy_offset;

                // Variációk: az eredeti körüli kis eltérések
                for dx_v in (orig_dx - self.max_offset_variation)..=(orig_dx + self.max_offset_variation) {
                    for dy_v in (orig_dy - self.max_offset_variation)..=(orig_dy + self.max_offset_variation) {
                        if dx_v == orig_dx && dy_v == orig_dy {
                            continue; // az eredeti már megvan
                        }

                        // Új TargetSpec a variált eltolásokkal
                        let new_spec = TargetSpec::RelativeToNode {
                            condition: step.target_spec.as_ref().unwrap().clone().into_condition(), // klónozzuk a feltételt
                            dx_offset: dx_v,
                            dy_offset: dy_v,
                        };

                        // Létrehozzuk az új lépést
                        let new_step = AbstractStep {
                            condition: step.condition.as_ref().map(|preds| {
                                if preds.len() == 1 {
                                    preds[0].clone_box()
                                } else {
                                    Box::new(crate::predicate::builtin::AndPredicate {
                                        predicates: preds.iter().map(|p| p.clone_box()).collect(),
                                    })
                                }
                            }),
                            transformation: step.transformation.clone(),
                            target_spec: Some(new_spec),
                            cardinality: Cardinality::All,
                        };

                        // Ellenőrizzük, hogy ez a lépés minden train párra működik-e
                        let mut works = true;
                        for (input_ksg, output_ksg, gw, gh) in ksg_pairs {
                            let result_ksg = GeneralizedProgram::apply_step(
                                input_ksg, &new_step, *gw, *gh,
                            );
                            if !crate::semantic_hypothesis::evaluator::step_reproduces_output(
                                &SemanticStep {
                                    condition: new_step.condition.clone().map(|c| {
                                        if let Some(and) = c.as_ref().downcast_ref::<crate::predicate::builtin::AndPredicate>() {
                                            and.predicates.clone()
                                        } else {
                                            vec![c.clone_box()]
                                        }
                                    }),
                                    transformation: new_step.transformation.clone(),
                                    target_spec: new_step.target_spec.clone(),
                                },
                                input_ksg,
                                output_ksg,
                                *gw,
                                *gh,
                            ) {
                                works = false;
                                break;
                            }
                        }

                        if works {
                            let prog = GeneralizedProgram::new(vec![new_step], 1.0, ksg_pairs.len());
                            new_programs.push(prog);
                        }
                    }
                }
            }
        }

        new_programs
    }
}

// Segédfüggvény a TargetSpec klónozásához (a Condition kinyerése)
trait IntoTargetSpec {
    fn into_condition(self) -> Box<crate::abstraction::transform::Condition>;
}

impl IntoTargetSpec for TargetSpec {
    fn into_condition(self) -> Box<crate::abstraction::transform::Condition> {
        match self {
            TargetSpec::RelativeToNode { condition, .. } => condition,
            _ => panic!("Unsupported TargetSpec for Solver"),
        }
    }
}
EOF

# ===== 2. Regisztrálás lib.rs-ben =====
if ! grep -q "pub mod solver;" src/lib.rs; then
    echo "pub mod solver;" >> src/lib.rs
fi

# ===== 3. MetaLearner.finalize() kiegészítése a Solver hívásával =====
python3 << 'PYEOF'
meta_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/meta_learner.rs"
with open(meta_path, 'r') as f:
    content = f.read()

# Beszúrjuk a Solver hívást a finalize végére, a consolidate elé
old_consolidate = "        self.program_synthesizer.consolidate();"
new_consolidate = """        // Module 3: Solver – finomhangolás a SHE kimenetén
        let solver = crate::solver::Solver::new();
        let refined_programs = solver.refine(&common_steps, &ksg_pairs);
        for prog in refined_programs {
            self.program_synthesizer.generalized_programs.push(prog);
        }

        self.program_synthesizer.consolidate();"""

content = content.replace(old_consolidate, new_consolidate)
with open(meta_path, 'w') as f:
    f.write(content)
print("Solver integrated into finalize().")
PYEOF

# ===== 4. Build & teszt =====
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1
echo "===== COMMIT ====="
git add -A && git commit -m "feat: add Module 3 Solver for deterministic parameter refinement" && git push
