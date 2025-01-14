use swc_core::{
    ecma::{
        ast::Program,
        parser::{EsConfig, Syntax},
        visit::{as_folder, FoldWith, Fold},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_prefer")]
    prefer: String,
    #[serde(default = "default_style_file_reg")]
    style_file_reg: Vec<String>,
}

fn default_prefer() -> String {
    "local".to_string()
}

fn default_style_file_reg() -> Vec<String> {
    vec![r"\.(css|scss|sass|less)$".to_string()]
}

mod visitor;
use visitor::JsxCssModulesVisitor;

#[plugin_transform]
pub fn transform_program(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let config: Config = serde_json::from_str(&metadata.get_transform_plugin_config().unwrap_or_default()).unwrap_or_else(|_| Config {
        prefer: default_prefer(),
        style_file_reg: default_style_file_reg(),
    });
    let mut folder = as_folder(JsxCssModulesVisitor::new(config));
    program.fold_with(&mut folder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::ecma::transforms::testing::test_inline;

    fn syntax() -> Syntax {
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        })
    }

    test_inline!(
        syntax(),
        |_| {
            let config = Config {
                prefer: "local".to_string(),
                style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
            };
            as_folder(JsxCssModulesVisitor::new(config))
        },
        basic_transform,
        r#"
import './styles.css';

const Component = () => (
    <div className="container">
        <span className="text">Hello</span>
    </div>
);
"#,
        r#"
import styles from './styles.css';

const Component = () => (
    <div className={styles.container}>
        <span className={styles.text}>Hello</span>
    </div>
);
"#
    );
}
