use std::collections::{HashMap, HashSet, VecDeque};

use petgraph::{
    dot::Dot,
    stable_graph::{DefaultIx, NodeIndex, StableDiGraph},
    visit::{EdgeRef, IntoNodeReferences},
};

use crate::formula::Expression;
use crate::{ContainVariable, Evaluable};

#[derive(Clone)]
pub struct BinaryDecisionDiagram {
    graph: StableDiGraph<String, bool>,
}

impl BinaryDecisionDiagram {
    pub fn from_formula(formula: &Expression) -> Self {
        fn recursive_add_subgraph(
            graph: &mut StableDiGraph<String, bool>,
            formula: &Expression,
            last_node_index: Option<NodeIndex<DefaultIx>>,
            last_node_value: bool,
            mut remain_variables: impl Iterator<Item = String> + Clone,
            current_variable_values: &mut HashMap<String, bool>,
        ) {
            let current_variable = remain_variables.next();
            if let Some(current_variable) = current_variable {
                let node = graph.add_node(current_variable.clone());
                if let Some(last_node_index) = last_node_index {
                    graph.add_edge(last_node_index, node, last_node_value);
                }

                current_variable_values.insert(current_variable.clone(), false);
                recursive_add_subgraph(
                    graph,
                    formula,
                    Some(node),
                    false,
                    remain_variables.clone(),
                    current_variable_values,
                );

                current_variable_values.insert(current_variable, true);
                recursive_add_subgraph(
                    graph,
                    formula,
                    Some(node),
                    true,
                    remain_variables,
                    current_variable_values,
                );
            } else {
                let value = formula.eval(current_variable_values);
                let node = graph.add_node(value.to_string());
                if let Some(last_node_index) = last_node_index {
                    graph.add_edge(last_node_index, node, last_node_value);
                }
            }
        }
        let variables = formula.variables();
        let variables_iter = variables.iter().cloned();
        let mut graph = StableDiGraph::new();
        let mut current_variable_values = HashMap::new();
        recursive_add_subgraph(
            &mut graph,
            formula,
            None,
            false,
            variables_iter,
            &mut current_variable_values,
        );
        Self { graph }
    }

    pub fn reduce(self) -> Self {
        let mut new_graph = StableDiGraph::new();
        let mut node_map = HashMap::new();
        let mut nodes_to_consider = VecDeque::new();
        let false_nodes = self.graph.node_references().filter_map(|it| {
            if it.1 == "false" {
                Some(it.0)
            } else {
                None
            }
        });
        let true_nodes =
            self.graph.node_references().filter_map(
                |it| {
                    if it.1 == "true" {
                        Some(it.0)
                    } else {
                        None
                    }
                },
            );
        let new_false_node = new_graph.add_node("false".to_string());
        let mut false_node_exists = false;
        for node in false_nodes {
            node_map.insert(node, new_false_node);
            let parent_nodes = self
                .graph
                .neighbors_directed(node, petgraph::Direction::Incoming);
            nodes_to_consider.extend(parent_nodes);
            false_node_exists = true;
        }
        if !false_node_exists {
            new_graph.remove_node(new_false_node);
        }
        let mut true_node_exists = false;
        let new_true_node = new_graph.add_node("true".to_string());
        for node in true_nodes {
            node_map.insert(node, new_true_node);
            let parent_nodes = self
                .graph
                .neighbors_directed(node, petgraph::Direction::Incoming);
            nodes_to_consider.extend(parent_nodes);
            true_node_exists = true;
        }
        if !true_node_exists {
            new_graph.remove_node(new_true_node);
        }
        while !nodes_to_consider.is_empty() {
            let node_to_consider = nodes_to_consider.pop_front().unwrap();
            if node_map.contains_key(&node_to_consider) {
                continue;
            }
            let (false_child_in_new_graph, true_child_in_new_graph) =
                self.children_in_new_graph(node_to_consider, &node_map);
            if false_child_in_new_graph == true_child_in_new_graph {
                node_map.insert(node_to_consider, *false_child_in_new_graph);
            } else {
                let false_child_parents: HashSet<_> = new_graph
                    .edges_directed(*false_child_in_new_graph, petgraph::Direction::Incoming)
                    .map(|it| it.source())
                    .collect();
                let true_child_parents: HashSet<_> = new_graph
                    .edges_directed(*true_child_in_new_graph, petgraph::Direction::Incoming)
                    .map(|it| it.source())
                    .collect();
                if let Some(equiv_node) =
                    false_child_parents.intersection(&true_child_parents).next()
                {
                    node_map.insert(node_to_consider, *equiv_node);
                } else {
                    let new_node = new_graph
                        .add_node(self.graph.node_weight(node_to_consider).unwrap().clone());
                    new_graph.add_edge(new_node, *false_child_in_new_graph, false);
                    new_graph.add_edge(new_node, *true_child_in_new_graph, true);
                    node_map.insert(node_to_consider, new_node);
                }
            }
            let parent_nodes = self
                .graph
                .neighbors_directed(node_to_consider, petgraph::Direction::Incoming);
            nodes_to_consider.extend(parent_nodes);
        }
        Self { graph: new_graph }
    }

    fn children_in_new_graph<'a>(
        &self,
        node: NodeIndex,
        node_map: &'a HashMap<NodeIndex, NodeIndex>,
    ) -> (&'a NodeIndex, &'a NodeIndex) {
        let false_child = self
            .graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .find(|it| !(*it.weight()))
            .unwrap()
            .target();
        let false_child_in_new_graph = node_map.get(&false_child).unwrap();
        let true_child = self
            .graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .find(|it| *it.weight())
            .unwrap()
            .target();
        let true_child_in_new_graph = node_map.get(&true_child).unwrap();
        (false_child_in_new_graph, true_child_in_new_graph)
    }

    pub fn dot(&self) -> String {
        Dot::new(&self.graph).to_string()
    }
}
