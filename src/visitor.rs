use swc_core::ecma::{
    ast::*,
    visit::{Fold, FoldWith},
};
use swc_core::common::{Span, SyntaxContext};
use regex::Regex;
use super::Config;

pub struct JsxCssModulesVisitor {
    config: Config,
    styles_ident: Option<Ident>,
    matcher_ident: Option<Ident>,
}

impl JsxCssModulesVisitor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
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

    fn create_matcher_stmt(&mut self) -> Stmt {
        let matcher_ident = Ident::new(
            "_matcher".into(),
            Span::default(),
            SyntaxContext::default(),
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
                        SyntaxContext::default(),
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
                                raw: Some(format!("'{}'", self.config.prefer).into()),
                            }))),
                        },
                    ],
                    type_args: None,
                    ctxt: SyntaxContext::default(),
                }))),
                definite: false,
            }],
            ctxt: SyntaxContext::default(),
        })))
    }
}

impl Fold for JsxCssModulesVisitor {
    fn fold_module(&mut self, mut module: Module) -> Module {
        // 收集样式导入
        let mut style_imports = Vec::new();
        let mut style_import_indices = Vec::new();
        let mut all_imports = Vec::new();
        for (i, item) in module.body.iter().enumerate() {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
                all_imports.push((i, import.clone()));
                if self.is_style_import(import) {
                    style_imports.push(import.clone());
                    style_import_indices.push(i);
                }
            }
        }

        if !style_imports.is_empty() {
            // 确保每个样式导入都有默认导入
            let mut default_styles = Vec::new();
            let mut style_imports_map = std::collections::HashMap::new();
            for (i, import) in style_imports.iter_mut().enumerate() {
                let default_style = Ident::new(
                    format!("style_{}", i).into(),
                    Span::default(),
                    SyntaxContext::default(),
                );
                import.specifiers = vec![if self.config.import_style == "namespace" {
                    ImportSpecifier::Namespace(ImportStarAsSpecifier {
                        span: Span::default(),
                        local: default_style.clone(),
                    })
                } else {
                    ImportSpecifier::Default(ImportDefaultSpecifier {
                        span: Span::default(),
                        local: default_style.clone(),
                    })
                }];
                default_styles.push(default_style);
                style_imports_map.insert(import.src.value.to_string(), import.clone());
            }

            // 添加 getMatcher 导入
            let get_matcher_import = ImportDecl {
                span: Span::default(),
                src: Box::new(Str {
                    span: Span::default(),
                    value: "swc-plugin-jsx-css-modules/helpers".into(),
                    raw: Some("'swc-plugin-jsx-css-modules/helpers'".into()),
                }),
                type_only: false,
                with: None,
                phase: Default::default(),
                specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                    span: Span::default(),
                    local: Ident::new(
                        "getMatcher".into(),
                        Span::default(),
                        SyntaxContext::default(),
                    ),
                    imported: None,
                    is_type_only: false,
                })],
            };

            // 创建 _styles 对象
            let styles_ident = Ident::new(
                "_styles".into(),
                Span::default(),
                SyntaxContext::default(),
            );
            self.styles_ident = Some(styles_ident.clone());
            let styles_assign = VarDecl {
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
                                SyntaxContext::default(),
                            ))),
                            prop: MemberProp::Ident(IdentName::new(
                                "assign".into(),
                                Span::default(),
                            )),
                        }))),
                        args: std::iter::once(Expr::Object(ObjectLit {
                            span: Span::default(),
                            props: vec![],
                        }))
                        .chain(default_styles.iter().map(|style| Expr::Ident(style.clone())))
                        .map(|expr| ExprOrSpread {
                            spread: None,
                            expr: Box::new(expr),
                        })
                        .collect(),
                        type_args: None,
                        ctxt: SyntaxContext::default(),
                    }))),
                    definite: false,
                }],
                ctxt: SyntaxContext::default(),
            };

            // 重新组织导入语句
            let mut new_body = Vec::new();
            let mut current_index = 0;

            // 1. 按原始顺序添加导入
            for (_, import) in all_imports {
                if let Some(updated_import) = style_imports_map.get(&import.src.value.to_string()) {
                    new_body.push(ModuleItem::ModuleDecl(ModuleDecl::Import(updated_import.clone())));
                } else {
                    new_body.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)));
                }
                current_index += 1;
            }

            // 2. 添加 getMatcher 导入
            new_body.push(ModuleItem::ModuleDecl(ModuleDecl::Import(get_matcher_import)));

            // 3. 添加 styles 和 matcher 语句
            new_body.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(styles_assign)))));
            new_body.push(ModuleItem::Stmt(self.create_matcher_stmt()));

            // 4. 添加剩余的内容
            for item in module.body.iter().skip(current_index) {
                if !matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(_))) {
                    new_body.push(item.clone());
                }
            }

            module.body = new_body;
        }

        // 递归处理子节点
        module.fold_children_with(self)
    }

    fn fold_jsx_element(&mut self, mut jsx: JSXElement) -> JSXElement {
        if let Some(matcher_ident) = &self.matcher_ident {
            for attr in &mut jsx.opening.attrs {
                if let JSXAttrOrSpread::JSXAttr(attr) = attr {
                    if let JSXAttrName::Ident(ident) = &attr.name {
                        if ident.sym == *"className" {
                            match &attr.value {
                                // 处理字符串字面量
                                Some(JSXAttrValue::Lit(Lit::Str(str_lit))) => {
                                    attr.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                        span: Span::default(),
                                        expr: JSXExpr::Expr(Box::new(Expr::Call(CallExpr {
                                            span: Span::default(),
                                            callee: Callee::Expr(Box::new(Expr::Ident(matcher_ident.clone()))),
                                            args: vec![ExprOrSpread {
                                                spread: None,
                                                expr: Box::new(Expr::Lit(Lit::Str(Str {
                                                    span: Span::default(),
                                                    value: str_lit.value.clone().into(),
                                                    raw: None
                                                })))
                                            }],
                                            type_args: None,
                                            ctxt: SyntaxContext::default()
                                        })))
                                    }));
                                },
                                // 处理 JSX 表达式容器（模板字符串和函数调用）
                                Some(JSXAttrValue::JSXExprContainer(container)) => {
                                    if let JSXExpr::Expr(expr) = &container.expr {
                                        attr.value = Some(JSXAttrValue::JSXExprContainer(JSXExprContainer {
                                            span: Span::default(),
                                            expr: JSXExpr::Expr(Box::new(Expr::Call(CallExpr {
                                                span: Span::default(),
                                                callee: Callee::Expr(Box::new(Expr::Ident(matcher_ident.clone()))),
                                                args: vec![ExprOrSpread {
                                                    spread: None,
                                                    expr: expr.clone()
                                                }],
                                                type_args: None,
                                                ctxt: SyntaxContext::default()
                                            })))
                                        }));
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        jsx.children = jsx.children.fold_with(self);
        jsx
    }
}
