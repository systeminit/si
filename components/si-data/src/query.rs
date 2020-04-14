pub use crate::data::{
    Query, QueryBooleanTerm, QueryItems, QueryItemsExpression, QueryItemsExpressionComparison,
    QueryItemsExpressionFieldType,
};
use crate::error::{DataError, Result};

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

impl fmt::Display for QueryItemsExpressionComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &QueryItemsExpressionComparison::Unknown => "UNKNOWN".to_string(),
            &QueryItemsExpressionComparison::Equals => "=".to_string(),
            &QueryItemsExpressionComparison::NotEquals => "!=".to_string(),
            &QueryItemsExpressionComparison::Contains => "contains".to_string(),
            &QueryItemsExpressionComparison::Like => "LIKE".to_string(),
            &QueryItemsExpressionComparison::NotLike => "NOT LIKE".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl fmt::Display for QueryBooleanTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &QueryBooleanTerm::Unknown => "UNKNOWN".to_string(),
            &QueryBooleanTerm::And => "AND".to_string(),
            &QueryBooleanTerm::Or => "OR".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl Query {
    pub fn generate_expression_for_string(
        field: impl Into<String>,
        comparison: QueryItemsExpressionComparison,
        value: impl Into<String>,
    ) -> QueryItems {
        QueryItems {
            expression: Some(QueryItemsExpression {
                field: Some(field.into()),
                comparison: comparison as i32,
                field_type: QueryItemsExpressionFieldType::String as i32,
                value: Some(value.into()),
            }),
            ..Default::default()
        }
    }

    pub fn generate_expression_for_int(
        field: impl Into<String>,
        comparison: QueryItemsExpressionComparison,
        value: impl Into<String>,
    ) -> QueryItems {
        QueryItems {
            expression: Some(QueryItemsExpression {
                field: Some(field.into()),
                comparison: comparison as i32,
                field_type: QueryItemsExpressionFieldType::Int as i32,
                value: Some(value.into()),
            }),
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
                let qbl = match QueryBooleanTerm::from_i32(self.boolean_term) {
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
                            match QueryItemsExpressionComparison::from_i32(exp.comparison) {
                                Some(c) => c,
                                None => return Err(DataError::InvalidQueryComparison),
                            };
                        let value = match QueryItemsExpressionFieldType::from_i32(exp.field_type) {
                            Some(QueryItemsExpressionFieldType::Unknown) => json![exp.value],
                            Some(QueryItemsExpressionFieldType::String) => json![exp.value],
                            Some(QueryItemsExpressionFieldType::Int) => {
                                // Elizabeth Jacob fixed this bug, her first, on 04-13-2020.
                                // Good job, Monkey.
                                let vint: u64 = exp.value.as_ref().unwrap_or(&"".into()).parse()?;
                                json![vint]
                            }
                            None => return Err(DataError::InvalidFieldType),
                        };
                        let expression = if comparison == QueryItemsExpressionComparison::Contains {
                            format!(
                                "ARRAY_CONTAINS({}.[{}], {})",
                                bucket_name,
                                json![exp.field],
                                value
                            )
                        } else {
                            format!(
                                "{}.[{}] {} {}",
                                bucket_name,
                                json![exp.field],
                                comparison,
                                value
                            )
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
                return Err(DataError::InvalidQueryItems);
            }
            item_count = item_count + 1;
        }
        where_string.push_str(")");

        Ok(where_string)
    }
}

#[cfg(test)]
mod query_test {

    use crate::data::{
        Query, QueryBooleanTerm, QueryItems, QueryItemsExpression, QueryItemsExpressionComparison,
        QueryItemsExpressionFieldType,
    };

    #[test]
    fn single_term() {
        let query = Query {
            items: vec![QueryItems {
                expression: Some(QueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: QueryItemsExpressionComparison::Equals as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: QueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] = \"bar\")");
    }

    #[test]
    fn single_term_contains() {
        let query = Query {
            items: vec![QueryItems {
                expression: Some(QueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: QueryItemsExpressionComparison::Contains as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: QueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(ARRAY_CONTAINS(si.[\"foo\"], \"bar\"))");
    }

    #[test]
    fn single_term_like() {
        let query = Query {
            items: vec![QueryItems {
                expression: Some(QueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: QueryItemsExpressionComparison::Like as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: QueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] LIKE \"bar\")");
    }

    #[test]
    fn single_term_notlike() {
        let query = Query {
            items: vec![QueryItems {
                expression: Some(QueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: QueryItemsExpressionComparison::NotLike as i32,
                    value: Some("bar".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            boolean_term: QueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] NOT LIKE \"bar\")");
    }

    #[test]
    fn single_term_int_field() {
        let query = Query {
            items: vec![QueryItems {
                expression: Some(QueryItemsExpression {
                    field: Some("foo".to_string()),
                    comparison: QueryItemsExpressionComparison::Equals as i32,
                    value: Some("1".to_string()),
                    field_type: QueryItemsExpressionFieldType::Int as i32,
                }),
                ..Default::default()
            }],
            boolean_term: QueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] = 1)");
    }

    #[test]
    fn multi_term_and() {
        let query = Query {
            items: vec![
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: QueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: QueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: QueryBooleanTerm::And as i32,
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
        let query = Query {
            items: vec![
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: QueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: QueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: QueryBooleanTerm::Or as i32,
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
        let query = Query {
            items: vec![
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: QueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: QueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                QueryItems {
                    query: Some(Query {
                        items: vec![
                            QueryItems {
                                expression: Some(QueryItemsExpression {
                                    field: Some("parent".to_string()),
                                    comparison: QueryItemsExpressionComparison::Equals as i32,
                                    value: Some("teacher".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            QueryItems {
                                expression: Some(QueryItemsExpression {
                                    field: Some("loop".to_string()),
                                    comparison: QueryItemsExpressionComparison::Equals as i32,
                                    value: Some("canoe".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ],
                        boolean_term: QueryBooleanTerm::Or as i32,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: QueryBooleanTerm::And as i32,
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
        let query = Query {
            items: vec![
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("foo".to_string()),
                        comparison: QueryItemsExpressionComparison::Equals as i32,
                        value: Some("bar".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                QueryItems {
                    expression: Some(QueryItemsExpression {
                        field: Some("freaky".to_string()),
                        comparison: QueryItemsExpressionComparison::NotEquals as i32,
                        value: Some("friday".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                QueryItems {
                    query: Some(Query {
                        items: vec![
                            QueryItems {
                                expression: Some(QueryItemsExpression {
                                    field: Some("parent".to_string()),
                                    comparison: QueryItemsExpressionComparison::Equals as i32,
                                    value: Some("teacher".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                            QueryItems {
                                expression: Some(QueryItemsExpression {
                                    field: Some("loop".to_string()),
                                    comparison: QueryItemsExpressionComparison::Equals as i32,
                                    value: Some("canoe".to_string()),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        ],
                        boolean_term: QueryBooleanTerm::Or as i32,
                        is_not: Some(true),
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: QueryBooleanTerm::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.[\"foo\"] = \"bar\" AND si.[\"freaky\"] != \"friday\" AND NOT (si.[\"parent\"] = \"teacher\" OR si.[\"loop\"] = \"canoe\"))"
        );
    }
}
