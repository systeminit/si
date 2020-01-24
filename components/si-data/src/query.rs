use crate::data::{
    query_expression_option, Query, QueryBooleanLogic, QueryComparison, QueryFieldType,
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

impl fmt::Display for QueryComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &QueryComparison::Equals => "=".to_string(),
            &QueryComparison::Notequals => "!=".to_string(),
            &QueryComparison::Contains => "contains".to_string(),
            &QueryComparison::Like => "LIKE".to_string(),
            &QueryComparison::Notlike => "NOT LIKE".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl fmt::Display for QueryBooleanLogic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            &QueryBooleanLogic::And => "AND".to_string(),
            &QueryBooleanLogic::Or => "OR".to_string(),
        };
        write!(f, "{}", msg)
    }
}

impl Query {
    pub fn as_n1ql(&self, bucket_name: &str) -> Result<String> {
        let mut where_string = String::new();
        if self.is_not {
            where_string.push_str("NOT ");
        }
        where_string.push_str("(");
        let mut item_count = 0;
        for query_e_option in self.items.iter() {
            if item_count != 0 {
                let qbl = match QueryBooleanLogic::from_i32(self.boolean_term) {
                    Some(l) => l,
                    None => return Err(DataError::InvalidBooleanLogic),
                };
                let boolean_term = format!(" {} ", qbl);
                where_string.push_str(&boolean_term);
            }
            match &query_e_option.qe {
                Some(query_expression_option::Qe::Expression(exp)) => {
                    let comparison = match QueryComparison::from_i32(exp.comparison) {
                        Some(c) => c,
                        None => return Err(DataError::InvalidQueryComparison),
                    };
                    let value = match QueryFieldType::from_i32(exp.field_type) {
                        Some(QueryFieldType::String) => json![exp.value],
                        Some(QueryFieldType::Int) => {
                            let vint: u64 = exp.value.parse()?;
                            json![vint]
                        }
                        None => return Err(DataError::InvalidFieldType),
                    };
                    let expression = if comparison == QueryComparison::Contains {
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
                Some(query_expression_option::Qe::Query(q)) => {
                    let query_group = q.as_n1ql(bucket_name)?;
                    where_string.push_str(&query_group);
                }
                None => return Err(DataError::InvalidQueryComparison),
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
        query_expression_option, Query, QueryBooleanLogic, QueryComparison, QueryExpression,
        QueryExpressionOption, QueryFieldType,
    };

    #[test]
    fn single_term() {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "foo".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    value: "bar".to_string(),
                    ..Default::default()
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] = \"bar\")");
    }

    #[test]
    fn single_term_contains() {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "foo".to_string(),
                    comparison: QueryComparison::Contains as i32,
                    value: "bar".to_string(),
                    ..Default::default()
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(ARRAY_CONTAINS(si.[\"foo\"], \"bar\"))");
    }

    #[test]
    fn single_term_like() {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "foo".to_string(),
                    comparison: QueryComparison::Like as i32,
                    value: "bar".to_string(),
                    ..Default::default()
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] LIKE \"bar\")");
    }

    #[test]
    fn single_term_notlike() {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "foo".to_string(),
                    comparison: QueryComparison::Notlike as i32,
                    value: "bar".to_string(),
                    ..Default::default()
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] NOT LIKE \"bar\")");
    }

    #[test]
    fn single_term_int_field() {
        let query = Query {
            items: vec![QueryExpressionOption {
                qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                    field: "foo".to_string(),
                    comparison: QueryComparison::Equals as i32,
                    value: "1".to_string(),
                    field_type: QueryFieldType::Int as i32,
                })),
            }],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(query_string, "(si.[\"foo\"] = 1)");
    }

    #[test]
    fn multi_term_and() {
        let query = Query {
            items: vec![
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "foo".to_string(),
                        comparison: QueryComparison::Equals as i32,
                        value: "bar".to_string(),
                        ..Default::default()
                    })),
                },
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "freaky".to_string(),
                        comparison: QueryComparison::Notequals as i32,
                        value: "friday".to_string(),
                        ..Default::default()
                    })),
                },
            ],
            boolean_term: QueryBooleanLogic::And as i32,
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
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "foo".to_string(),
                        comparison: QueryComparison::Equals as i32,
                        value: "bar".to_string(),
                        ..Default::default()
                    })),
                },
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "freaky".to_string(),
                        comparison: QueryComparison::Notequals as i32,
                        value: "friday".to_string(),
                        ..Default::default()
                    })),
                },
            ],
            boolean_term: QueryBooleanLogic::Or as i32,
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
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "foo".to_string(),
                        comparison: QueryComparison::Equals as i32,
                        value: "bar".to_string(),
                        ..Default::default()
                    })),
                },
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "freaky".to_string(),
                        comparison: QueryComparison::Notequals as i32,
                        value: "friday".to_string(),
                        ..Default::default()
                    })),
                },
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Query(Query {
                        items: vec![
                            QueryExpressionOption {
                                qe: Some(query_expression_option::Qe::Expression(
                                    QueryExpression {
                                        field: "parent".to_string(),
                                        comparison: QueryComparison::Equals as i32,
                                        value: "teacher".to_string(),
                                        ..Default::default()
                                    },
                                )),
                            },
                            QueryExpressionOption {
                                qe: Some(query_expression_option::Qe::Expression(
                                    QueryExpression {
                                        field: "loop".to_string(),
                                        comparison: QueryComparison::Equals as i32,
                                        value: "canoe".to_string(),
                                        ..Default::default()
                                    },
                                )),
                            },
                        ],
                        boolean_term: QueryBooleanLogic::Or as i32,
                        ..Default::default()
                    })),
                },
            ],
            boolean_term: QueryBooleanLogic::And as i32,
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
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "foo".to_string(),
                        comparison: QueryComparison::Equals as i32,
                        value: "bar".to_string(),
                        ..Default::default()
                    })),
                },
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Expression(QueryExpression {
                        field: "freaky".to_string(),
                        comparison: QueryComparison::Notequals as i32,
                        value: "friday".to_string(),
                        ..Default::default()
                    })),
                },
                QueryExpressionOption {
                    qe: Some(query_expression_option::Qe::Query(Query {
                        items: vec![
                            QueryExpressionOption {
                                qe: Some(query_expression_option::Qe::Expression(
                                    QueryExpression {
                                        field: "parent".to_string(),
                                        comparison: QueryComparison::Equals as i32,
                                        value: "teacher".to_string(),
                                        ..Default::default()
                                    },
                                )),
                            },
                            QueryExpressionOption {
                                qe: Some(query_expression_option::Qe::Expression(
                                    QueryExpression {
                                        field: "loop".to_string(),
                                        comparison: QueryComparison::Equals as i32,
                                        value: "canoe".to_string(),
                                        ..Default::default()
                                    },
                                )),
                            },
                        ],
                        boolean_term: QueryBooleanLogic::Or as i32,
                        is_not: true,
                    })),
                },
            ],
            boolean_term: QueryBooleanLogic::And as i32,
            ..Default::default()
        };
        let query_string = query.as_n1ql("si").expect("Failed to create n1ql query");
        assert_eq!(
            query_string,
            "(si.[\"foo\"] = \"bar\" AND si.[\"freaky\"] != \"friday\" AND NOT (si.[\"parent\"] = \"teacher\" OR si.[\"loop\"] = \"canoe\"))"
        );
    }
}
