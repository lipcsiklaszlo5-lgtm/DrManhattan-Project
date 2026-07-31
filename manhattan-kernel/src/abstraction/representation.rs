use crate::structure::KernelStructureGraph;
use crate::adapter::arc::adapter::ArcGrid;
use crate::concept::{ConceptRegistry, Concept};

#[derive(Debug, Clone)]
pub struct Representation {
    pub name: String,
    pub graph: KernelStructureGraph,
    pub concepts: Vec<Concept>,
}

pub struct RepresentationFactory {
    concept_registry: ConceptRegistry,
}

impl RepresentationFactory {
    pub fn new() -> Self {
        Self { concept_registry: ConceptRegistry::default() }
    }
    pub fn with_registry(mut self, reg: ConceptRegistry) -> Self {
        self.concept_registry = reg; self
    }
    pub fn build_all(&self, grid: &ArcGrid) -> Vec<Representation> {
        let mut reps = Vec::new();
        let color_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(grid);
        let concepts = self.concept_registry.scan(&color_ksg);
        reps.push(Representation { name: "color".into(), graph: color_ksg, concepts });

        let cc_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(grid);
        let cc_concepts = self.concept_registry.scan(&cc_ksg);
        reps.push(Representation { name: "connected_components".into(), graph: cc_ksg, concepts: cc_concepts });

        let sym_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(grid);
        let sym_concepts = self.concept_registry.scan(&sym_ksg);
        reps.push(Representation { name: "symmetry".into(), graph: sym_ksg, concepts: sym_concepts });

        let topo_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(grid);
        let topo_concepts = self.concept_registry.scan(&topo_ksg);
        reps.push(Representation { name: "topology".into(), graph: topo_ksg, concepts: topo_concepts });

        reps
    }
}
