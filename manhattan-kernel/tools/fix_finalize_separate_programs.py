import pathlib

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")
meta_path = ROOT / "src" / "meta_learner.rs"

with open(meta_path, 'r') as f:
    content = f.read()

old_block = """        // A generate_common_steps már validálta a lépéseket, itt csak átvesszük őket
        let validated_steps = common_steps;

        // Convert validated steps to GeneralizedProgram
        if !validated_steps.is_empty() {
            let abstract_steps: Vec<crate::abstraction::program::AbstractStep> = validated_steps.into_iter().map(|s| {
                use crate::abstraction::program::{AbstractStep, Cardinality};
                AbstractStep {
                    condition: s.condition.map(|preds| {
                        if preds.len() == 1 {
                            preds[0].clone_box()
                        } else {
                            Box::new(crate::predicate::builtin::AndPredicate {
                                predicates: preds.iter().map(|p| p.clone_box()).collect(),
                            })
                        }
                    }),
                    transformation: s.transformation,
                    target_spec: s.target_spec,
                    cardinality: Cardinality::All,
                }
            }).collect();
            let program = crate::abstraction::program::GeneralizedProgram::new(abstract_steps, 1.0, ksg_pairs.len());
            self.program_synthesizer.generalized_programs.push(program);
        }"""

new_block = """        // Minden közös lépésből külön programot készítünk,
        // hogy a coverage teszt a legjobbat választhassa ki.
        for step in common_steps {
            let abstract_step = {
                use crate::abstraction::program::{AbstractStep, Cardinality};
                AbstractStep {
                    condition: step.condition.map(|preds| {
                        if preds.len() == 1 {
                            preds[0].clone_box()
                        } else {
                            Box::new(crate::predicate::builtin::AndPredicate {
                                predicates: preds.iter().map(|p| p.clone_box()).collect(),
                            })
                        }
                    }),
                    transformation: step.transformation,
                    target_spec: step.target_spec,
                    cardinality: Cardinality::All,
                }
            };
            let program = crate::abstraction::program::GeneralizedProgram::new(
                vec![abstract_step],
                1.0,
                ksg_pairs.len(),
            );
            self.program_synthesizer.generalized_programs.push(program);
        }"""

if old_block in content:
    content = content.replace(old_block, new_block)
    meta_path.write_text(content)
    print("finalize() updated: each common step gets its own GeneralizedProgram.")
else:
    print("ERROR: old block not found – manual check needed.")
