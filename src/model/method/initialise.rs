use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Params {
    #[serde(rename = "processId")]
    process_id: Option<i32>,
    #[serde(rename = "clientInfo")]
    client_info: Option<ClientInfo>,
    locale: Option<String>,
    #[serde(rename = "initializationOptions")]
    initialisation_options: Option<serde_json::Value>,
    capabilities: ClientCapabilities,
    #[serde(default)]
    trace: TraceValue,
    #[serde(rename = "workspaceFolders")]
    workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct ClientInfo {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct ClientCapabilities {
    #[serde(rename = "textDocument")]
    text_document: Option<TextDocumentClientCapabilities>,
    general: Option<GeneralClientCapabilities>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct TextDocumentClientCapabilities {
    diagnostic: Option<DiagnosticClientCapabilities>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase")]
struct DiagnosticClientCapabilities {
    dynamic_registration: Option<bool>,
    related_document_support: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct GeneralClientCapabilities {
    #[serde(rename = "positionEncodings")]
    position_encodings: Option<Vec<PositionEncoding>>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "lowercase")]
enum TraceValue {
    Off,
    Messages,
    Verbose,
}

impl Default for TraceValue {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct WorkspaceFolder {
    uri: String, // TODO: use real URI
    name: String,
}

#[derive(Debug, Serialize, Default)]
pub struct Result {
    capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    server_info: ServerInfo,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ServerCapabilities {
    position_encoding: PositionEncoding,
    text_document_sync: TextDocumentSync,
    diagnostic_provider: DiagnosticOptions,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
enum PositionEncoding {
    #[serde(rename = "utf-16")]
    Utf16,
}

impl Default for PositionEncoding {
    fn default() -> Self {
        Self::Utf16
    }
}

#[derive(Debug, Serialize)]
struct TextDocumentSync {
    #[serde(rename = "openClose")]
    open_close: bool,
    change: TextDocumentSyncKind,
}

impl Default for TextDocumentSync {
    fn default() -> Self {
        Self {
            open_close: true,
            change: TextDocumentSyncKind::default(),
        }
    }
}

// TODO: numerical representation
#[derive(Debug, Serialize)]
enum TextDocumentSyncKind {
    None,
    Incremental,
    Full,
}

impl Default for TextDocumentSyncKind {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Debug, Serialize)]
struct DiagnosticOptions {
    identifier: String,
    #[serde(rename = "interFileDependencies")]
    inter_file_dependencies: bool,
    #[serde(rename = "workspaceDiagnostics")]
    workspace_diagnostics: bool,
}

impl Default for DiagnosticOptions {
    fn default() -> Self {
        Self {
            identifier: env!("CARGO_PKG_NAME").into(),
            inter_file_dependencies: false,
            workspace_diagnostics: false,
        }
    }
}

#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

#[derive(Debug, Serialize, Default)]
pub struct Error {
    retry: bool,
}

impl Error {
    pub fn to_value(&self) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        object.insert("retry".into(), serde_json::Value::Bool(self.retry));
        serde_json::Value::Object(object)
    }
}
