// Auth-generated code!
// No touchy!

impl crate::protobuf::Edge {
    pub fn new(
        name: Option<String>,
        display_name: Option<String>,
        si_properties: Option<crate::protobuf::EdgeSiProperties>,
        head_vertex: Option<crate::protobuf::EdgeHeadVertex>,
        tail_vertex: Option<crate::protobuf::EdgeTailVertex>,
        bidirectional: Option<bool>,
        edge_kind: Option<crate::protobuf::EdgeEdgeKind>,
    ) -> si_data::Result<crate::protobuf::Edge> {
        let mut si_storable = si_data::protobuf::DataStorable::default();
        si_properties
            .as_ref()
            .ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;
        let billing_account_id = si_properties
            .as_ref()
            .unwrap()
            .billing_account_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.billingAccountId".into())
            })?;
        si_storable.add_to_tenant_ids(billing_account_id);
        let organization_id = si_properties
            .as_ref()
            .unwrap()
            .organization_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.organizationId".into())
            })?;
        si_storable.add_to_tenant_ids(organization_id);
        let workspace_id = si_properties
            .as_ref()
            .unwrap()
            .workspace_id
            .as_ref()
            .ok_or_else(|| {
                si_data::DataError::ValidationError("siProperties.workspaceId".into())
            })?;
        si_storable.add_to_tenant_ids(workspace_id);

        let mut result: crate::protobuf::Edge = Default::default();
        result.name = name;
        result.display_name = display_name;
        result.si_properties = si_properties;
        result.head_vertex = head_vertex;
        result.tail_vertex = tail_vertex;
        result.bidirectional = bidirectional;
        result.edge_kind = edge_kind
            .map(|l| {
                let value: i32 = l.into();
                value
            })
            .unwrap_or(0);
        result.set_edge_kind(crate::protobuf::EdgeEdgeKind::Connected);
        result.si_storable = Some(si_storable);

        Ok(result)
    }

    pub async fn create(
        db: &si_data::Db,
        name: Option<String>,
        display_name: Option<String>,
        si_properties: Option<crate::protobuf::EdgeSiProperties>,
        head_vertex: Option<crate::protobuf::EdgeHeadVertex>,
        tail_vertex: Option<crate::protobuf::EdgeTailVertex>,
        bidirectional: Option<bool>,
        edge_kind: Option<crate::protobuf::EdgeEdgeKind>,
    ) -> si_data::Result<crate::protobuf::Edge> {
        let mut result = crate::protobuf::Edge::new(
            name,
            display_name,
            si_properties,
            head_vertex,
            tail_vertex,
            bidirectional,
            edge_kind,
        )?;
        db.validate_and_insert_as_new(&mut result).await?;

        Ok(result)
    }

    pub async fn get(db: &si_data::Db, id: &str) -> si_data::Result<crate::protobuf::Edge> {
        let obj = db.get(id).await?;
        Ok(obj)
    }

    pub async fn get_by_natural_key(
        db: &si_data::Db,
        natural_key: &str,
    ) -> si_data::Result<crate::protobuf::Edge> {
        let obj = db.lookup_by_natural_key(natural_key).await?;
        Ok(obj)
    }

    pub async fn save(&self, db: &si_data::Db) -> si_data::Result<()> {
        db.upsert(self).await?;
        Ok(())
    }

    pub async fn finalize(&self, db: &si_data::Db) -> si_data::Result<()> {
        tracing::debug!("finalizing_edge");
        db.upsert(self).await?;

        Ok(())
    }

    pub async fn list(
        db: &si_data::Db,
        list_request: crate::protobuf::EdgeListRequest,
    ) -> si_data::Result<si_data::ListResult<crate::protobuf::Edge>> {
        let result = match list_request.page_token {
            Some(token) => db.list_by_page_token(token).await?,
            None => {
                let page_size = match list_request.page_size {
                    Some(page_size) => page_size,
                    None => 10,
                };
                let order_by = match list_request.order_by {
                    Some(order_by) => order_by,
                    // The empty string is the signal for a default, thanks protobuf history
                    None => "".to_string(),
                };
                let contained_within = match list_request.scope_by_tenant_id {
                    Some(contained_within) => contained_within,
                    None => return Err(si_data::DataError::MissingScopeByTenantId),
                };
                db.list(
                    &list_request.query,
                    page_size,
                    order_by,
                    list_request.order_by_direction,
                    contained_within,
                    "",
                )
                .await?
            }
        };
        Ok(result)
    }
}

