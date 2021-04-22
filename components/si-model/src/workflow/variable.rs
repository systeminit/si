use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::lodash;
use crate::workflow::{WorkflowContext, WorkflowError, WorkflowResult};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Variable {
    String(VariableString),
    Number(VariableNumber),
    Bool(VariableBool),
    Array(VariableArray),
    Object(VariableObject),
    Args(VariableArgs),
    Context(VariableContext),
    Output(VariableOutput),
    Store(VariableStore),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum VariableScalar {
    Number(VariableNumber),
    Bool(VariableBool),
    String(VariableString),
    Args(VariableArgs),
    Context(VariableContext),
    Output(VariableOutput),
    Store(VariableStore),
}

impl VariableScalar {
    pub fn evaluate_as_string(&self, ctx: &WorkflowContext) -> WorkflowResult<String> {
        match self {
            VariableScalar::String(var) => Ok(var.value.clone()),
            VariableScalar::Args(var) => var.evaluate_as_string(&ctx),
            _ => todo!("fill in the remaining string evaluations!"),
        }
    }

    pub fn evaluate_as_bool(&self, ctx: &WorkflowContext) -> WorkflowResult<bool> {
        match self {
            VariableScalar::Bool(var) => Ok(var.value.clone()),
            VariableScalar::Args(var) => var.evaluate_as_bool(&ctx),
            _ => todo!("fill in the remaining string evaluations!"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum VariableRef {
    Args(VariableArgs),
    Context(VariableContext),
    Output(VariableOutput),
    Store(VariableStore),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableBool {
    pub value: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableString {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableNumber {
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableArgs {
    pub path: Vec<String>,
}

impl VariableArgs {
    pub fn evaluate_as_bool(&self, ctx: &WorkflowContext) -> WorkflowResult<bool> {
        match &ctx.args {
            Some(json) => {
                let result = lodash::get(&json, &self.path)?;
                match result {
                    Some(v) => {
                        if v.is_boolean() {
                            return Ok(v.as_bool().unwrap());
                        } else {
                            return Err(WorkflowError::WrongType("bool".to_string(), v.clone()));
                        }
                    }
                    None => {
                        return Err(WorkflowError::NoValue(
                            "bool".to_string(),
                            "args".to_string(),
                            self.path.join(", "),
                        ));
                    }
                }
            }
            None => {
                return Err(WorkflowError::NoValue(
                    "bool".to_string(),
                    "args".to_string(),
                    self.path.join(", "),
                ))
            }
        }
    }

    pub fn evaluate_as_string(&self, ctx: &WorkflowContext) -> WorkflowResult<String> {
        match &ctx.args {
            Some(json) => {
                let result = lodash::get(&json, &self.path)?;
                match result {
                    Some(v) => {
                        if v.is_string() {
                            return Ok(v.as_str().unwrap().to_string());
                        } else {
                            return Err(WorkflowError::WrongType("string".to_string(), v.clone()));
                        }
                    }
                    None => {
                        return Err(WorkflowError::NoValue(
                            "string".to_string(),
                            "args".to_string(),
                            self.path.join(", "),
                        ));
                    }
                }
            }
            None => {
                return Err(WorkflowError::NoValue(
                    "string".to_string(),
                    "args".to_string(),
                    self.path.join(", "),
                ))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableOutput {
    pub path: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableStore {
    pub path: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableContext {
    pub path: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum VariableObjectValue {
    Ref(VariableRef),
    Map(HashMap<String, Variable>),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableObject {
    pub value: VariableObjectValue,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum VariableArrayValue {
    Ref(VariableRef),
    List(Vec<Variable>),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VariableArray {
    pub value: VariableArrayValue,
}
