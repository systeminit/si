use std::{
    collections::{vec_deque::Iter, HashSet, VecDeque},
    time::Instant,
};

use si_data_nats::Subject;

use crate::{server::Error, Id};

#[derive(Debug)]
pub struct NodeMetadata {
    // This should really be an ordered set, to remove duplicates, but we'll deal with
    // that later.
    wanted_by_reply_channels: VecDeque<Subject>,
    processing_reply_channel: Option<Subject>,
    depends_on_node_ids: HashSet<Id>,
    processing_started_at: Option<Instant>,
    last_updated_at: Instant,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self {
            wanted_by_reply_channels: VecDeque::default(),
            processing_reply_channel: Option::default(),
            depends_on_node_ids: HashSet::default(),
            processing_started_at: Option::default(),
            last_updated_at: Instant::now(),
        }
    }
}

impl NodeMetadata {
    pub fn add_wanted_by_reply_channel(&mut self, reply_channel: &Subject) {
        self.wanted_by_reply_channels
            .push_back(reply_channel.to_owned());
        self.last_updated_at = Instant::now();
    }

    pub fn dependencies_satisfied(&self) -> bool {
        self.depends_on_node_ids.is_empty()
    }

    pub fn depends_on(&self, node_id: Id) -> bool {
        self.depends_on_node_ids.contains(&node_id)
    }

    pub fn is_empty(&self) -> bool {
        self.wanted_by_reply_channels.is_empty() && self.processing_reply_channel.is_none()
    }

    pub fn is_processing_stale(&self) -> bool {
        if let Some(processing_started_at) = self.processing_started_at {
            // If we've been updated more recently than when we last set the reply channel
            // for the job that should be processing this node, then the information that
            // job is using to update this node is out of date. We'll need to act as though
            // we never told that job to process this node in the first place, and have
            // a job that wants this node process it again with the up to date inputs.
            return processing_started_at < self.last_updated_at;
        }

        false
    }

    pub fn mark_as_processed(
        &mut self,
        reply_channel: &Subject,
    ) -> Result<(bool, HashSet<String>), Error> {
        if self.processing_reply_channel().map(|p| &**p) != Some(reply_channel) {
            return Err(Error::ShouldNotBeProcessingByJob);
        }

        let processing_reply_channel = self.processing_reply_channel.take();

        if self.is_processing_stale() {
            self.add_wanted_by_reply_channel(reply_channel);

            return Ok((false, HashSet::new()));
        }

        if self.dependencies_satisfied() {
            let mut wanted_by_reply_channels = self.wanted_by_reply_channels();
            if let Some(processed_by_reply_channel) = processing_reply_channel {
                wanted_by_reply_channels.insert(processed_by_reply_channel.to_string());
            }

            Ok((true, wanted_by_reply_channels))
        } else {
            Ok((false, HashSet::new()))
        }
    }

    pub fn merge_metadata(&mut self, reply_channel: Subject, dependencies: &Vec<Id>) {
        self.last_updated_at = Instant::now();

        if !self.wanted_by_reply_channels.contains(&reply_channel) {
            self.wanted_by_reply_channels.push_back(reply_channel);
        }
        self.depends_on_node_ids.extend(dependencies);
    }

    pub fn next_to_process(&mut self) -> Option<Subject> {
        if self.depends_on_node_ids.is_empty() && self.processing_reply_channel.is_none() {
            self.last_updated_at = Instant::now();

            self.processing_reply_channel = self.wanted_by_reply_channels.pop_front();
            if self.processing_reply_channel.is_some() {
                self.processing_started_at = Some(Instant::now());
            } else {
                self.processing_started_at = None;
            }
            return self.processing_reply_channel.clone();
        }
        None
    }

    pub fn processing_reply_channel(&self) -> Option<&Subject> {
        self.processing_reply_channel.as_ref()
    }

    pub fn remove_channel(&mut self, reply_channel: &Subject) {
        self.last_updated_at = Instant::now();

        self.wanted_by_reply_channels
            .retain(|el| el != reply_channel);
        self.processing_reply_channel = self
            .processing_reply_channel
            .take()
            .filter(|el| el != reply_channel);
    }

    pub fn remove_dependency(&mut self, node_id: Id) {
        if self.depends_on_node_ids.remove(&node_id) {
            self.last_updated_at = Instant::now();
        };
    }

    pub fn wanted_by_reply_channels(&self) -> HashSet<String> {
        HashSet::from_iter(self.wanted_by_reply_channels.iter().map(|s| s.to_string()))
    }

    pub fn wanted_by_reply_channels_iter(&self) -> Iter<'_, Subject> {
        self.wanted_by_reply_channels.iter()
    }
}
