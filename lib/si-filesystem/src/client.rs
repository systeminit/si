use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Serialize};
use si_frontend_types::{
    fs::{
        AssetFuncs, Binding, Bindings, ChangeSet, CreateChangeSetRequest, CreateChangeSetResponse,
        CreateFuncRequest, CreateSchemaRequest, CreateSchemaResponse, FsApiError, Func,
        IdentityBindings, ListChangeSetsResponse, Schema, SchemaAttributes, SetFuncBindingsRequest,
        SetFuncCodeRequest, VariantQuery,
    },
    FuncKind,
};
use si_id::{ChangeSetId, FuncId, SchemaId, WorkspaceId};
use thiserror::Error;
use tokio::{sync::RwLock, time::Instant};

#[derive(Error, Debug)]
pub enum SiFsClientError {
    #[error("backend error: {0}")]
    BackendError(FsApiError),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type SiFsClientResult<T> = Result<T, SiFsClientError>;

const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    created_at: Instant,
    duration: Duration,
}

impl CacheEntry {
    pub fn new(value: String, duration: Option<Duration>) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            duration: duration.unwrap_or(DEFAULT_CACHE_TTL),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SiFsClient {
    token: String,
    workspace_id: WorkspaceId,
    endpoint: String,
    client: reqwest::Client,
    cache: Arc<RwLock<BTreeMap<ChangeSetId, HashMap<String, CacheEntry>>>>,
}

const USER_AGENT: &str = "si-fs/0.0";

#[derive(Debug, Clone)]
pub struct SchemaFunc {
    pub locked: Option<Func>,
    pub unlocked: Option<Func>,
}

impl SiFsClient {
    pub fn new(
        token: String,
        workspace_id: WorkspaceId,
        endpoint: String,
    ) -> SiFsClientResult<Self> {
        Ok(Self {
            token,
            workspace_id,
            endpoint,
            client: reqwest::Client::builder().user_agent(USER_AGENT).build()?,
            cache: Arc::new(RwLock::new(BTreeMap::new())),
        })
    }

    fn make_cache_key<Q>(url: &str, query: Option<&Q>) -> String
    where
        Q: Serialize,
    {
        match query.as_ref() {
            Some(query) => {
                format!(
                    "{url}-{}",
                    serde_json::to_string(query)
                        .ok()
                        .unwrap_or("should never happen".into())
                )
            }
            None => url.to_owned(),
        }
    }

    async fn get_cache_entry<Q, R>(
        &self,
        change_set_id: ChangeSetId,
        url: String,
        query: Option<Q>,
    ) -> Option<R>
    where
        Q: Serialize,
        R: Serialize + DeserializeOwned + Clone,
    {
        let cache_key = Self::make_cache_key(&url, query.as_ref());

        self.get_cache_entry_custom_key(change_set_id, cache_key)
            .await
    }

    async fn get_cache_entry_custom_key<R>(
        &self,
        change_set_id: ChangeSetId,
        cache_key: String,
    ) -> Option<R>
    where
        R: Serialize + DeserializeOwned + Clone,
    {
        let cache = self.cache.read().await;
        cache.get(&change_set_id).and_then(|change_set_map| {
            change_set_map.get(&cache_key).and_then(|value| {
                if value.created_at.elapsed() >= value.duration {
                    None
                } else {
                    serde_json::from_str(&value.value).ok()
                }
            })
        })
    }

    async fn invalidate_change_set_id(&self, change_set_id: ChangeSetId) {
        self.cache.write().await.remove(&change_set_id);
    }

    async fn set_cache_entry_custom<R>(
        &self,
        change_set_id: ChangeSetId,
        cache_key: String,
        duration: Option<Duration>,
        value: &R,
    ) -> SiFsClientResult<()>
    where
        R: Serialize + DeserializeOwned + Clone,
    {
        let value_string = serde_json::to_string(value)?;
        let cache_entry = CacheEntry::new(value_string, duration);

        let mut cache = self.cache.write().await;
        cache
            .entry(change_set_id)
            .and_modify(|change_set_map| {
                change_set_map.insert(cache_key.clone(), cache_entry.clone());
            })
            .or_insert_with(|| {
                let mut change_set_map = HashMap::new();
                change_set_map.insert(cache_key, cache_entry);
                change_set_map
            });

        Ok(())
    }

