use std::{
    sync::Arc,
    time::{
        Duration,
        Instant,
    },
};

use dashmap::DashMap;
use innit_core::Parameter;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct CachedValue<T> {
    pub value: T,
    pub cached_at: Instant,
}

impl<T> CachedValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            cached_at: Instant::now(),
        }
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }
}

#[derive(Debug, Clone)]
pub struct ParameterCache {
    // Cache for single parameters
    parameter_cache: Arc<DashMap<String, CachedValue<Parameter>>>,
    // Cache for parameter paths
    path_cache: Arc<DashMap<String, CachedValue<Vec<Parameter>>>>,
    // Last refresh timestamp
    last_refresh: Arc<RwLock<Option<Instant>>>,
    // TTL for cache entries
    ttl: Duration,
}

impl ParameterCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            parameter_cache: Arc::new(DashMap::new()),
            path_cache: Arc::new(DashMap::new()),
            last_refresh: Arc::new(RwLock::new(None)),
            ttl,
        }
    }

    pub async fn get_parameter(&self, name: &str) -> Option<Parameter> {
        if let Some(cached) = self.parameter_cache.get(name) {
            if !cached.is_expired(self.ttl) {
                return Some(cached.value.clone());
            }
        }
        None
    }

    pub async fn set_parameter(&self, parameter: Parameter) {
        self.parameter_cache
            .insert(parameter.name.clone(), CachedValue::new(parameter));
    }

    pub async fn get_parameters_by_path(&self, path: &str) -> Option<Vec<Parameter>> {
        if let Some(cached) = self.path_cache.get(path) {
            if !cached.is_expired(self.ttl) {
                return Some(cached.value.clone());
            }
        }
        None
    }

    pub async fn set_parameters_by_path(&self, path: String, parameters: Vec<Parameter>) {
        self.path_cache.insert(path, CachedValue::new(parameters));
    }

    pub async fn clear_cache(&self) {
        self.parameter_cache.clear();
        self.path_cache.clear();
        *self.last_refresh.write().await = Some(Instant::now());
    }

    pub async fn get_last_refresh(&self) -> Option<Instant> {
        *self.last_refresh.read().await
    }
}
