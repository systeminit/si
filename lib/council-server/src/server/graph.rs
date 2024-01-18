use crate::{server::Error, Graph, Id};
use std::collections::{HashMap, HashSet, VecDeque};

mod node_metadata;

use node_metadata::NodeMetadata;
use si_data_nats::Subject;

#[derive(Default, Debug)]
pub struct ChangeSetGraph {
    dependency_data: HashMap<Id, HashMap<Id, NodeMetadata>>,
}

impl ChangeSetGraph {
    pub fn is_empty(&self) -> bool {
        self.dependency_data.is_empty()
    }

    pub fn fetch_all_available(&mut self) -> HashMap<String, Vec<Id>> {
        let mut result: HashMap<String, Vec<Id>> = HashMap::new();
        for graph in self.dependency_data.values_mut() {
            for (id, metadata) in graph.iter_mut() {
                if let Some(reply_channel) = metadata.next_to_process() {
                    result
                        .entry(reply_channel.to_string())
                        .or_default()
                        .push(*id);
                }
            }
        }
        result
    }

    pub fn merge_dependency_graph(
        &mut self,
        reply_channel: Subject,
        new_dependency_data: Graph,
        change_set_id: Id,
    ) -> Result<(), Error> {
        let change_set_graph_data = self.dependency_data.entry(change_set_id).or_default();

        for (attribute_value_id, dependencies) in new_dependency_data {
            change_set_graph_data
                .entry(attribute_value_id)
                .and_modify(|node| {
                    node.merge_metadata(reply_channel.clone(), &dependencies);
                })
                .or_insert_with(|| {
                    let mut new_node = NodeMetadata::default();
                    new_node.merge_metadata(reply_channel.clone(), &dependencies);

                    new_node
                });

            for dependency in dependencies {
                change_set_graph_data
                    .entry(dependency)
                    .and_modify(|node| {
                        node.merge_metadata(reply_channel.clone(), &Vec::new());
                    })
                    .or_insert_with(|| {
                        let mut new_node = NodeMetadata::default();
                        new_node.merge_metadata(reply_channel.clone(), &Vec::new());

                        new_node
                    });
            }
        }

        Ok(())
    }

    pub fn mark_node_as_processed(
        &mut self,
        reply_channel: &Subject,
        change_set_id: Id,
        node_id: Id,
    ) -> Result<HashSet<String>, Error> {
        let change_set_graph_data = self.dependency_data.get_mut(&change_set_id).unwrap();

        let (ok_to_remove_node, wanted_by_reply_channels) =
            if let Some(node_metadata) = change_set_graph_data.get_mut(&node_id) {
                node_metadata.mark_as_processed(reply_channel)?
            } else {
                return Err(Error::UnknownNodeId);
            };

        if ok_to_remove_node {
            change_set_graph_data.remove(&node_id);

            for node_metadata in change_set_graph_data.values_mut() {
                node_metadata.remove_dependency(node_id);
            }

            if change_set_graph_data.is_empty() {
                self.dependency_data.remove(&change_set_id);
            }
        }

        Ok(wanted_by_reply_channels)
    }

    pub fn remove_channel(&mut self, change_set_id: Id, reply_channel: &Subject) {
        if let Some(graph) = self.dependency_data.get_mut(&change_set_id) {
            let mut to_remove = Vec::new();
            for (id, metadata) in graph.iter_mut() {
                metadata.remove_channel(reply_channel);
                if metadata.is_empty() {
                    to_remove.push(*id);
                }
            }

            for id in to_remove {
                graph.remove(&id).unwrap();
            }
        }
    }

    /// Return all `wanted_by_reply_channels` for `node_id` and remove the node
    /// from the graph. Also, remove the sub-graph starting at `node_id`,
    /// returning all `wanted_by_reply_channels` (with the associated `node_id`)
    /// for the nodes that are being removed.
    pub fn remove_node_and_dependents(
        &mut self,
        reply_channel: Subject,
        change_set_id: Id,
        node_id: Id,
    ) -> Result<Vec<(Subject, Id)>, Error> {
        let mut failure_notifications = Vec::new();
        let change_set_graph_data = self.dependency_data.get_mut(&change_set_id).unwrap();

        let mut node_ids_to_fail = VecDeque::new();
        node_ids_to_fail.push_back(node_id);
        // Include the initial node & the processing reply channel in the
        // list of notifications to send.
        failure_notifications.push((reply_channel.clone(), node_id));

        while let Some(node_id_to_fail) = node_ids_to_fail.pop_front() {
            if let Some(node_metadata) = change_set_graph_data.remove(&node_id_to_fail) {
                if node_metadata.processing_reply_channel().is_some()
                    && node_metadata.processing_reply_channel() != Some(&reply_channel)
                {
                    return Err(Error::ShouldNotBeProcessingByJob);
                }

                for notification_reply_channel in node_metadata.wanted_by_reply_channels_iter() {
                    failure_notifications
                        .push((notification_reply_channel.clone(), node_id_to_fail));
                }

                for (dependent_node_id, dependent_node_metadata) in change_set_graph_data.iter() {
                    if dependent_node_metadata.depends_on(node_id_to_fail) {
                        node_ids_to_fail.push_back(*dependent_node_id);
                    }
                }
            }
        }

        // Nothing left in the graph for the change set, we shouldn't keep
        // an entry for it around.
        if change_set_graph_data.is_empty() {
            self.dependency_data.remove(&change_set_id);
        }

        Ok(failure_notifications)
    }
}
