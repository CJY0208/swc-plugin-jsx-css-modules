use super::*;
use swc_core::ecma::parser::{Syntax, Parser, EsConfig};
use swc_core::common::sync::Lrc;
use swc_core::common::SourceMap;
use testing::assert_eq_ignore_whitespace;

fn parse_module(src: &str) -> Module {
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(
        swc_core::common::FileName::Anon,
        src.to_string(),
    );
    let config = EsConfig {
        jsx: true,
        ..Default::default()
    };
    let syntax = Syntax::Es(config);
    let mut parser = Parser::new(syntax, swc_core::ecma::parser::StringInput::from(&*fm), None);
    parser.parse_module().unwrap()
}

#[test]
fn test_multiple_style_imports() {
    let src = r#"
        import './style1.css';
        import './style2.scss';
        const Component = () => (
            <div className="container">
                <span className="text">Hello</span>
            </div>
        );
    "#;

    let expected = r#"
        import style_0 from './style1.css';
        import style_1 from './style2.scss';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0, style_1);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => (
            <div className={_matcher("container")}>
                <span className={_matcher("text")}>Hello</span>
            </div>
        );
    "#;

    let module = parse_module(src);
    let config = Config {
        prefer: "local".to_string(),
        style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
    };
    let transformed = module.fold_with(&mut as_folder(JsxCssModulesVisitor::new(config)));

    assert_eq_ignore_whitespace!(
        expected,
        format!("{:?}", transformed)
    );
}

#[test]
fn test_global_and_local_classes() {
    let src = r#"
        import './styles.css';
        const Component = () => (
            <div className=":global(container) :local(wrapper)">
                <span className="text :global(highlight)">Hello</span>
            </div>
        );
    "#;

    let expected = r#"
        import styles from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, styles);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => (
            <div className={_matcher(":global(container) :local(wrapper)")}>
                <span className={_matcher("text :global(highlight)")}>Hello</span>
            </div>
        );
    "#;

    let module = parse_module(src);
    let config = Config {
        prefer: "local".to_string(),
        style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
    };
    let transformed = module.fold_with(&mut as_folder(JsxCssModulesVisitor::new(config)));

    assert_eq_ignore_whitespace!(
        expected,
        format!("{:?}", transformed)
    );
}

#[test]
fn test_existing_default_import() {
    let src = r#"
        import myStyles from './styles.css';
        const Component = () => (
            <div className="container">Hello</div>
        );
    "#;

    let expected = r#"
        import myStyles from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, myStyles);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => (
            <div className={_matcher("container")}>Hello</div>
        );
    "#;

    let module = parse_module(src);
    let config = Config {
        prefer: "local".to_string(),
        style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
    };
    let transformed = module.fold_with(&mut as_folder(JsxCssModulesVisitor::new(config)));

    assert_eq_ignore_whitespace!(
        expected,
        format!("{:?}", transformed)
    );
}

#[test]
fn test_prefer_global() {
    let src = r#"
        import './styles.css';
        const Component = () => (
            <div className="container">
                <span className=":local(text)">Hello</span>
            </div>
        );
    "#;

    let expected = r#"
        import styles from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, styles);
        const _matcher = getMatcher(_styles, 'global');
        const Component = () => (
            <div className={_matcher("container")}>
                <span className={_matcher(":local(text)")}>Hello</span>
            </div>
        );
    "#;

    let module = parse_module(src);
    let config = Config {
        prefer: "global".to_string(),
        style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
    };
    let transformed = module.fold_with(&mut as_folder(JsxCssModulesVisitor::new(config)));

    assert_eq_ignore_whitespace!(
        expected,
        format!("{:?}", transformed)
    );
}
