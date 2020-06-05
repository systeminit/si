use crate::error::{DataError, Result};

pub use crate::protobuf::DataStorable;

/// The Storable trait defines anything that can be stored in Couchbase; which
/// means anything that has an ID.
pub trait Storable {
    fn type_name() -> &'static str;
    fn set_type_name(&mut self);

    fn id(&self) -> Result<&str>;
    fn set_id(&mut self, id: impl Into<String>);
    fn generate_id(&mut self);

    fn change_set_id(&self) -> Result<Option<&str>> {
        Ok(None)
    }
    fn set_change_set_entry_count(&mut self, _entry_count: u64) -> Result<()> {
        Ok(())
    }

    fn natural_key(&self) -> Result<Option<&str>> {
        Ok(None)
    }
    fn set_natural_key(&mut self) -> Result<()> {
        Ok(())
    }

    fn tenant_ids(&self) -> Result<&[String]>;
    fn add_to_tenant_ids(&mut self, id: impl Into<String>);

    fn validate(&self) -> Result<()>;

    fn referential_fields(&self) -> Vec<Reference>;

    fn order_by_fields() -> Vec<&'static str>;

    fn is_order_by_valid<S: AsRef<str>>(
        order_by: S,
        order_by_fields: Vec<&'static str>,
    ) -> Result<()> {
        let test = order_by.as_ref();

        match order_by_fields.iter().find(|o| *o == &test) {
            Some(_) => Ok(()),
            None => Err(DataError::InvalidOrderBy),
        }
    }
}

pub enum Reference<'a> {
    HasOne(&'static str, &'a str),
    HasMany(&'static str, Vec<String>),
}

impl DataStorable {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            tenant_ids: Vec::new(),
            natural_key: None,
            type_name: Some(type_name.into()),
            view_context: None,
            change_set_id: None,
            change_set_entry_count: None,
            change_set_event_type: crate::protobuf::DataStorableChangeSetEventType::Unknown as i32,
            change_set_executed: Some(false),
            deleted: Some(false),
            item_id: None,
        }
    }

    pub fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
    }
}
