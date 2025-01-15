use swc_core::{
    ecma::{
        ast::*,
        visit::{FoldWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_prefer")]
    pub prefer: String,
    #[serde(default = "default_style_file_reg")]
    pub style_file_reg: Vec<String>,
}

fn default_prefer() -> String {
    "local".to_string()
}

fn default_style_file_reg() -> Vec<String> {
    vec![r"\.(css|scss|sass|less)$".to_string()]
}

mod visitor;
pub use visitor::JsxCssModulesVisitor;

#[cfg(test)]
mod tests;

#[plugin_transform]
pub fn transform_program(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: Config = serde_json::from_str(&metadata.get_transform_plugin_config().unwrap_or_default()).unwrap_or_else(|_| Config {
        prefer: default_prefer(),
        style_file_reg: default_style_file_reg(),
    });
    let mut folder = JsxCssModulesVisitor::new(config);
    program.fold_with(&mut folder)
}
