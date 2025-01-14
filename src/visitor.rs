use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
};
use swc_core::common::Span;
use regex::Regex;
use super::Config;

pub struct JsxCssModulesVisitor {
    config: Config,
    style_imports: Vec<ImportDecl>,
    styles_ident: Option<Ident>,
    matcher_ident: Option<Ident>,
}

impl JsxCssModulesVisitor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            style_imports: Vec::new(),
            styles_ident: None,
            matcher_ident: None,
        }
    }

    fn is_style_import(&self, import: &ImportDecl) -> bool {
        for pattern in &self.config.style_file_reg {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(&import.src.value.to_string()) {
                    return true;
                }
            }
        }
        false
    }

    fn create_merged_styles_stmt(&mut self, styles: Vec<Ident>) -> Stmt {
        let styles_ident = Ident::new(
            "_styles".into(),
            Span::default(),
        );
        self.styles_ident = Some(styles_ident.clone());

        Stmt::Decl(Decl::Var(Box::new(VarDecl {
            span: Span::default(),
            kind: VarDeclKind::Const,
            declare: false,
            decls: vec![VarDeclarator {
                span: Span::default(),
                name: Pat::Ident(BindingIdent {
                    id: styles_ident,
                    type_ann: None,
                }),
                init: Some(Box::new(Expr::Call(CallExpr {
                    span: Span::default(),
                    callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                        span: Span::default(),
                        obj: Box::new(Expr::Ident(Ident::new(
                            "Object".into(),
                            Span::default(),
                        ))),
                        prop: MemberProp::Ident(Ident::new(
                            "assign".into(),
                            Span::default(),
                        )),
                    }))),
                    args: vec![
                        ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Object(ObjectLit {
                                span: Span::default(),
                                props: vec![],
                            })),
                        },
                        ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Ident(styles[0].clone())),
                        },
                    ],
                    type_args: None,
                }))),
                definite: false,
            }],
        })))
    }

    fn create_matcher_stmt(&mut self) -> Stmt {
        let matcher_ident = Ident::new(
            "_matcher".into(),
            Span::default(),
        );
        self.matcher_ident = Some(matcher_ident.clone());

        Stmt::Decl(Decl::Var(Box::new(VarDecl {
            span: Span::default(),
            kind: VarDeclKind::Const,
            declare: false,
            decls: vec![VarDeclarator {
                span: Span::default(),
                name: Pat::Ident(BindingIdent {
                    id: matcher_ident,
                    type_ann: None,
                }),
                init: Some(Box::new(Expr::Call(CallExpr {
                    span: Span::default(),
                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                        "getMatcher".into(),
                        Span::default(),
                    )))),
                    args: vec![
                        ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Ident(self.styles_ident.clone().unwrap())),
                        },
                        ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Str(Str {
                                span: Span::default(),
                                value: self.config.prefer.clone().into(),
                                raw: Some("'local'".into()),
                            }))),
                        },
                    ],
                    type_args: None,
                }))),
                definite: false,
            }],
        })))
    }
}

impl VisitMut for JsxCssModulesVisitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // 收集样式导入
        let mut style_imports = Vec::new();
        let mut style_import_indices = Vec::new();
        for (i, item) in module.body.iter().enumerate() {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
                if self.is_style_import(import) {
                    style_imports.push(import.clone());
                    style_import_indices.push(i);
                }
            }
        }

        if !style_imports.is_empty() {
            // 确保每个样式导入都有默认导入
            let mut default_styles = Vec::new();
            for import in &mut style_imports {
                let has_default = import.specifiers.iter()
                    .any(|s| matches!(s, ImportSpecifier::Default(_)));
                
                if !has_default {
                    let default_style = Ident::new(
                        "styles".into(),
                        Span::default(),
                    );
                    import.specifiers.push(ImportSpecifier::Default(ImportDefaultSpecifier {
                        span: Span::default(),
                        local: default_style.clone(),
                    }));
                    default_styles.push(default_style);
                } else if let Some(ImportSpecifier::Default(spec)) = import.specifiers.first() {
                    default_styles.push(spec.local.clone());
                }
            }

            // 更新原始导入
            for (i, import) in style_imports.iter().enumerate() {
                if let Some(&idx) = style_import_indices.get(i) {
                    module.body[idx] = ModuleItem::ModuleDecl(ModuleDecl::Import(import.clone()));
                }
            }

            // 添加 helper 导入
            let last_import_idx = *style_import_indices.last().unwrap();
            module.body.insert(
                last_import_idx + 1,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    span: Span::default(),
                    specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                        span: Span::default(),
                        local: Ident::new("getMatcher".into(), Span::default()),
                        imported: None,
                        is_type_only: false,
                    })],
                    src: Box::new(Str {
                        span: Span::default(),
                        value: "swc-plugin-jsx-css-modules/helpers".into(),
                        raw: Some("'swc-plugin-jsx-css-modules/helpers'".into()),
                    }),
                    type_only: false,
                    with: None,
                    phase: ImportPhase::default(),
                })),
            );

            // 添加合并样式和 matcher 语句
            module.body.insert(
                last_import_idx + 2,
                ModuleItem::Stmt(self.create_merged_styles_stmt(default_styles)),
            );
            module.body.insert(
                last_import_idx + 3,
                ModuleItem::Stmt(self.create_matcher_stmt()),
            );
        }

        module.visit_mut_children_with(self);
    }

    fn visit_mut_jsx_element(&mut self, jsx: &mut JSXElement) {
        if let Some(matcher_ident) = &self.matcher_ident {
            for attr in &mut jsx.opening.attrs {
                if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        if ident.sym == *"className" {
                            if let Some(JSXAttrValue::Lit(Lit::Str(str_lit))) = &attr.value {
                                attr.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                    span: Span::default(),
                                    expr: JSXExpr::Expr(Box::new(Expr::Call(CallExpr {
                                        span: Span::default(),
                                        callee: Callee::Expr(Box::new(Expr::Ident(matcher_ident.clone()))),
                                        args: vec![ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(Expr::Lit(Lit::Str(str_lit.clone()))),
                                        }],
                                        type_args: None,
                                    }))),
                                }));
                            }
                        }
                    }
                }
            }
        }

        jsx.visit_mut_children_with(self);
    }
}
