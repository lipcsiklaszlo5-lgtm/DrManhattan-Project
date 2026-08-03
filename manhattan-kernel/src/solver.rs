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
