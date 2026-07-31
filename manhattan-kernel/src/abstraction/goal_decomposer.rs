use crate::structure::KernelStructureGraph;

/// Egy részcél: egy köztes állapot, amit el kell érni a végső cél felé
#[derive(Debug, Clone)]
pub struct SubGoal {
    pub target_ksg: KernelStructureGraph,
    pub priority: u8,
    pub description: String,
}

/// A Goal Decomposer a komplex feladatokat kisebb, kezelhető részcélokra bontja
pub struct GoalDecomposer;

impl GoalDecomposer {
    /// Részcélok kinyerése a kezdeti és a cél KSG különbségéből
    pub fn decompose(
        initial: &KernelStructureGraph,
        target: &KernelStructureGraph,
    ) -> Vec<SubGoal> {
        let mut subgoals = Vec::new();

        // Stratégia: minden objektumra, ami a célban van, de a kezdetiben nincs,
        // vagy ami megváltozott, hozzunk létre egy részcél-t.
        for target_node in &target.nodes {
            let initial_node = initial.nodes.iter().find(|n| n.id == target_node.id);

            match initial_node {
                Some(init) => {
                    // Objektum megváltozott – hozzunk létre részcél-t a változásra
                    let mut changes = Vec::new();

                    if init.attributes.get("color") != target_node.attributes.get("color") {
                        changes.push("recolor");
                    }
                    if init.attributes.get("bbox_x") != target_node.attributes.get("bbox_x")
                        || init.attributes.get("bbox_y") != target_node.attributes.get("bbox_y")
                    {
                        changes.push("translate");
                    }
                    if init.attributes.get("bbox_w") != target_node.attributes.get("bbox_w")
                        && init.attributes.get("bbox_h") != target_node.attributes.get("bbox_h")
                    {
                        changes.push("rotate");
                    }

                    if !changes.is_empty() {
                        let mut sub_ksg = initial.clone();
                        // Frissítjük a részcél KSG-t, hogy csak ezt az egy változást tartalmazza
                        if let Some(sub_node) = sub_ksg.nodes.iter_mut().find(|n| n.id == target_node.id) {
                            for change in &changes {
                                match *change {
                                    "recolor" => {
                                        if let Some(color) = target_node.attributes.get("color") {
                                            sub_node.attributes.insert("color".to_string(), color.clone());
                                        }
                                    }
                                    "translate" => {
                                        if let Some(x) = target_node.attributes.get("bbox_x") {
                                            sub_node.attributes.insert("bbox_x".to_string(), x.clone());
                                        }
                                        if let Some(y) = target_node.attributes.get("bbox_y") {
                                            sub_node.attributes.insert("bbox_y".to_string(), y.clone());
                                        }
                                    }
                                    "rotate" => {
                                        if let Some(w) = target_node.attributes.get("bbox_w") {
                                            sub_node.attributes.insert("bbox_w".to_string(), w.clone());
                                        }
                                        if let Some(h) = target_node.attributes.get("bbox_h") {
                                            sub_node.attributes.insert("bbox_h".to_string(), h.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        subgoals.push(SubGoal {
                            target_ksg: sub_ksg,
                            priority: 1,
                            description: format!("Change {:?} for {}", changes, target_node.id),
                        });
                    }
                }
                None => {
                    // Új objektum a célban – részcél: létrehozás
                    let mut sub_ksg = initial.clone();
                    let new_node = sub_ksg.add_node(&target_node.id, &target_node.node_type);
                    new_node.attributes = target_node.attributes.clone();

                    subgoals.push(SubGoal {
                        target_ksg: sub_ksg,
                        priority: 2,
                        description: format!("Create object {}", target_node.id),
                    });
                }
            }
        }

        // Törölt objektumok – részcél: törlés
        for initial_node in &initial.nodes {
            if !target.nodes.iter().any(|n| n.id == initial_node.id) {
                let mut sub_ksg = initial.clone();
                sub_ksg.nodes.retain(|n| n.id != initial_node.id);
                sub_ksg.edges.retain(|e| e.from != initial_node.id && e.to != initial_node.id);

                subgoals.push(SubGoal {
                    target_ksg: sub_ksg,
                    priority: 3,
                    description: format!("Delete object {}", initial_node.id),
                });
            }
        }

        // Prioritás szerint rendezzük
        subgoals.sort_by_key(|s| s.priority);
        subgoals
    }
}

