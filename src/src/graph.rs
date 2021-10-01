use crate::{CommandResult, GraphCommand, Id, Label};
use crate::graphviz::Exporter;

pub struct Graph {
    node_high_water: usize,
    edge_high_water: usize,
    nodes: Vec<Node>,
    edges: Vec<Edge>
}

struct Node {
    id: Id,
    label: Label
}

struct Edge {
    id: Id,
    from: Id,
    to: Id
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            node_high_water: 0,
            edge_high_water: 0,
            nodes: vec![],
            edges: vec![]
        }
    }
}


impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }

    fn next_node_id(&mut self) -> Id {
        let id = format!("n{}", self.node_high_water);
        self.node_high_water += 1;
        Id::new(id)
    }

    fn next_edge_id(&mut self) -> Id {
        let id = format!("e{}", self.edge_high_water);
        self.edge_high_water += 1;
        Id::new(id)
    }

    fn find_edge_idx(&self, id: &Id) -> Option<usize> {
        self.edges
            .iter()
            .enumerate()
            .find(|(_, e)| &e.id == id)
            .map(|(idx, _)| idx)
    }

    fn find_node_idx(&self, id: &Id) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .find(|(_, e)| &e.id == id)
            .map(|(idx, _)| idx)
    }

    pub fn export(&self, exporter: &mut Exporter) {
        for node in &self.nodes {
            exporter.add_node(&node.id, &node.label);
        }

        for edge in &self.edges {
            exporter.add_edge(&edge.from, &edge.to);
        }
    }

    pub fn apply_command(&mut self, command: GraphCommand) -> CommandResult {
        match command {
            GraphCommand::InsertNode { label } => {
                let id = self.next_node_id();
                let node = Node {
                    id: id.clone(),
                    label: label.clone()
                };
                self.nodes.push(node);
                CommandResult::new(format!("inserted node {}: '{}'", id, label))
            }
            GraphCommand::DeleteNode { id } => {
                match self.find_node_idx(&id) {
                    Some(idx) => {
                        // delete all edges to or from this node
                        let mut edges_touching: Vec<Id> = vec![];
                        for edge in &self.edges {
                            if (edge.from == id || edge.to == id) && !edges_touching.contains(&&edge.id) {
                                edges_touching.push(edge.id.clone())
                            }
                        }

                        for delete in &edges_touching {
                            if let Some(idx) = self.find_edge_idx(delete) {
                                self.edges.remove(idx);
                            }
                        }

                        self.nodes.remove(idx);
                        CommandResult::new(format!("node {} removed", id))
                    },
                    None => CommandResult::new(format!("node {} not found", id))
                }
            }
            GraphCommand::LinkEdge { from, to } => {
                if !self.find_node_idx(&from).is_some() {
                    return CommandResult::new(format!("source node {} not found", from))
                }

                if !self.find_node_idx(&to).is_some() {
                    return CommandResult::new(format!("target node {} not found", to))
                }

                // we know both exist; create the edge
                let id = self.next_edge_id();
                let edge = Edge { id: id.clone(), from: from.clone(), to: to.clone() };
                self.edges.push(edge);
                CommandResult::new(format!("Added edge {} from {} to {}", id, from, to))

            }
            GraphCommand::RenameNode { id, label } => {
                if let Some(idx) = self.find_node_idx(&id) {
                    if let Some(node)  = self.nodes.get_mut(idx) {
                        node.label = label.clone();
                        CommandResult::new(format!("Node {} renamed to '{}'", id, label))
                    } else {
                        CommandResult::new(format!("Could not find node at index {}", idx))
                    }
                } else {
                    CommandResult::new(format!("Could not find node {}", id))
                }
            }
            GraphCommand::UnlinkEdge { id } => {
                match self.find_edge_idx(&id) {
                    Some(idx) => {
                        self.edges.remove(idx);
                        CommandResult::new(format!("edge {} removed", id))
                    }
                    None => CommandResult::new(format!("edge {} not found", id))
                }
            }
        }
    }
}