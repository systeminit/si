use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

use crate::attribute::value::AttributeValueId;

/// An IndexMap keeps track of which 'child' attribute resolvers of an
/// Array or Map property exist, their order, and what keys (if any) they
/// map to.
#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct IndexMap {
    order: Vec<AttributeValueId>,
    key_map: HashMap<AttributeValueId, String>,
}

impl IndexMap {
    /// Create a new IndexMap
    pub fn new() -> Self {
        IndexMap { ..Self::default() }
    }

    /// Push to the index map. If the `key` param is `None`, then the key will be the index
    /// of the item in the final order.
    pub fn push(&mut self, attribute_value_id: AttributeValueId, key: Option<String>) {
        self.order.push(attribute_value_id);
        let index = self.order.len() - 1;
        match key {
            Some(key_string) => {
                self.key_map.insert(attribute_value_id, key_string);
            }
            None => {
                self.key_map.insert(attribute_value_id, index.to_string());
            }
        }

        // Remove any duplicated order entries that appear after the first
        // occurrence of the entry.
        //
        // TODO: This should probably be done on (de)serialization, instead
        //       of on every insert, but that's an optimization for later.
        let mut order_set = HashSet::new();
        self.order.retain(|x| order_set.insert(*x));
    }

    pub fn delete(&mut self, attribute_value_id: AttributeValueId) {
        self.order.retain(|av| attribute_value_id != *av);
        self.key_map.remove(&attribute_value_id);
    }

    /// Returns the order of attribute resolvers for this index map as
    /// array; it does not include the keys.
    pub fn order(&self) -> &[AttributeValueId] {
        &self.order
    }

    /// Returns the order of attribute resolvers as index map as a map
    /// vec - the tuple will be the `key` and the `AttributeResolverId`
    /// this entry represents.
    pub fn order_as_map(&self) -> Vec<(String, AttributeValueId)> {
        self.order
            .iter()
            .map(|attribute_value_id| {
                let key = self
                    .key_map
                    .get(attribute_value_id)
                    .expect("index present in order, but not in keymap; this is a bug!");
                (key.clone(), *attribute_value_id)
            })
            .collect()
    }
}

impl<'a> postgres_types::FromSql<'a> for IndexMap {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let json: serde_json::Value = postgres_types::FromSql::from_sql(ty, raw)?;
        let index_map: IndexMap = serde_json::from_value(json)?;
        Ok(index_map)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty == &postgres_types::Type::JSONB
    }
}

impl postgres_types::ToSql for IndexMap {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        postgres_types::ToSql::to_sql(&self, ty, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_array() {
        let mut index_map = IndexMap::new();
        let first_id = AttributeValueId::generate();
        let second_id = AttributeValueId::generate();
        index_map.push(first_id, None);
        index_map.push(second_id, None);
        assert_eq!(index_map.order(), &[first_id, second_id]);
    }

    #[test]
    fn as_map() {
        let mut index_map = IndexMap::new();

        let first_id = AttributeValueId::generate();
        let second_id = AttributeValueId::generate();
        index_map.push(first_id, Some("bleed from within".to_string()));
        index_map.push(second_id, Some("lamb of god".to_string()));
        assert_eq!(
            index_map.order_as_map(),
            &[
                ("bleed from within".to_string(), first_id),
                ("lamb of god".to_string(), second_id)
            ]
        );
    }
}