    async fn set_cache_entry<Q, R>(
        &self,
        change_set_id: ChangeSetId,
        url: String,
        query: Option<Q>,
        value: &R,
    ) -> SiFsClientResult<()>
    where
        Q: Serialize,
        R: Serialize + DeserializeOwned + Clone,
    {
        let cache_key = Self::make_cache_key(&url, query.as_ref());
        self.set_cache_entry_custom(change_set_id, cache_key, None, value)
            .await
    }

    async fn get_text<Q>(
        &self,
        change_set_id: ChangeSetId,
        url: String,
        query: Option<Q>,
        cache: bool,
    ) -> SiFsClientResult<String>
    where
        Q: Serialize + Clone,
    {
        if let Some(cached_value) = cache
            .then_some(
                self.get_cache_entry(change_set_id, url.clone(), query.clone())
                    .await,
            )
            .flatten()
        {
            return Ok(cached_value);
        }

        let mut request_builder = self.client.get(url.clone()).bearer_auth(&self.token);
        request_builder = if let Some(query) = query.clone() {
            request_builder.query(&query)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;
        if response.status() == StatusCode::OK {
            let value = response.text().await?;
            if cache {
                self.set_cache_entry(change_set_id, url, query, &value)
                    .await?;
            }

            Ok(value)
        } else {
            let error: FsApiError = response.json().await?;
            dbg!(&error);

            Err(SiFsClientError::BackendError(error))
        }
    }

    async fn get_json<Q, R>(
        &self,
        change_set_id: ChangeSetId,
        url: String,
        query: Option<Q>,
    ) -> SiFsClientResult<R>
    where
        Q: Serialize + Clone,
        R: Serialize + DeserializeOwned + Clone,
    {
        if let Some(cached_value) = self
            .get_cache_entry(change_set_id, url.clone(), query.clone())
            .await
        {
            return Ok(cached_value);
        }

        let mut request_builder = self.client.get(url.clone()).bearer_auth(&self.token);
        request_builder = if let Some(query) = query.clone() {
            request_builder.query(&query)
        } else {
            request_builder
        };

        let start = Instant::now();
        let response = request_builder.send().await?;
        if response.status() == StatusCode::OK {
            println!("{url} ({:?})", start.elapsed());
            let value: R = response.json().await?;
            self.set_cache_entry(change_set_id, url, query, &value)
                .await?;

            Ok(value)
        } else {
            let error: FsApiError = response.json().await?;
            dbg!(&error);

            Err(SiFsClientError::BackendError(error))
        }
    }

    async fn post_empty_response<Q, V>(
        &self,
        change_set_id: ChangeSetId,
        url: String,
        query: Option<Q>,
        json: Option<V>,
    ) -> SiFsClientResult<()>
    where
        Q: Serialize + Clone,
        V: Serialize + DeserializeOwned + Clone,
    {
        let request_builder = self.client.post(url).bearer_auth(&self.token);

        let request_builder = if let Some(query) = query {
            request_builder.query(&query)
        } else {
            request_builder
        };

        let request_builder = if let Some(json) = json {
            request_builder.json(&json)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;
        if response.status() == StatusCode::OK {
            self.invalidate_change_set_id(change_set_id).await;
            Ok(())
        } else {
            let error: FsApiError = response.json().await?;
            dbg!(&error);

            Err(SiFsClientError::BackendError(error))
        }
    }

    async fn post<Q, V, R>(
        &self,
        change_set_id: ChangeSetId,
        url: String,
        query: Option<Q>,
        json: Option<V>,
    ) -> SiFsClientResult<R>
    where
        Q: Serialize + Clone,
        V: Serialize + DeserializeOwned + Clone,
        R: Serialize + DeserializeOwned + Clone,
    {
        let request_builder = self.client.post(url).bearer_auth(&self.token);

        let request_builder = if let Some(query) = query {
            request_builder.query(&query)
        } else {
            request_builder
        };

        let request_builder = if let Some(json) = json {
            request_builder.json(&json)
        } else {
            request_builder
        };

        let response = request_builder.send().await?;
        if response.status() == StatusCode::OK {
            self.invalidate_change_set_id(change_set_id).await;
            Ok(response.json().await?)
        } else {
            let error: FsApiError = response.json().await?;
            dbg!(&error);

            Err(SiFsClientError::BackendError(error))
        }
    }

    fn fs_api_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v2/workspaces/{}/fs/{suffix}",
            self.endpoint, self.workspace_id
        )
    }