impl si_data::Storable for crate::protobuf::Edge {
    fn type_name() -> &'static str {
        "edge"
    }

    fn set_type_name(&mut self) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::Edge.si_storable \
                has been set or initialized",
        );
        si_storable.type_name = Some(Self::type_name().to_string());
    }

    fn id(&self) -> si_data::Result<&str> {
        self.id
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = Some(id.into());
    }

    fn change_set_id(&self) -> si_data::Result<Option<&str>> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .change_set_id
            .as_ref()
            .map(String::as_str))
    }

    fn set_change_set_entry_count(&mut self, entry_count: u64) -> si_data::Result<()> {
        self.si_storable
            .as_mut()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .change_set_entry_count
            .replace(entry_count);
        Ok(())
    }

    // How this should work:
    //
    //  * Do we have an ID?
    //      * Are we in a change set?
    //          * Update the order
    //          * Set the new ID
    //      * keep the current ID
    //  * We don't have an ID
    //      * Generate a new real object id
    //          * Set the item ID to it
    //      * Make the change-set id, and set that as the real one.
    //
    // This needs to possibly error now!
    fn generate_id(&mut self) {
        if let Ok(_current_id) = self.id() {
            if let Some(change_set_id) = self
                .si_storable
                .as_ref()
                .map(|si_storable| si_storable.change_set_id.as_ref())
                .flatten()
            {
                let real_id = self
                    .si_storable
                    .as_ref()
                    .map(|si_storable| si_storable.item_id.as_ref())
                    .flatten()
                    .expect("must have a real item_id");
                let change_set_entry_count = self
                    .si_storable
                    .as_ref()
                    .map(|si_storable| si_storable.change_set_entry_count.as_ref())
                    .flatten()
                    .expect("must have a change_set_entry_count");
                let new_id = format!("{}:{}:{}", change_set_id, change_set_entry_count, real_id);
                self.set_id(new_id);
            }
        } else {
            let real_id = format!("{}:{}", Self::type_name(), si_data::uuid_string(),);
            self.si_storable
                .as_mut()
                .map(|si_storable| si_storable.item_id = Some(real_id.clone()));
            if let Some(change_set_id) = self
                .si_storable
                .as_ref()
                .map(|si_storable| si_storable.change_set_id.as_ref())
                .flatten()
            {
                let change_set_entry_count = self
                    .si_storable
                    .as_ref()
                    .map(|si_storable| si_storable.change_set_entry_count.as_ref())
                    .flatten()
                    .expect("must have a change_set_entry_count");
                let new_id = format!("{}:{}:{}", change_set_id, change_set_entry_count, real_id);
                self.set_id(new_id);
            } else {
                self.set_id(real_id);
            }
        }
    }

    fn natural_key(&self) -> si_data::Result<Option<&str>> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .natural_key
            .as_ref()
            .map(String::as_str))
    }

    fn set_natural_key(&mut self) -> si_data::Result<()> {
        let natural_key = format!(
            "{}:{}:{}",
            self.tenant_ids()?
                .first()
                .ok_or_else(|| si_data::DataError::MissingTenantIds)?,
            Self::type_name(),
            self.name
                .as_ref()
                .ok_or_else(|| si_data::DataError::RequiredField("name".to_string()))?,
        );

        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::Edge.si_storable \
                has been set or initialized",
        );
        si_storable.natural_key = Some(natural_key);

        Ok(())
    }

    fn tenant_ids(&self) -> si_data::Result<&[String]> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .tenant_ids
            .as_slice())
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        if self.si_storable.is_none() {
            self.si_storable = Some(Default::default());
        }

        let si_storable = self.si_storable.as_mut().expect(
            "crate::protobuf::Edge.si_storable \
                has been set or initialized",
        );
        si_storable.tenant_ids.push(id.into());
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.id.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required id value".into(),
            ));
        }
        if self.name.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required name value".into(),
            ));
        }
        if self.display_name.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required display_name value".into(),
            ));
        }
        if self.si_storable.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required si_storable value".into(),
            ));
        }
        if self.si_properties.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required si_properties value".into(),
            ));
        }
        if self.head_vertex.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required head_vertex value".into(),
            ));
        }
        if self.tail_vertex.is_none() {
            return Err(si_data::DataError::ValidationError(
                "missing required tail_vertex value".into(),
            ));
        }

        Ok(())
    }

    fn referential_fields(&self) -> Vec<si_data::Reference> {
        Vec::new()
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "siStorable.naturalKey",
            "id",
            "name",
            "displayName",
            "siStorable.naturalKey",
            "dataStorable.viewContext",
            "dataStorable.changeSetId",
            "dataStorable.itemId",
            "dataStorable.changeSetEntryCount",
            "dataStorable.changeSetEventType",
            "dataStorable.changeSetReverted",
            "dataStorable.changeSetExecuted",
            "dataStorable.deleted",
            "dataStorable.editSessionId",
            "dataStorable.reverted",
            "siStorable.naturalKey",
            "siProperties.billingAccountId",
            "siProperties.organizationId",
            "siProperties.workspaceId",
            "siStorable.naturalKey",
            "headVertex.id",
            "headVertex.socket",
            "headVertex.typeName",
            "siStorable.naturalKey",
            "tailVertex.id",
            "tailVertex.socket",
            "tailVertex.typeName",
            "bidirectional",
            "edgeKind",
        ]
    }

    fn edit_session_id(&self) -> si_data::Result<Option<&str>> {
        Ok(self
            .si_storable
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("si_storable".to_string()))?
            .edit_session_id
            .as_ref()
            .map(String::as_str))
    }
}
