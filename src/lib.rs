use swc_core::ecma::{
    parser::{EsConfig, Syntax},
    visit::{as_folder, Fold},
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

pub fn jsx_css_modules(config: Config) -> impl Fold {
    as_folder(JsxCssModulesVisitor::new(config))
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
import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
const _styles = Object.assign({}, styles);
const _matcher = getMatcher(_styles, 'local');
const Component = () => <div className={_matcher("container")}>
        <span className={_matcher("text")}>Hello</span>
    </div>;
"#
    );
}
