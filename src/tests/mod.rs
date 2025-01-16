use swc_core::ecma::{
    ast::{self, *},
    parser::{EsSyntax, Syntax},
    transforms::testing::test_inline,
    visit::Fold,
};
use crate::{Config, visitor::JsxCssModulesVisitor};

struct AsFolder<T>(T);

impl<T: Fold> ast::Pass for AsFolder<T> {
    fn process(&mut self, program: &mut Program) {
        match program {
            Program::Module(module) => {
                *module = self.0.fold_module(module.clone());
            }
            _ => {}
        }
    }
}

fn as_folder<T: Fold>(t: T) -> impl ast::Pass {
    AsFolder(t)
}

fn syntax() -> Syntax {
    Syntax::Es(EsSyntax {
        jsx: true,
        ..Default::default()
    })
}

fn get_config() -> Config {
    Config {
        prefer: "local".to_string(),
        style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
        import_style: "default".to_string(),
    }
}

// test_existing_default_import
test_inline!(
    syntax(),
    |_| {
        let config = get_config();
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_existing_default_import,
    r#"
        import './styles.css';

        const Component = () => (
            <div className="container">
                <span className="text">Hello</span>
            </div>
        );
    "#,
    r#"
        import style_0 from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => <div className={_matcher("container")}>
                <span className={_matcher("text")}>Hello</span>
            </div>;
    "#
);

// test_multiple_style_imports
test_inline!(
    syntax(),
    |_| {
        let config = get_config();
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_multiple_style_imports,
    r#"
        import './style1.css';
        import './style2.scss';
        const Component = () => (
            <div className="container">
                <span className="text">Hello</span>
            </div>
        );
    "#,
    r#"
        import style_0 from './style1.css';
        import style_1 from './style2.scss';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0, style_1);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => 
            <div className={_matcher("container")}>
                <span className={_matcher("text")}>Hello</span>
            </div>
        ;
    "#
);

// test_global_and_local_classes
test_inline!(
    syntax(),
    |_| {
        let config = get_config();
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_global_and_local_classes,
    r#"
        import './styles.css';
        const Component = () => (
            <div className=":global(container) :local(wrapper) default-class">
                <span className="text :global(highlight)">Hello</span>
            </div>
        );
    "#,
    r#"
        import style_0 from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => 
            <div className={_matcher(":global(container) :local(wrapper) default-class")}>
                <span className={_matcher("text :global(highlight)")}>Hello</span>
            </div>;
    "#
);

// test_prefer_global
test_inline!(
    syntax(),
    |_| {
        let config = Config {
            prefer: "global".to_string(),
            style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
            import_style: "default".to_string(),
        };
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_prefer_global,
    r#"
        import './styles.css';
        const Component = () => (
            <div className="container">
                <span className=":local(text)">Hello</span>
            </div>
        );
    "#,
    r#"
        import style_0 from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0);
        const _matcher = getMatcher(_styles, 'global');
        const Component = () => 
            <div className={_matcher("container")}>
                <span className={_matcher(":local(text)")}>Hello</span>
            </div>;
    "#
);

// test_only_module_scss
test_inline!(
    syntax(),
    |_| {
        let config = Config {
            prefer: "local".to_string(),
            style_file_reg: vec![r"\.module\.scss$".to_string()],
            import_style: "default".to_string(),
        };
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_only_module_scss,
    r#"
        import './styles.css';
        import './foo.module.scss';
        import './bar.scss';
        const Component = () => (
            <div className="container">
                <span className="text">Hello</span>
            </div>
        );
    "#,
    r#"
        import './styles.css';
        import style_0 from './foo.module.scss';
        import './bar.scss';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => 
            <div className={_matcher("container")}>
                <span className={_matcher("text")}>Hello</span>
            </div>;
    "#
);

// test_comprehensive
test_inline!(
    syntax(),
    |_| {
        let config = Config {
            prefer: "global".to_string(),
            style_file_reg: vec![r"\.module\.scss$".to_string()],
            import_style: "default".to_string(),
        };
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_comprehensive,
    r#"
        import './normal.css';
        import './foo.module.scss';
        import './normal.scss';
        import './bar.module.scss';
        const Component = () => (
            <div className=":global(container) :local(wrapper)">
                <span className="text :global(highlight)">Hello</span>
                <button className=":local(button) primary">Click me</button>
            </div>
        );
    "#,
    r#"
        import './normal.css';
        import style_0 from './foo.module.scss';
        import './normal.scss';
        import style_1 from './bar.module.scss';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';
        const _styles = Object.assign({}, style_0, style_1);
        const _matcher = getMatcher(_styles, 'global');
        const Component = () => 
            <div className={_matcher(":global(container) :local(wrapper)")}>
                <span className={_matcher("text :global(highlight)")}>Hello</span>
                <button className={_matcher(":local(button) primary")}>Click me</button>
            </div>;
    "#
);

// Test namespace import style (import style_X)
test_inline!(
    syntax(),
    |_| {
        let config = Config {
            prefer: "local".to_string(),
            style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
            import_style: "namespace".to_string(),
        };
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_namespace_import_style,
    r#"
        import './styles.css';

        const Component = () => (
            <div className="container">Hello</div>
        );
    "#,
    r#"
        import * as style_0 from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';

        const _styles = Object.assign({}, style_0);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => 
            <div className={_matcher("container")}>Hello</div>;
    "#
);

// Test default import style (import style_X)
test_inline!(
    syntax(),
    |_| {
        let config = Config {
            prefer: "local".to_string(),
            style_file_reg: vec![r"\.(css|scss|sass|less)$".to_string()],
            import_style: "default".to_string(),
        };
        as_folder(JsxCssModulesVisitor::new(config))
    },
    test_default_import_style,
    r#"
        import './styles.css';

        const Component = () => (
            <div className="container">Hello</div>
        );
    "#,
    r#"
        import style_0 from './styles.css';
        import { getMatcher } from 'swc-plugin-jsx-css-modules/helpers';

        const _styles = Object.assign({}, style_0);
        const _matcher = getMatcher(_styles, 'local');
        const Component = () => 
            <div className={_matcher("container")}>Hello</div>;
    "#
);
