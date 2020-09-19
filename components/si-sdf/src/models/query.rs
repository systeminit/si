use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use std::fmt;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("a query with multiple items requires a boolean term")]
    MissingBooleanLogic,
    #[error("a query must contain either an expression or another query")]
    MissingExpressionOrQuery,
    #[error("a query field should be an integer, but it wasn't: {0}")]
    IntegerError(#[from] std::num::ParseIntError),
}

pub type QueryResult<T> = Result<T, QueryError>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum FieldType {
    String,
    Int,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Comparison {
    Equals,
    NotEquals,
    Contains,
    Like,
    NotLike,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Expression {
    pub field: String,
    pub value: String,
    pub comparison: Comparison,
    pub field_type: FieldType,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub query: Option<Query>,
    pub expression: Option<Expression>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum BooleanTerm {
    And,
    Or,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub boolean_term: Option<BooleanTerm>,
    pub is_not: Option<bool>,
    pub items: Vec<Item>,
}

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

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &Comparison::Equals => "=".to_string(),
            &Comparison::NotEquals => "!=".to_string(),
            &Comparison::Contains => "CONTAINS".to_string(),
            &Comparison::Like => "LIKE".to_string(),
            &Comparison::NotLike => "NOT LIKE".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl fmt::Display for BooleanTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &BooleanTerm::And => "AND".to_string(),
            &BooleanTerm::Or => "OR".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl Query {
    pub fn generate_for_string(
        field: impl Into<String>,
        comparison: Comparison,
        value: impl Into<String>,
    ) -> Self {
        Query {
            items: vec![Item::generate_expression_for_string(
                field, comparison, value,
            )],
            ..Default::default()
        }
    }

    pub fn generate_for_int(
        field: impl Into<String>,
        comparison: Comparison,
        value: impl Into<String>,
    ) -> Query {
        Query {
            items: vec![Item::generate_expression_for_int(field, comparison, value)],
            ..Default::default()
        }
    }

    pub fn as_n1ql(&self, bucket_name: &str) -> QueryResult<String> {
        let mut where_string = String::new();
        if self.is_not.unwrap_or(false) {
            where_string.push_str("NOT ");
        }
        where_string.push_str("(");
        let mut item_count = 0;
        for query_e_option in self.items.iter() {
            if item_count != 0 {
                let qbl = match &self.boolean_term {
                    Some(l) => l,
                    None => return Err(QueryError::MissingBooleanLogic),
                };
                let boolean_term = format!(" {} ", qbl);
                where_string.push_str(&boolean_term);
            }
            if query_e_option.expression.is_some() {
                match &query_e_option.expression {
                    Some(exp) => {
                        let value = match exp.field_type {
                            FieldType::String => json![exp.value],
                            FieldType::Int => {
                                // Elizabeth Jacob fixed this bug, her first, on 04-13-2020.
                                // Good job, Monkey.
                                let vint: u64 = exp.value.parse()?;
                                //exp.value.as_ref().unwrap_or(&"".into()).parse()?;
                                json![vint]
                            }
                        };
                        let field = &exp.field;
                        let exp_field = if field.contains(".") {
                            let mut escaped_fields = Vec::new();
                            for field_part in field.split(".") {
                                let escaped = format!("`{}`", field_part);
                                escaped_fields.push(escaped);
                            }
                            escaped_fields.join(".")
                        } else {
                            field.clone()
                        };
                        let expression = if exp.comparison == Comparison::Contains {
                            format!("ARRAY_CONTAINS({}.{}, {})", bucket_name, exp_field, value)
                        } else {
                            format!("{}.{} {} {}", bucket_name, exp_field, exp.comparison, value)
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
                return Err(QueryError::MissingExpressionOrQuery);
            }
            item_count = item_count + 1;
        }
        where_string.push_str(")");

        Ok(where_string)
    }
}

impl Item {
    pub fn generate_expression_for_string(
        field: impl Into<String>,
        comparison: Comparison,
        value: impl Into<String>,
    ) -> Self {
        let field = field.into();
        let value = value.into();
        Item {
            expression: Some(Expression {
                field,
                comparison,
                field_type: FieldType::String,
                value,
            }),
            ..Default::default()
        }
    }

    pub fn generate_expression_for_int(
        field: impl Into<String>,
        comparison: Comparison,
        value: impl Into<String>,
    ) -> Self {
        let field = field.into();
        let value = value.into();
        Item {
            expression: Some(Expression {
                field,
                comparison,
                field_type: FieldType::Int,
                value,
            }),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod query_test {
    use super::{BooleanTerm, Comparison, Expression, FieldType, Item, Query};

    #[test]
    fn single_term() {
        let query = Query {
            items: vec![Item {
                expression: Some(Expression {
                    field: "foo".to_string(),
                    comparison: Comparison::Equals,
                    value: "bar".to_string(),
                    field_type: FieldType::String,
                }),
                ..Default::default()
            }],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.foo = \"bar\")");
    }

    #[test]
    fn single_term_contains() {
        let query = Query {
            items: vec![Item {
                expression: Some(Expression {
                    field: "foo".to_string(),
                    comparison: Comparison::Contains,
                    value: "bar".to_string(),
                    field_type: FieldType::String,
                }),
                ..Default::default()
            }],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(ARRAY_CONTAINS(si.foo, \"bar\"))");
    }

    #[test]
    fn single_term_like() {
        let query = Query {
            items: vec![Item {
                expression: Some(Expression {
                    field: "foo".to_string(),
                    comparison: Comparison::Like,
                    value: "bar".to_string(),
                    field_type: FieldType::String,
                }),
                ..Default::default()
            }],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.foo LIKE \"bar\")");
    }

    #[test]
    fn single_term_notlike() {
        let query = Query {
            items: vec![Item {
                expression: Some(Expression {
                    field: "foo".to_string(),
                    comparison: Comparison::NotLike,
                    value: "bar".to_string(),
                    field_type: FieldType::String,
                }),
                ..Default::default()
            }],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.foo NOT LIKE \"bar\")");
    }

    #[test]
    fn single_term_int_field() {
        let query = Query {
            items: vec![Item {
                expression: Some(Expression {
                    field: "foo".to_string(),
                    comparison: Comparison::Equals,
                    value: "1".to_string(),
                    field_type: FieldType::Int,
                }),
                ..Default::default()
            }],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.foo = 1)");
    }

    #[test]
    fn multi_term_and() {
        let query = Query {
            items: vec![
                Item {
                    expression: Some(Expression {
                        field: "foo".to_string(),
                        comparison: Comparison::Equals,
                        value: "bar".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
                Item {
                    expression: Some(Expression {
                        field: "freaky".to_string(),
                        comparison: Comparison::NotEquals,
                        value: "friday".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.foo = \"bar\" AND si.freaky != \"friday\")"
        );
    }

    #[test]
    fn multi_term_or() {
        let query = Query {
            items: vec![
                Item {
                    expression: Some(Expression {
                        field: "foo".to_string(),
                        comparison: Comparison::Equals,
                        value: "bar".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
                Item {
                    expression: Some(Expression {
                        field: "freaky".to_string(),
                        comparison: Comparison::NotEquals,
                        value: "friday".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: Some(BooleanTerm::Or),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.foo = \"bar\" OR si.freaky != \"friday\")"
        );
    }

    #[test]
    fn multi_group() {
        let query = Query {
            items: vec![
                Item {
                    expression: Some(Expression {
                        field: "foo".to_string(),
                        comparison: Comparison::Equals,
                        value: "bar".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
                Item {
                    expression: Some(Expression {
                        field: "freaky".to_string(),
                        comparison: Comparison::NotEquals,
                        value: "friday".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
                Item {
                    query: Some(Query {
                        items: vec![
                            Item {
                                expression: Some(Expression {
                                    field: "parent".to_string(),
                                    comparison: Comparison::Equals,
                                    value: "teacher".to_string(),
                                    field_type: FieldType::String,
                                }),
                                ..Default::default()
                            },
                            Item {
                                expression: Some(Expression {
                                    field: "loop".to_string(),
                                    comparison: Comparison::Equals,
                                    value: "canoe".to_string(),
                                    field_type: FieldType::String,
                                }),
                                ..Default::default()
                            },
                        ],
                        boolean_term: Some(BooleanTerm::Or),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.foo = \"bar\" AND si.freaky != \"friday\" AND (si.parent = \"teacher\" OR si.loop = \"canoe\"))"
        );
    }

    #[test]
    fn multi_group_not() {
        let query = Query {
            items: vec![
                Item {
                    expression: Some(Expression {
                        field: "foo".to_string(),
                        comparison: Comparison::Equals,
                        value: "bar".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
                Item {
                    expression: Some(Expression {
                        field: "freaky".to_string(),
                        comparison: Comparison::NotEquals,
                        value: "friday".to_string(),
                        field_type: FieldType::String,
                    }),
                    ..Default::default()
                },
                Item {
                    query: Some(Query {
                        items: vec![
                            Item {
                                expression: Some(Expression {
                                    field: "parent".to_string(),
                                    comparison: Comparison::Equals,
                                    value: "teacher".to_string(),
                                    field_type: FieldType::String,
                                }),
                                ..Default::default()
                            },
                            Item {
                                expression: Some(Expression {
                                    field: "loop".to_string(),
                                    comparison: Comparison::Equals,
                                    value: "canoe".to_string(),
                                    field_type: FieldType::String,
                                }),
                                ..Default::default()
                            },
                        ],
                        boolean_term: Some(BooleanTerm::Or),
                        is_not: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            boolean_term: Some(BooleanTerm::And),
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.foo = \"bar\" AND si.freaky != \"friday\" AND NOT (si.parent = \"teacher\" OR si.loop = \"canoe\"))"
        );
    }
}
