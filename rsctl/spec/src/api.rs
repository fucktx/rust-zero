use serde::{Deserialize, Serialize};

/// Stable API spec IR (domain-first).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub services: Vec<Service>,
    /// Routes outside any `service {}` block.
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub annotations: Vec<Annotation>,
    pub method: HttpMethod,
    pub path: String,
    pub request: Option<String>,
    pub response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub name: String,
    pub args: AnnotationArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum AnnotationArgs {
    None,
    Str(String),
    Map(Vec<(String, String)>),
}


