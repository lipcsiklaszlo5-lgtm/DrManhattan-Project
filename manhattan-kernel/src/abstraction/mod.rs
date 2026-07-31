pub mod transform;
pub mod invariant;
pub mod program;
pub mod representation;
pub mod hypothesis;

pub use transform::{TransformationAlgebra, TransformRule, Condition};
pub use invariant::InvariantDetector;
pub use program::{Program, ProgramSynthesizer};
pub use representation::RepresentationFactory;
pub use hypothesis::{Hypothesis, HypothesisManager};

#[cfg(test)] mod representation_tests;
#[cfg(test)] mod hypothesis_tests;
