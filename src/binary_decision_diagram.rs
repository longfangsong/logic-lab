use std::{
    cell::OnceCell,
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
};

use petgraph::{
    dot::Dot,
    stable_graph::{DefaultIx, NodeIndex, StableDiGraph},
    visit::{EdgeRef, IntoNodeReferences},
};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::formula::{Expression, expression};
use crate::{ContainVariable, Evaluable};

#[wasm_bindgen]
#[derive(Clone)]
pub struct BinaryDecisionDiagram {
    graph: StableDiGraph<String, bool>,
    variables_cache: OnceCell<BTreeSet<String>>,
}

impl ContainVariable for BinaryDecisionDiagram {
    fn variables(&self) -> BTreeSet<String> {
        self.variables_cache
            .get_or_init(|| self.graph.node_weights().cloned().collect())
            .clone()
    }
}

#[wasm_bindgen]
impl BinaryDecisionDiagram {
    pub fn reduce(self) -> Self {
        fn children_in_new_graph<'a>(
            old_graph: &StableDiGraph<String, bool>,
            node: NodeIndex,
            node_map: &'a HashMap<NodeIndex, NodeIndex>,
        ) -> (&'a NodeIndex, &'a NodeIndex) {
            let false_child = old_graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .find(|it| !(*it.weight()))
                .unwrap()
                .target();
            let false_child_in_new_graph = node_map.get(&false_child).unwrap();
            let true_child = old_graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .find(|it| *it.weight())
                .unwrap()
                .target();
            let true_child_in_new_graph = node_map.get(&true_child).unwrap();
            (false_child_in_new_graph, true_child_in_new_graph)
        }

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
                children_in_new_graph(&self.graph, node_to_consider, &node_map);
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
        Self {
            graph: new_graph,
            variables_cache: self.variables_cache,
        }
    }

    fn root_in_graph(graph: &StableDiGraph<String, bool>) -> NodeIndex {
        graph
            .node_indices()
            .find(|it| {
                graph
                    .neighbors_directed(*it, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .unwrap()
    }

    pub fn restrict(&mut self, variable_name: &str, variable_value: bool) {
        let nodes: Vec<NodeIndex> = self
            .graph
            .node_references()
            .filter(|(_, weight)| *weight == variable_name)
            .map(|(index, _)| index)
            .collect();
        for node in nodes {
            let incoming_edges: Vec<_> = self
                .graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .map(|it| (it.id(), it.source(), *it.weight()))
                .collect();
            let redirect_to = self
                .graph
                .edges_directed(node, petgraph::Direction::Outgoing)
                .find(|it| *it.weight() == variable_value)
                .unwrap()
                .target();
            for (id, source, weight) in incoming_edges {
                self.graph.add_edge(source, redirect_to, weight);
                self.graph.remove_edge(id);
            }
            self.graph.remove_node(node);
        }
    }

    pub fn exists(&self, variable_name: &str) -> Self {
        let mut restrict_false = self.clone();
        restrict_false.restrict(variable_name, false);
        let mut restrict_true = self.clone();
        restrict_true.restrict(variable_name, true);
        restrict_false.apply(&restrict_true, |a, b| a || b).reduce()
    }

    pub fn universal(&self, variable_name: &str) -> Self {
        let mut restrict_false = self.clone();
        restrict_false.restrict(variable_name, false);
        let mut restrict_true = self.clone();
        restrict_true.restrict(variable_name, true);
        restrict_false.apply(&restrict_true, |a, b| a && b).reduce()
    }

    pub fn dot(&self) -> String {
        Dot::new(&self.graph).to_string()
    }
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
        let variables_cell = OnceCell::new();
        variables_cell.set(variables).unwrap();
        Self {
            graph,
            variables_cache: variables_cell,
        }
    }
    
    #[allow(clippy::bool_comparison)]
    pub fn apply(&self, other: &Self, f: fn(bool, bool) -> bool) -> Self {
        fn recursive_apply(
            graph: &mut StableDiGraph<String, bool>,
            f: fn(bool, bool) -> bool,
            lhs_graph: &StableDiGraph<String, bool>,
            lhs_cursor: NodeIndex,
            rhs_graph: &StableDiGraph<String, bool>,
            rhs_cursor: NodeIndex,
        ) -> NodeIndex {
            fn explore_left(
                graph: &mut StableDiGraph<String, bool>,
                f: fn(bool, bool) -> bool,
                lhs_graph: &StableDiGraph<String, bool>,
                rhs_graph: &StableDiGraph<String, bool>,
                lhs_cursor: NodeIndex,
                rhs_cursor: NodeIndex,
                lhs_value: &str,
            ) -> NodeIndex {
                let node = graph.add_node(lhs_value.to_string());

                let lhs_cursor_left = lhs_graph
                    .edges_directed(lhs_cursor, petgraph::Direction::Outgoing)
                    .find(|it| *it.weight() == false)
                    .unwrap()
                    .target();
                let lhs_node_left =
                    recursive_apply(graph, f, lhs_graph, lhs_cursor_left, rhs_graph, rhs_cursor);
                graph.add_edge(node, lhs_node_left, false);

                let lhs_cursor_right = lhs_graph
                    .edges_directed(lhs_cursor, petgraph::Direction::Outgoing)
                    .find(|it| *it.weight() == true)
                    .unwrap()
                    .target();
                let lhs_node_right =
                    recursive_apply(graph, f, lhs_graph, lhs_cursor_right, rhs_graph, rhs_cursor);
                graph.add_edge(node, lhs_node_right, true);

                node
            }
            fn explore_right(
                graph: &mut StableDiGraph<String, bool>,
                f: fn(bool, bool) -> bool,
                lhs_graph: &StableDiGraph<String, bool>,
                rhs_graph: &StableDiGraph<String, bool>,
                lhs_cursor: NodeIndex,
                rhs_cursor: NodeIndex,
                rhs_value: &str,
            ) -> NodeIndex {
                let node = graph.add_node(rhs_value.to_string());

                let rhs_cursor_left = rhs_graph
                    .edges_directed(rhs_cursor, petgraph::Direction::Outgoing)
                    .find(|it| *it.weight() == false)
                    .unwrap()
                    .target();
                let rhs_node_left =
                    recursive_apply(graph, f, lhs_graph, lhs_cursor, rhs_graph, rhs_cursor_left);
                graph.add_edge(node, rhs_node_left, false);

                let rhs_cursor_right = rhs_graph
                    .edges_directed(rhs_cursor, petgraph::Direction::Outgoing)
                    .find(|it| *it.weight() == true)
                    .unwrap()
                    .target();
                let rhs_node_right =
                    recursive_apply(graph, f, lhs_graph, lhs_cursor, rhs_graph, rhs_cursor_right);
                graph.add_edge(node, rhs_node_right, true);

                node
            }
            let mut explore_both =
                |lhs_cursor: NodeIndex, rhs_cursor: NodeIndex, value: &str| -> NodeIndex {
                    let node = graph.add_node(value.to_string());
                    let lhs_cursor_left = lhs_graph
                        .edges_directed(lhs_cursor, petgraph::Direction::Outgoing)
                        .find(|it| *it.weight() == false)
                        .unwrap()
                        .target();
                    let rhs_cursor_left = rhs_graph
                        .edges_directed(rhs_cursor, petgraph::Direction::Outgoing)
                        .find(|it| *it.weight() == false)
                        .unwrap()
                        .target();
                    let node_left = recursive_apply(
                        graph,
                        f,
                        lhs_graph,
                        lhs_cursor_left,
                        rhs_graph,
                        rhs_cursor_left,
                    );
                    graph.add_edge(node, node_left, false);

                    let lhs_cursor_right = lhs_graph
                        .edges_directed(lhs_cursor, petgraph::Direction::Outgoing)
                        .find(|it| *it.weight() == true)
                        .unwrap()
                        .target();
                    let rhs_cursor_right = rhs_graph
                        .edges_directed(rhs_cursor, petgraph::Direction::Outgoing)
                        .find(|it| *it.weight() == true)
                        .unwrap()
                        .target();
                    let node_right = recursive_apply(
                        graph,
                        f,
                        lhs_graph,
                        lhs_cursor_right,
                        rhs_graph,
                        rhs_cursor_right,
                    );
                    graph.add_edge(node, node_right, true);

                    node
                };
            let lhs_value = lhs_graph.node_weight(lhs_cursor).unwrap();
            let rhs_value = rhs_graph.node_weight(rhs_cursor).unwrap();
            match (lhs_value.as_str(), rhs_value.as_str()) {
                ("true", "true") | ("true", "false") | ("false", "true") | ("false", "false") => {
                    let lhs_value = lhs_value == "true";
                    let rhs_value = rhs_value == "true";
                    let result = f(lhs_value, rhs_value);
                    graph.add_node(result.to_string())
                }
                (lhs, "true") | (lhs, "false") => {
                    explore_left(graph, f, lhs_graph, rhs_graph, lhs_cursor, rhs_cursor, lhs)
                }
                ("true", rhs) | ("false", rhs) => {
                    explore_right(graph, f, lhs_graph, rhs_graph, lhs_cursor, rhs_cursor, rhs)
                }
                (lhs, rhs) if lhs < rhs => {
                    explore_left(graph, f, lhs_graph, rhs_graph, lhs_cursor, rhs_cursor, lhs)
                }
                (lhs, rhs) if lhs > rhs => {
                    explore_right(graph, f, lhs_graph, rhs_graph, lhs_cursor, rhs_cursor, rhs)
                }
                (lhs, _rhs) => explore_both(lhs_cursor, rhs_cursor, lhs),
            }
        }
        let mut new_graph = StableDiGraph::new();
        let lhs_cursor = Self::root_in_graph(&self.graph);
        let rhs_cursor = Self::root_in_graph(&other.graph);
        recursive_apply(
            &mut new_graph,
            f,
            &self.graph,
            lhs_cursor,
            &other.graph,
            rhs_cursor,
        );
        Self {
            graph: new_graph,
            variables_cache: OnceCell::new(),
        }
    }
}

#[wasm_bindgen]
impl BinaryDecisionDiagram {
    pub fn from_str(code: &str) -> Self {
        let expr = expression::parse(code).unwrap().1;
        BinaryDecisionDiagram::from_formula(&expr)
    }

    pub fn or(&self, other: &BinaryDecisionDiagram) -> Self {
        self.apply(other, |lhs, rhs| lhs || rhs)
    }
    
    pub fn and(&self, other: &BinaryDecisionDiagram) -> Self {
        self.apply(other, |lhs, rhs| lhs && rhs)
    }
}