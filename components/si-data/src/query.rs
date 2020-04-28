use crate::error::{DataError, Result};
pub use crate::protobuf::{
    DataQuery, DataQueryBooleanTerm, DataQueryItems, DataQueryItemsExpression,
    DataQueryItemsExpressionComparison, DataQueryItemsExpressionFieldType,
};

use std::fmt;

use serde_json::json;

//
//  Queries are made up of a number of Expression Set. Expression Set are made up
//  of a number of Expression Set or Expressions (F=V), joined together by a boolean
//  operator (AND, OR) and a group modifier, NOT.
//
//  (AND expression, expression, (OR expression expression))
//
//  (foo = "bar" AND (foo=v OR f=v))
//  (foo = "bar" AND NOT (foo=v OR g=v))
//
//  The rules are we evaluate each expression in a group, combined with its boolean
//  operator. If the next expression is an expression group, and the not modifier is
//  applied, we do the boolean operator followed byt the NOT. Then the group is
//  evaluated, with eevery expression joined together with AND.
//

impl fmt::Display for DataQueryItemsExpressionComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &DataQueryItemsExpressionComparison::Unknown => "UNKNOWN".to_string(),
            &DataQueryItemsExpressionComparison::Equals => "=".to_string(),
            &DataQueryItemsExpressionComparison::NotEquals => "!=".to_string(),
            &DataQueryItemsExpressionComparison::Contains => "CONTAINS".to_string(),
            &DataQueryItemsExpressionComparison::Like => "LIKE".to_string(),
            &DataQueryItemsExpressionComparison::NotLike => "NOT LIKE".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl fmt::Display for DataQueryBooleanTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &DataQueryBooleanTerm::Unknown => "UNKNOWN".to_string(),
            &DataQueryBooleanTerm::And => "AND".to_string(),
            &DataQueryBooleanTerm::Or => "OR".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl DataQuery {
    pub fn generate_for_string(
        field: impl Into<String>,
        comparison: DataQueryItemsExpressionComparison,
        value: impl Into<String>,
    ) -> Self {
        DataQuery {
            items: vec![DataQueryItems::generate_expression_for_string(
                field, comparison, value,
            )],
            ..Default::default()
        }
    }

    pub fn generate_for_int(
        field: impl Into<String>,
        comparison: DataQueryItemsExpressionComparison,
        value: impl Into<String>,
    ) -> DataQuery {
        DataQuery {
            items: vec![DataQueryItems::generate_expression_for_int(
                field, comparison, value,
            )],
            ..Default::default()
        }
    }

    pub fn as_n1ql(&self, bucket_name: &str) -> Result<String> {
        let mut where_string = String::new();
        if self.is_not.unwrap_or(false) {
            where_string.push_str("NOT ");
        }
        where_string.push_str("(");
        let mut item_count = 0;
        for query_e_option in self.items.iter() {
            if item_count != 0 {
                let qbl = match DataQueryBooleanTerm::from_i32(self.boolean_term) {
                    Some(l) => l,
                    None => return Err(DataError::InvalidBooleanLogic),
                };
                let boolean_term = format!(" {} ", qbl);
                where_string.push_str(&boolean_term);
            }
            if query_e_option.expression.is_some() {
                match &query_e_option.expression {
                    Some(exp) => {
                        let comparison =
                            match DataQueryItemsExpressionComparison::from_i32(exp.comparison) {
                                Some(c) => c,
                                None => return Err(DataError::InvalidDataQueryComparison),
                            };
                        let value =
                            match DataQueryItemsExpressionFieldType::from_i32(exp.field_type) {
                                Some(DataQueryItemsExpressionFieldType::Unknown) => {
                                    json![exp.value]
                                }
                                Some(DataQueryItemsExpressionFieldType::String) => json![exp.value],
                                Some(DataQueryItemsExpressionFieldType::Int) => {
                                    // Elizabeth Jacob fixed this bug, her first, on 04-13-2020.
                                    // Good job, Monkey.
                                    let vint: u64 =
                                        exp.value.as_ref().unwrap_or(&"".into()).parse()?;
                                    json![vint]
                                }
                                None => return Err(DataError::InvalidFieldType),
                            };
                        let exp_field = match &exp.field {
                            Some(field) => {
                                if field.contains(".") {
                                    let mut escaped_fields = Vec::new();
                                    for field_part in field.split(".") {
                                        let escaped = format!("`{}`", field_part);
                                        escaped_fields.push(escaped);
                                    }
                                    escaped_fields.join(".")
                                } else {
                                    field.clone()
                                }
                            }
                            None => {
                                return Err(DataError::RequiredField(
                                    "query.expression.field".into(),
                                ))
                            }
                        };
                        let expression =
                            if comparison == DataQueryItemsExpressionComparison::Contains {
                                format!("ARRAY_CONTAINS({}.{}, {})", bucket_name, exp_field, value)
                            } else {
                                format!("{}.{} {} {}", bucket_name, exp_field, comparison, value)
                            };
                        where_string.push_str(&expression);
                    }
                    None => unreachable!(),
                }
            } else if query_e_option.query.is_some() {
                match &query_e_option.query {
                    Some(q) => {
                        let query_group = q.as_n1ql(bucket_name)?;
                        where_string.push_str(&query_group);
                    }
                    None => unreachable!(),
                }
            } else {
                return Err(DataError::InvalidDataQueryItems);
            }
            item_count = item_count + 1;
        }
        where_string.push_str(")");

        Ok(where_string)
    }
}

