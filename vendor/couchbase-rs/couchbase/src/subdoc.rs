use crate::error::CouchbaseError;
use serde::Serialize;
use serde_json::to_vec;
use std::ffi::CString;

#[derive(Debug)]
pub struct LookupInSpec {
    path: CString,
    path_len: usize,
    command_type: SubdocLookupCommandType,
    xattr: bool,
}

impl LookupInSpec {
    pub fn get<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        let path = path.into();
        LookupInSpec {
            path_len: path.len(),
            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocLookupCommandType::Get,
            xattr: false,
        }
    }

    pub fn get_full_document() -> Self {
        LookupInSpec {
            path_len: 0,
            path: CString::new("").expect("Could not encode path"),
            command_type: SubdocLookupCommandType::GetDoc,
            xattr: false,
        }
    }

    pub fn count<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        let path = path.into();
        LookupInSpec {
            path_len: path.len(),

            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocLookupCommandType::Count,
            xattr: false,
        }
    }

    pub fn exists<S>(path: S) -> Self
    where
        S: Into<String>,
    {
        let path = path.into();

        LookupInSpec {
            path_len: path.len(),

            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocLookupCommandType::Exists,
            xattr: false,
        }
    }

    pub fn xattr(mut self) -> Self {
        self.xattr = true;
        self
    }

    pub(crate) fn command_type(&self) -> &SubdocLookupCommandType {
        &self.command_type
    }

    pub(crate) fn path(&self) -> &CString {
        &self.path
    }

    pub(crate) fn path_len(&self) -> usize {
        self.path_len
    }
}

#[derive(Debug)]
pub enum SubdocLookupCommandType {
    Get,
    Exists,
    Count,
    GetDoc,
}

#[derive(Debug)]
pub struct MutateInSpec {
    path: CString,
    path_len: usize,
    command_type: SubdocMutationCommandType,
    xattr: bool,
    content: CString,
    content_len: usize,
}

impl MutateInSpec {
    pub fn insert<S, T>(path: S, content: T) -> Result<Self, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let path = path.into();
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let content_len = serialized.len();
        let converted = match CString::new(serialized) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };

        Ok(MutateInSpec {
            path_len: path.len(),
            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocMutationCommandType::Insert,
            xattr: false,
            content: converted,
            content_len: content_len,
        })
    }

    pub fn upsert<S, T>(path: S, content: T) -> Result<Self, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let path = path.into();
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let content_len = serialized.len();
        let converted = match CString::new(serialized) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };

        Ok(MutateInSpec {
            path_len: path.len(),
            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocMutationCommandType::Upsert,
            xattr: false,
            content: converted,
            content_len: content_len,
        })
    }

    pub fn replace<S, T>(path: S, content: T) -> Result<Self, CouchbaseError>
    where
        S: Into<String>,
        T: Serialize,
    {
        let path = path.into();
        let serialized = match to_vec(&content) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };
        let content_len = serialized.len();
        let converted = match CString::new(serialized) {
            Ok(v) => v,
            Err(_e) => return Err(CouchbaseError::EncodingError),
        };

        Ok(MutateInSpec {
            path_len: path.len(),
            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocMutationCommandType::Replace,
            xattr: false,
            content: converted,
            content_len: content_len,
        })
    }

    pub fn remove<S>(path: S) -> Result<Self, CouchbaseError>
    where
        S: Into<String>,
    {
        let path = path.into();
        Ok(MutateInSpec {
            path_len: path.len(),
            path: CString::new(path).expect("Could not encode path"),
            command_type: SubdocMutationCommandType::Remove,
            xattr: false,
            content: CString::new("").expect("??"),
            content_len: 0,
        })
    }

    pub(crate) fn command_type(&self) -> &SubdocMutationCommandType {
        &self.command_type
    }

    pub(crate) fn path(&self) -> &CString {
        &self.path
    }

    pub(crate) fn path_len(&self) -> usize {
        self.path_len
    }

    pub(crate) fn content(&self) -> &CString {
        &self.content
    }

    pub(crate) fn content_len(&self) -> usize {
        self.content_len
    }
}

#[derive(Debug)]
pub enum SubdocMutationCommandType {
    Insert,
    Upsert,
    Replace,
    Remove,
}
