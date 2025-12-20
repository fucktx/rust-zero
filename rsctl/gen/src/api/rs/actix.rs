#[derive(Debug, Clone)]
pub struct Options {
    pub service_name: String,
    pub merge: bool,
    pub style: String,
    pub template_root: std::path::PathBuf,
}

pub fn generate(spec: &spec::api::Spec, opts: &Options) -> anyhow::Result<crate::artifact::Artifacts> {
    super::shared::generate_go_like_tree(
        "actix",
        spec,
        &opts.service_name,
        opts.merge,
        &opts.style,
        &opts.template_root,
    )
}


