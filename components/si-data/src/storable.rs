use crate::error::{DataError, Result};

pub enum Reference<'a> {
    HasOne(&'static str, &'a str),
    HasMany(&'static str, Vec<String>),
}

/// The Storable trait defines anything that can be stored in Couchbase; which
/// means anything that has an ID.
pub trait Storable {
    fn get_id(&self) -> &str;
    fn set_id(&mut self, id: impl Into<String>);
    fn type_name() -> &'static str;
    fn set_type_name(&mut self);
    fn set_natural_key(&mut self) {}
    fn get_natural_key(&self) -> Option<&str> {
        None
    }
    fn get_tenant_ids(&self) -> &[String];
    fn add_to_tenant_ids(&mut self, id: impl Into<String>);
    fn generate_id(&mut self);
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

impl crate::protobuf::DataStorable {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            tenant_ids: Vec::new(),
            natural_key: None,
            type_name: Some(type_name.into()),
        }
    }

    pub fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
    }
}
