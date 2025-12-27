use anyhow::{Result, anyhow};

pub fn to_spec(ast: &crate::parse::api::Ast) -> Result<crate::spec::api::Spec> {
    let mut services = Vec::new();
    let mut routes = Vec::new();
    let mut types = Vec::new();

    for item in &ast.items {
        match item {
            crate::parse::api::Item::Service(s) => services.push(service_to_spec(s)?),
            crate::parse::api::Item::Route(r) => routes.push(route_to_spec(r)?),
            crate::parse::api::Item::Type(t) => types.push(type_to_spec(t)?),
        }
    }

    Ok(crate::spec::api::Spec {
        services,
        routes,
        types,
    })
}

fn type_to_spec(t: &crate::parse::api::TypeDef) -> Result<crate::spec::api::TypeDef> {
    Ok(crate::spec::api::TypeDef {
        name: t.name.clone(),
        fields: t
            .fields
            .iter()
            .map(|f| crate::spec::api::Field {
                name: f.name.clone(),
                ty: f.ty.clone(),
                tag: f.tag.clone(),
            })
            .collect(),
    })
}

fn service_to_spec(svc: &crate::parse::api::Service) -> Result<crate::spec::api::Service> {
    let mut routes = Vec::new();
    for r in &svc.routes {
        routes.push(route_to_spec(r)?);
    }

    Ok(crate::spec::api::Service {
        name: svc.name.clone(),
        annotations: annotations_to_spec(&svc.annotations),
        routes,
    })
}

fn route_to_spec(route: &crate::parse::api::Route) -> Result<crate::spec::api::Route> {
    let method = parse_method(&route.method)?;

    Ok(crate::spec::api::Route {
        annotations: annotations_to_spec(&route.annotations),
        method,
        path: route.path.clone(),
        request: route.request.clone(),
        response: route.response.clone(),
    })
}

fn parse_method(s: &str) -> Result<crate::spec::api::HttpMethod> {
    match s.to_ascii_lowercase().as_str() {
        "get" => Ok(crate::spec::api::HttpMethod::Get),
        "post" => Ok(crate::spec::api::HttpMethod::Post),
        "put" => Ok(crate::spec::api::HttpMethod::Put),
        "delete" => Ok(crate::spec::api::HttpMethod::Delete),
        "patch" => Ok(crate::spec::api::HttpMethod::Patch),
        other => Err(anyhow!("unsupported http method: {other}")),
    }
}

fn annotations_to_spec(src: &[crate::parse::api::Annotation]) -> Vec<crate::spec::api::Annotation> {
    src.iter()
        .map(|a| crate::spec::api::Annotation {
            name: a.name.clone(),
            args: match &a.args {
                crate::parse::api::AnnotationArgs::None => crate::spec::api::AnnotationArgs::None,
                crate::parse::api::AnnotationArgs::Str(s) => {
                    crate::spec::api::AnnotationArgs::Str(s.clone())
                }
                crate::parse::api::AnnotationArgs::Map(kvs) => {
                    crate::spec::api::AnnotationArgs::Map(kvs.clone())
                }
            },
        })
        .collect()
}
