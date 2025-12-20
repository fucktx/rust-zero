use anyhow::{Context, Result};
use pest::Parser;
use pest_derive::Parser;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Ast {
    pub items: Vec<Item>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub enum Item {
    Service(Service),
    Route(Route),
}

#[derive(Debug, Clone)]
pub struct Service {
    pub name: String,
    pub annotations: Vec<Annotation>,
    pub routes: Vec<Route>,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub annotations: Vec<Annotation>,
    pub method: String,
    pub path: String,
    pub request: Option<String>,
    pub response: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub args: AnnotationArgs,
}

#[derive(Debug, Clone)]
pub enum AnnotationArgs {
    None,
    Str(String),
    Map(Vec<(String, String)>),
}

#[derive(Parser)]
#[grammar = "api/grammar.pest"]
struct ApiDslParser;

pub fn parse(input: &str) -> Result<Ast> {
    let mut pairs = ApiDslParser::parse(Rule::file, input).context("parse api dsl")?;
    let file = pairs
        .next()
        .context("parse api dsl: missing file pair")?;

    let mut items: Vec<Item> = Vec::new();
    let mut pending_annotations: Vec<Annotation> = Vec::new();

    for p in file.into_inner() {
        match p.as_rule() {
            Rule::annotation_stmt => {
                if let Some(ann) = parse_annotation_stmt(p)? {
                    pending_annotations.push(ann);
                }
            }
            Rule::route_stmt => {
                let mut route = parse_route_stmt(p)?;
                route.annotations = std::mem::take(&mut pending_annotations);
                items.push(Item::Route(route));
            }
            Rule::service_block => {
                let mut svc = parse_service_block(p)?;
                svc.annotations = std::mem::take(&mut pending_annotations);
                items.push(Item::Service(svc));
            }
            _ => {}
        }
    }

    Ok(Ast {
        items,
        source: input.to_string(),
    })
}

pub fn parse_file(path: impl AsRef<Path>) -> Result<Ast> {
    let path = path.as_ref();
    let input = std::fs::read_to_string(path)
        .with_context(|| format!("read api file: {}", path.display()))?;
    parse(&input).with_context(|| format!("parse api file: {}", path.display()))
}

fn parse_annotation_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Option<Annotation>> {
    let mut inner = pair.into_inner();
    let Some(p) = inner.next() else { return Ok(None) };
    if p.as_rule() != Rule::annotation {
        return Ok(None);
    }
    Ok(Some(parse_annotation(p)?))
}

fn parse_annotation(pair: pest::iterators::Pair<Rule>) -> Result<Annotation> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .context("annotation: missing name")?
        .as_str()
        .to_string();

    let args = match inner.next() {
        None => AnnotationArgs::None,
        Some(a) => match a.as_rule() {
            Rule::annotation_args => parse_annotation_args(a)?,
            _ => AnnotationArgs::None,
        },
    };

    Ok(Annotation { name, args })
}

fn parse_annotation_args(pair: pest::iterators::Pair<Rule>) -> Result<AnnotationArgs> {
    let mut inner = pair.into_inner();
    let Some(first) = inner.next() else { return Ok(AnnotationArgs::None) };
    match first.as_rule() {
        Rule::string => Ok(AnnotationArgs::Str(unquote(first.as_str()))),
        // 单值参数：@handler login / @server() / @doc foo 等
        // 这里把 bare/path/duration 统一当做字符串存起来。
        Rule::bare | Rule::path | Rule::duration => Ok(AnnotationArgs::Str(first.as_str().to_string())),
        Rule::kv_list => {
            let mut kvs = Vec::new();
            for kv in first.into_inner() {
                if kv.as_rule() == Rule::kv {
                    let mut it = kv.into_inner();
                    let k = it.next().context("kv: missing key")?.as_str().to_string();
                    let v = it.next().context("kv: missing value")?.as_str().to_string();
                    kvs.push((k, unquote_if_needed(&v)));
                }
            }
            Ok(AnnotationArgs::Map(kvs))
        }
        _ => Ok(AnnotationArgs::None),
    }
}

fn parse_route_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Route> {
    let mut inner = pair.into_inner();
    let method = inner
        .next()
        .context("route: missing method")?
        .as_str()
        .to_string();
    let path = inner
        .next()
        .context("route: missing path")?
        .as_str()
        .to_string();

    // Optional: type_name, type_name (from returns)
    let mut request: Option<String> = None;
    let mut response: Option<String> = None;
    for p in inner {
        if p.as_rule() == Rule::type_name {
            let t = p.as_str().to_string();
            if request.is_none() {
                request = Some(t);
            } else if response.is_none() {
                response = Some(t);
            }
        }
    }

    Ok(Route {
        annotations: vec![],
        method,
        path,
        request,
        response,
    })
}

fn parse_service_block(pair: pest::iterators::Pair<Rule>) -> Result<Service> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .context("service: missing name")?
        .as_str()
        .to_string();

    let mut routes: Vec<Route> = Vec::new();
    let mut pending_annotations: Vec<Annotation> = Vec::new();

    for p in inner {
        match p.as_rule() {
            Rule::annotation_stmt => {
                if let Some(ann) = parse_annotation_stmt(p)? {
                    pending_annotations.push(ann);
                }
            }
            Rule::route_stmt => {
                let mut route = parse_route_stmt(p)?;
                route.annotations = std::mem::take(&mut pending_annotations);
                routes.push(route);
            }
            _ => {}
        }
    }

    Ok(Service {
        name,
        annotations: vec![],
        routes,
    })
}

fn unquote(s: &str) -> String {
    if let Some(stripped) = s.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
        stripped.replace("\\\"", "\"")
    } else {
        s.to_string()
    }
}

fn unquote_if_needed(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        unquote(s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample_should_work() {
        let input = r#"
// 空内容
@server()

@doc "foo"

get /ping

@server (
  prefix: /v1
  group: Foo
)
service user {
  @doc "登录"
  @handler login
  post /user/login (LoginReq) returns (LoginResp)

  @handler getUserInfo
  get /user/info/:id (GetUserInfoReq) returns (GetUserInfoResp)
}
"#;

        let ast = parse(input).unwrap();
        assert!(!ast.items.is_empty());
    }
}