impl DataQueryItems {
    pub fn generate_expression_for_string(
        field: impl Into<String>,
        comparison: DataQueryItemsExpressionComparison,
        value: impl Into<String>,
    ) -> Self {
        DataQueryItems {
            expression: Some(DataQueryItemsExpression {
                field: Some(field.into()),
                comparison: comparison as i32,
                field_type: DataQueryItemsExpressionFieldType::String as i32,
                value: Some(value.into()),
            }),
            ..Default::default()
        }
    }

    pub fn generate_expression_for_int(
        field: impl Into<String>,
        comparison: DataQueryItemsExpressionComparison,
        value: impl Into<String>,
    ) -> Self {
        DataQueryItems {
            expression: Some(DataQueryItemsExpression {
                field: Some(field.into()),
                comparison: comparison as i32,
                field_type: DataQueryItemsExpressionFieldType::Int as i32,
                value: Some(value.into()),
            }),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod query_test {

    use crate::query::{
        DataQuery, DataQueryBooleanTerm, DataQueryItems, DataQueryItemsExpression,
        DataQueryItemsExpressionComparison, DataQueryItemsExpressionFieldType,
    };

    #[test]
    fn single_term() {
        let query = DataQuery {
            items: vec![DataQueryItems {
                expression: Some(DataQueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: DataQueryItemsExpressionComparison::Equals as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] = \"bar\")");
    }

    #[test]
    fn single_term_contains() {
        let query = DataQuery {
            items: vec![DataQueryItems {
                expression: Some(DataQueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: DataQueryItemsExpressionComparison::Contains as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(ARRAY_CONTAINS(si.[\"foo\"], \"bar\"))");
    }

    #[test]
    fn single_term_like() {
        let query = DataQuery {
            items: vec![DataQueryItems {
                expression: Some(DataQueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: DataQueryItemsExpressionComparison::Like as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] LIKE \"bar\")");
    }

    #[test]
    fn single_term_notlike() {
        let query = DataQuery {
            items: vec![DataQueryItems {
                expression: Some(DataQueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: DataQueryItemsExpressionComparison::NotLike as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] NOT LIKE \"bar\")");
    }

    #[test]
    fn single_term_int_field() {
        let query = DataQuery {
            items: vec![DataQueryItems {
                expression: Some(DataQueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: DataQueryItemsExpressionComparison::Equals as i32,
                    value: Some("1".to_string()),
                    field_type: DataQueryItemsExpressionFieldType::Int as i32,
                }),
                ..Default::default()
            }],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] = 1)");
    }

    #[test]
    fn multi_term_and() {
        let query = DataQuery {
            items: vec![
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: DataQueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: DataQueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.[\"foo\"] = \"bar\" AND si.[\"freaky\"] != \"friday\")"
        );
    }

    #[test]
    fn multi_term_or() {
        let query = DataQuery {
            items: vec![
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: DataQueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: DataQueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: DataQueryBooleanTerm::Or as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.[\"foo\"] = \"bar\" OR si.[\"freaky\"] != \"friday\")"
        );
    }

    #[test]
    fn multi_group() {
        let query = DataQuery {
            items: vec![
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: DataQueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: DataQueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DataQueryItems {
                    query: Some(DataQuery {
                        items: vec![
                            DataQueryItems {
                                expression: Some(DataQueryItemsExpression {
                                    field: Some("parent".to_string()),
                                    comparison: DataQueryItemsExpressionComparison::Equals as i32,
                                    value: Some("teacher".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            DataQueryItems {
                                expression: Some(DataQueryItemsExpression {
                                    field: Some("loop".to_string()),
                                    comparison: DataQueryItemsExpressionComparison::Equals as i32,
                                    value: Some("canoe".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ],
                        boolean_term: DataQueryBooleanTerm::Or as i32,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.[\"foo\"] = \"bar\" AND si.[\"freaky\"] != \"friday\" AND (si.[\"parent\"] = \"teacher\" OR si.[\"loop\"] = \"canoe\"))"
        );
    }

    #[test]
    fn multi_group_not() {
        let query = DataQuery {
            items: vec![
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: DataQueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DataQueryItems {
                    expression: Some(DataQueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: DataQueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DataQueryItems {
                    query: Some(DataQuery {
                        items: vec![
                            DataQueryItems {
                                expression: Some(DataQueryItemsExpression {
                                    field: Some("parent".to_string()),
                                    comparison: DataQueryItemsExpressionComparison::Equals as i32,
                                    value: Some("teacher".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            DataQueryItems {
                                expression: Some(DataQueryItemsExpression {
                                    field: Some("loop".to_string()),
                                    comparison: DataQueryItemsExpressionComparison::Equals as i32,
                                    value: Some("canoe".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ],
                        boolean_term: DataQueryBooleanTerm::Or as i32,
                        is_not: Some(true),
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: DataQueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.[\"foo\"] = \"bar\" AND si.[\"freaky\"] != \"friday\" AND NOT (si.[\"parent\"] = \"teacher\" OR si.[\"loop\"] = \"canoe\"))"
        );
    }
}