    fn fs_api_change_sets(&self, suffix: &str, change_set_id: ChangeSetId) -> String {
        format!(
            "{}/api/v2/workspaces/{}/fs/change-sets/{change_set_id}/{suffix}",
            self.endpoint, self.workspace_id
        )
    }

    /// Fetches including the active change sets
    pub async fn list_change_sets(&self) -> SiFsClientResult<ListChangeSetsResponse> {
        let response = self
            .client
            .get(self.fs_api_url("change-sets"))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn create_change_set(&self, name: String) -> SiFsClientResult<ChangeSet> {
        let create_change_set_request = CreateChangeSetRequest { name };

        let response = self
            .client
            .post(self.fs_api_url("change-sets/create"))
            .bearer_auth(&self.token)
            .json(&create_change_set_request)
            .send()
            .await?
            .error_for_status()?;

        let response: CreateChangeSetResponse = response.json().await?;

        Ok(response)
    }

    pub async fn schemas(&self, change_set_id: ChangeSetId) -> SiFsClientResult<Vec<Schema>> {
        let url = self.fs_api_change_sets("schemas", change_set_id);
        self.get_json(change_set_id, url, None::<()>).await
    }

    pub async fn change_set_funcs_of_kind(
        &self,
        change_set_id: ChangeSetId,
        func_kind: FuncKind,
    ) -> SiFsClientResult<Vec<Func>> {
        let kind_string = si_frontend_types::fs::kind_to_string(func_kind);
        let url = self.fs_api_change_sets(&format!("funcs/{kind_string}"), change_set_id);
        self.get_json(change_set_id, url, None::<()>).await
    }

    pub async fn asset_funcs_for_schema(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<AssetFuncs> {
        self.get_json(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/asset_funcs"), change_set_id),
            None::<()>,
        )
        .await
    }

    pub async fn variant_funcs_of_kind(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        func_kind: FuncKind,
    ) -> SiFsClientResult<HashMap<String, SchemaFunc>> {
        let kind_string = si_frontend_types::fs::kind_to_string(func_kind);

        let funcs: Vec<Func> = self
            .get_json(
                change_set_id,
                self.fs_api_change_sets(
                    &format!("schemas/{schema_id}/funcs/{kind_string}"),
                    change_set_id,
                ),
                None::<()>,
            )
            .await?;

        let mut schema_funcs: HashMap<String, SchemaFunc> = HashMap::new();

        for func in funcs {
            schema_funcs
                .entry(func.name.clone())
                .and_modify(|f| {
                    if func.is_locked {
                        f.locked = Some(func.clone());
                    } else {
                        f.unlocked = Some(func.clone());
                    }
                })
                .or_insert_with(|| {
                    if func.is_locked {
                        SchemaFunc {
                            locked: Some(func),
                            unlocked: None,
                        }
                    } else {
                        SchemaFunc {
                            locked: None,
                            unlocked: Some(func),
                        }
                    }
                });
        }

        Ok(schema_funcs)
    }

    pub async fn get_func_code(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
    ) -> SiFsClientResult<String> {
        self.get_text(
            change_set_id,
            self.fs_api_change_sets(&format!("funcs/{func_id}/code"), change_set_id),
            None::<()>,
            true,
        )
        .await
    }

    pub async fn get_func_types(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
    ) -> SiFsClientResult<String> {
        self.get_text(
            change_set_id,
            self.fs_api_change_sets(&format!("funcs/{func_id}/types"), change_set_id),
            None::<()>,
            true,
        )
        .await
    }

    pub async fn set_func_code(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
        code: String,
    ) -> SiFsClientResult<()> {
        self.post_empty_response(
            change_set_id,
            self.fs_api_change_sets(&format!("funcs/{func_id}/code"), change_set_id),
            None::<()>,
            Some(SetFuncCodeRequest { code }),
        )
        .await
    }

    pub async fn get_asset_func_code(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        unlocked: bool,
    ) -> SiFsClientResult<String> {
        self.get_text(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/asset_func"), change_set_id),
            Some(VariantQuery { unlocked }),
            true,
        )
        .await
    }

    pub async fn get_asset_func_types(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<String> {
        if let Some(cached_types) = self
            .get_cache_entry_custom_key(change_set_id, "ASSET_TYPES".into())
            .await
        {
            return Ok(cached_types);
        }

        let types: String = self
            .get_text(
                change_set_id,
                self.fs_api_change_sets(
                    &format!("schemas/{schema_id}/asset_func/types"),
                    change_set_id,
                ),
                None::<()>,
                false,
            )
            .await?;

        // Asset types are static, but could change on deploy. This will cache
        // the types for 8 hours
        self.set_cache_entry_custom(
            change_set_id,
            "ASSET_TYPES".into(),
            Some(Duration::from_secs(60 * 60 * 8)),
            &types,
        )
        .await?;

        Ok(types)
    }

    pub async fn set_asset_func_code(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        code: String,
    ) -> SiFsClientResult<()> {
        self.post_empty_response(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/asset_func"), change_set_id),
            None::<()>,
            Some(SetFuncCodeRequest { code }),
        )
        .await
    }

    pub async fn create_schema(
        &self,
        change_set_id: ChangeSetId,
        name: String,
    ) -> SiFsClientResult<CreateSchemaResponse> {
        self.post(
            change_set_id,
            self.fs_api_change_sets("schemas/create", change_set_id),
            None::<()>,
            Some(CreateSchemaRequest { name }),
        )
        .await
    }

    pub async fn install_schema(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<()> {
        self.post_empty_response(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/install"), change_set_id),
            None::<()>,
            None::<()>,
        )
        .await
    }

    pub async fn get_schema_attrs(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        unlocked: bool,
    ) -> SiFsClientResult<SchemaAttributes> {
        self.get_json(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/attrs"), change_set_id),
            Some(VariantQuery { unlocked }),
        )
        .await
    }

    pub async fn set_schema_attrs(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        attributes: SchemaAttributes,
    ) -> SiFsClientResult<()> {
        self.post_empty_response(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/attrs"), change_set_id),
            None::<()>,
            Some(attributes),
        )
        .await
    }

    /// NOTE: the return here will always have None for the locked variant
    pub async fn unlock_schema(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<AssetFuncs> {
        self.post(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/unlock"), change_set_id),
            None::<()>,
            None::<()>,
        )
        .await
    }

    pub async fn unlock_func(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        func_id: FuncId,
    ) -> SiFsClientResult<Func> {
        self.post(
            change_set_id,
            self.fs_api_change_sets(
                &format!("schemas/{schema_id}/funcs/{func_id}/unlock"),
                change_set_id,
            ),
            None::<()>,
            None::<()>,
        )
        .await
    }

    pub async fn get_func_bindings(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<Bindings> {
        let bindings = self
            .get_json(
                change_set_id,
                self.fs_api_change_sets(
                    &format!("schemas/{schema_id}/funcs/{func_id}/bindings"),
                    change_set_id,
                ),
                None::<()>,
            )
            .await?;

        Ok(bindings)
    }

    pub async fn set_func_bindings(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
        schema_id: SchemaId,
        bindings: Bindings,
        is_attaching_existing: bool,
    ) -> SiFsClientResult<Option<Func>> {
        self.post(
            change_set_id,
            self.fs_api_change_sets(
                &format!("schemas/{schema_id}/funcs/{func_id}/bindings"),
                change_set_id,
            ),
            None::<()>,
            Some(SetFuncBindingsRequest {
                bindings,
                is_attaching_existing,
            }),
        )
        .await
    }

    pub async fn get_identity_bindings(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        unlocked: bool,
    ) -> SiFsClientResult<IdentityBindings> {
        self.get_json(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/bindings"), change_set_id),
            Some(VariantQuery { unlocked }),
        )
        .await
    }

    pub async fn set_identity_bindings(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        bindings: IdentityBindings,
    ) -> SiFsClientResult<()> {
        self.post_empty_response(
            change_set_id,
            self.fs_api_change_sets(&format!("schemas/{schema_id}/bindings"), change_set_id),
            None::<()>,
            Some(bindings),
        )
        .await
    }

    pub async fn create_func(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        func_kind: FuncKind,
        name: String,
        binding: Binding,
    ) -> SiFsClientResult<Func> {
        let kind_string = si_frontend_types::fs::kind_to_string(func_kind);

        let request = CreateFuncRequest { name, binding };
        self.post(
            change_set_id,
            self.fs_api_change_sets(
                &format!("schemas/{schema_id}/funcs/{kind_string}/create"),
                change_set_id,
            ),
            None::<()>,
            Some(request),
        )
        .await
    }
}
