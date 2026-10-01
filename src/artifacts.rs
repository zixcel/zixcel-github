use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::ConnectorError;

const TEXT_SCHEMA: &str = "zixcel://github/text-artifact/v1";
const INPUTS_SCHEMA: &str = "zixcel://github/workflow-inputs-artifact/v1";
const SNAPSHOT_SCHEMA: &str = "zixcel://github/source-snapshot-artifact/v1";
const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubTextArtifact {
    pub schema: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubWorkflowInputsArtifact {
    pub schema: String,
    pub inputs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubSnapshotArtifact {
    pub schema: String,
    pub branch: String,
    pub message: String,
    pub files: Vec<GitHubSnapshotFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubSnapshotFile {
    pub path: String,
    pub mode: GitHubFileMode,
    pub content_base64: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum GitHubFileMode {
    #[serde(rename = "100644")]
    Regular,
    #[serde(rename = "100755")]
    Executable,
}

impl GitHubFileMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Regular => "100644",
            Self::Executable => "100755",
        }
    }
}

/// Parses one bounded UTF-8 text artifact.
///
/// # Errors
///
/// Rejects unknown fields, invalid schema, or text larger than 64 KiB.
pub fn parse_text_artifact(bytes: &[u8]) -> Result<GitHubTextArtifact, ConnectorError> {
    let value: GitHubTextArtifact = decode(bytes)?;
    if value.schema != TEXT_SCHEMA || value.text.len() > 65_536 {
        return Err(invalid());
    }
    Ok(value)
}

/// Parses one bounded workflow-input map.
///
/// # Errors
///
/// Rejects invalid names, values, schema, count, or unknown fields.
pub fn parse_workflow_inputs_artifact(
    bytes: &[u8],
) -> Result<GitHubWorkflowInputsArtifact, ConnectorError> {
    let value: GitHubWorkflowInputsArtifact = decode(bytes)?;
    if value.schema != INPUTS_SCHEMA
        || value.inputs.len() > 64
        || value.inputs.iter().any(|(key, item)| {
            !token(key, 128)
                || item.len() > 1024
                || item
                    .bytes()
                    .any(|byte| byte.is_ascii_control() && !matches!(byte, b'\n' | b'\r' | b'\t'))
        })
    {
        return Err(invalid());
    }
    Ok(value)
}

/// Parses a complete immutable source snapshot for the Git Data API.
///
/// # Errors
///
/// Rejects an invalid schema, branch, path, duplicate, mode, base64 payload,
/// count, or aggregate decoded size.
pub fn parse_snapshot_artifact(bytes: &[u8]) -> Result<GitHubSnapshotArtifact, ConnectorError> {
    let value: GitHubSnapshotArtifact = decode(bytes)?;
    if value.schema != SNAPSHOT_SCHEMA
        || !git_branch(&value.branch)
        || value.message.is_empty()
        || value.message.len() > 512
        || value.files.is_empty()
        || value.files.len() > 4096
    {
        return Err(invalid());
    }
    let mut paths = BTreeSet::new();
    let mut size = 0_usize;
    for file in &value.files {
        if !path(&file.path) || !paths.insert(file.path.as_str()) {
            return Err(invalid());
        }
        let content = STANDARD
            .decode(&file.content_base64)
            .map_err(|_| invalid())?;
        size = size.checked_add(content.len()).ok_or_else(invalid)?;
        if content.len() > 1_048_576 || size > 8 * 1024 * 1024 {
            return Err(invalid());
        }
    }
    Ok(value)
}

fn decode<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, ConnectorError> {
    if bytes.is_empty() || bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(invalid());
    }
    serde_json::from_slice(bytes).map_err(|_| invalid())
}

fn token(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && value.trim() == value
        && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn path(value: &str) -> bool {
    token(value, 512)
        && !value.contains('\\')
        && !value
            .split('/')
            .any(|part| part.eq_ignore_ascii_case(".git"))
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value
            .split('/')
            .any(|item| item.is_empty() || matches!(item, "." | ".."))
}

fn invalid() -> ConnectorError {
    ConnectorError::new("artifact", "invalid GitHub artifact")
}

pub(crate) fn git_branch(value: &str) -> bool {
    token(value, 128)
        && !value.starts_with('-')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.ends_with('.')
        && !value.contains("..")
        && !value.contains("@{")
        && !value.contains("//")
        && !value.bytes().any(|b| b" ~^:?*[\\".contains(&b))
        && !value
            .split('/')
            .any(|p| p.starts_with('.') || p.ends_with(".lock"))
}
