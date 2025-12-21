use anyhow::{anyhow, Result};

pub fn to_spec(ast: &parse::api::Ast) -> Result<spec::api::Spec> {
    let mut services = Vec::new();
    let mut routes = Vec::new();
    let mut types = Vec::new();

    for item in &ast.items {
        match item {
            parse::api::Item::Service(s) => services.push(service_to_spec(s)?),
            parse::api::Item::Route(r) => routes.push(route_to_spec(r)?),
            parse::api::Item::Type(t) => types.push(type_to_spec(t)?),
        }
    }

    Ok(spec::api::Spec { services, routes, types })
}

fn type_to_spec(t: &parse::api::TypeDef) -> Result<spec::api::TypeDef> {
    Ok(spec::api::TypeDef {
        name: t.name.clone(),
        fields: t
            .fields
            .iter()
            .map(|f| spec::api::Field {
                name: f.name.clone(),
                ty: f.ty.clone(),
                tag: f.tag.clone(),
            })
            .collect(),
    })
}

fn service_to_spec(svc: &parse::api::Service) -> Result<spec::api::Service> {
    let mut routes = Vec::new();
    for r in &svc.routes {
        routes.push(route_to_spec(r)?);
    }

    Ok(spec::api::Service {
        name: svc.name.clone(),
        annotations: annotations_to_spec(&svc.annotations),
        routes,
    })
}

fn route_to_spec(route: &parse::api::Route) -> Result<spec::api::Route> {
    let method = parse_method(&route.method)?;

    Ok(spec::api::Route {
        annotations: annotations_to_spec(&route.annotations),
        method,
        path: route.path.clone(),
        request: route.request.clone(),
        response: route.response.clone(),
    })
}

fn parse_method(s: &str) -> Result<spec::api::HttpMethod> {
    match s.to_ascii_lowercase().as_str() {
        "get" => Ok(spec::api::HttpMethod::Get),
        "post" => Ok(spec::api::HttpMethod::Post),
        "put" => Ok(spec::api::HttpMethod::Put),
        "delete" => Ok(spec::api::HttpMethod::Delete),
        "patch" => Ok(spec::api::HttpMethod::Patch),
        other => Err(anyhow!("unsupported http method: {other}")),
    }
}

fn annotations_to_spec(src: &[parse::api::Annotation]) -> Vec<spec::api::Annotation> {
    src.iter()
        .map(|a| spec::api::Annotation {
            name: a.name.clone(),
            args: match &a.args {
                parse::api::AnnotationArgs::None => spec::api::AnnotationArgs::None,
                parse::api::AnnotationArgs::Str(s) => {
                    spec::api::AnnotationArgs::Str(s.clone())
                }
                parse::api::AnnotationArgs::Map(kvs) => {
                    spec::api::AnnotationArgs::Map(kvs.clone())
                }
            },
        })
        .collect()
}


