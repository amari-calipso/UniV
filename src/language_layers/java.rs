use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

use tree_sitter::Node;

use crate::{ast_error, language_layer, token_error, token_warning, unil::{ast::{Expression, LiteralKind, NamedExpr, ObjectField}, tokens::{Token, TokenType}}, univm::object::{AnonObject, UniLValue}, utils::{lang::{get_token_from_variable, get_vec_of_expr_from_block, make_null, BaseASTTransformer}, substring}};

struct Environment {
    pub names: HashSet<Rc<str>>,
    enclosing: Option<Rc<RefCell<Environment>>>
} 

impl Environment {
    pub fn new() -> Self {
        Environment { names: HashSet::new(), enclosing: None }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { names: HashSet::new(), enclosing: Some(enclosing) }
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn define(&mut self, name: &Rc<str>) {
        self.names.insert(Rc::clone(name));
    }

    pub fn del(&mut self, name: &str) -> Result<(), ()> {
        if self.names.contains(name) {
            self.names.remove(name);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().del(name)
        } else {
            Err(())
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        if self.names.contains(name) {
            true
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().contains(name)
        } else {
            false
        }
    }
}

pub struct ASTTransformer {
    base: BaseASTTransformer,

    environment: Rc<RefCell<Environment>>,
    curr_class_fields: Option<HashSet<Rc<str>>>,
    
    last_label: Option<Rc<str>>,

    in_sort_decl: Option<HashMap<Rc<str>, Expression>>,
    ignore_super: bool,
    arrayv: bool,
}

impl ASTTransformer {
    pub fn new(source: String, filename: Rc<str>) -> Self {
        ASTTransformer { 
            base: BaseASTTransformer::new(source, filename),
            environment: Rc::new(RefCell::new(Environment::new())),
    
            curr_class_fields: None,
            last_label: None,
            
            in_sort_decl: None,
            ignore_super: false,
            arrayv: false,
        }
    }

    fn text_from_node(&mut self, node: &Node) -> Rc<str> {
        if let Ok(text) = node.utf8_text(self.base.source.as_bytes()) {
            Rc::from(text)
        } else {
            let tok = self.tok_from_node(node);
            token_error!(self.base, tok, "UTF-8 error");
            Rc::from("")
        }
    }

    fn tok_from_node(&mut self, node: &Node) -> Token {
        let text = self.text_from_node(node);
        self.tok_from_node_with_type_and_lexeme(node, TokenType::Null, &text)
    }

    fn tok_from_node_with_lexeme(&mut self, node: &Node, lexeme: &str) -> Token {
        self.tok_from_node_with_type_and_lexeme(node, TokenType::Null, lexeme)
    }

    fn tok_from_node_with_type(&mut self, node: &Node, type_: TokenType) -> Token {
        let text = self.text_from_node(node);
        self.tok_from_node_with_type_and_lexeme(node, type_, &text)
    }

    fn tok_from_node_with_type_and_lexeme(&self, node: &Node, type_: TokenType, lexeme: &str) -> Token {
        Token {
            source: Rc::from(self.base.source.as_ref()),
            filename: Rc::clone(&self.base.filename),
            type_,
            lexeme: Rc::from(lexeme),
            pos: node.start_position().column,
            end: node.end_position().column,
            line: node.start_position().row
        }
    }

    fn handle_labeled_loop_interrupt(&mut self, loop_: Expression, interrupt_type: &str, label_name: &str, tok: &Token) -> Expression {
        let catch_tmp = self.base.tmp_var();
        let mut catch_tmp_tok = tok.clone();
        catch_tmp_tok.set_lexeme(&catch_tmp);
        let catch_tmp_var = Expression::Variable { name: catch_tmp_tok.clone() };

        let mut has_attribute_tok = tok.clone();
        has_attribute_tok.set_lexeme("hasAttribute");

        Expression::Try {
            kw: tok.clone(),
            try_branch: Box::new(loop_),
            catch_branch: Some(Box::new(Expression::If { 
                kw: tok.clone(), 
                condition: Box::new(Expression::Call { 
                    callee: Box::new(Expression::Variable { name: has_attribute_tok }), 
                    paren: tok.clone(), 
                    args: vec![
                        catch_tmp_var.clone(),
                        Expression::Literal { 
                            value: format!("__Java_{interrupt_type}_{label_name}").into(), 
                            tok: tok.clone(), 
                            kind: LiteralKind::String 
                        }
                    ] 
                }), 
                then_branch: Box::new(make_null()), 
                else_branch: Some(Box::new(Expression::Throw { 
                    kw: tok.clone(), 
                    value: Some(Box::new(catch_tmp_var))
                }))
            })),
            catch_var: Some(catch_tmp_tok),
        } 
    }

    fn get_parameters(&mut self, node: &Node) -> Vec<NamedExpr> {
        match node.kind() {
            "identifier" | "_reserved_identifier" | "type_identifier" => {
                let mut name = self.tok_from_node(node);
                name.set_lexeme(&self.base.get_var_name(&name.lexeme));
                vec![NamedExpr { name, expr: None }]
            }
            "inferred_parameters" => {
                let mut output = Vec::new();

                let mut i = 1;
                loop {
                    let param_node = node.child(i).unwrap();
                    let text = self.text_from_node(&param_node);
                    if text.as_ref() == ")" {
                        break;
                    }

                    let mut name = self.tok_from_node(node);
                    name.set_lexeme(&self.base.get_var_name(&name.lexeme));
                    output.push(NamedExpr { name, expr: None });

                    i += 1;
                }

                output
            }
            "formal_parameters" => {
                let mut output = Vec::new();

                let mut i = 1;
                loop {
                    let param_node = node.child(i).unwrap();
                    let param_tok = self.tok_from_node(&param_node);
                    let text = self.text_from_node(&param_node);
                    if text.as_ref() == ")" {
                        break;
                    }

                    match param_node.kind() {
                        "receiver_parameter" => (),
                        "formal_parameter" => {
                            let mut name = self.tok_from_node(
                                &param_node.child(param_node.child_count() - 1).unwrap()
                            );

                            name.set_lexeme(&self.base.get_var_name(&name.lexeme));
                            output.push(NamedExpr { name, expr: None });
                        }
                        "spread_parameter" => {
                            token_error!(self.base, param_tok, "Variable parameter length is not supported");
                            continue;
                        }
                        _ => unreachable!()
                    }

                    i += 1;
                }

                output
            }
            _ => panic!("Invalid parameter node type")
        }
    }

    async fn get_class_fields(&mut self, node: &Node<'_>, ctx: &mut reblessive::Stk) {
        match node.kind() {
            "class_body" | "block" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if !matches!(self.text_from_node(&child).as_ref(), "{" | "}") {
                        ctx.run(|ctx| self.get_class_fields(&child, ctx)).await;
                    }
                }
            }
            "field_declaration" => {
                let name_node = node.child(node.child_count() - 2).unwrap();
                let orig_text = self.text_from_node(&name_node);
                let name = self.base.get_var_name(&orig_text);
                self.curr_class_fields.as_mut()
                    .expect("get_class_fields can only be called if curr_class_fields is initialized")
                    .insert(name);
            }
            "method_declaration" => {
                let name_node = node.child_by_field_name("name").unwrap();
                let orig_text = self.text_from_node(&name_node);
                let name = self.base.get_fn_name(&orig_text);

                self.curr_class_fields.as_mut()
                    .expect("get_class_fields can only be called if curr_class_fields is initialized")
                    .insert(name);
            }
            _ => ()
        }
    }

    fn get_class_definitions(&mut self, body: &Vec<Expression>, class_name: &Token, fields: &mut Vec<ObjectField>, init: &mut Option<Expression>) {
        for element in body {
            match &element {
                Expression::Assign { target, op, value, .. } => {
                    if !matches!(op.type_, TokenType::Walrus) {
                        token_error!(self.base, op, "Assignments are not allowed in class body");
                    }
    
                    if let Expression::Variable { name } = &**target {
                        fields.push(ObjectField::new(name.clone(), *value.clone(), None));
                    } else {
                        ast_error!(self.base, target, "Only variables are supported as assignment targets in class body");
                    }
                }
                Expression::Function { name, .. } => {
                    let actual_name = name.lexeme.strip_prefix(format!("{}__", class_name.lexeme).as_str()).unwrap_or("_");
                    if name.lexeme.as_ref() == "__Java_constructor" {
                        if init.is_none() {
                            *init = Some(element.clone());
                        } else {
                            token_error!(self.base, name, "Multiple constructors are not supported");
                        }
                    }
    
                    let mut tok = name.clone();
                    tok.set_lexeme(actual_name);
    
                    let mut any = name.clone();
                    any.set_lexeme("any");
    
                    fields.push(ObjectField::new(
                        tok, 
                        Expression::Variable { name: name.clone() }, 
                        Some(Expression::Variable { name: any })
                    ));
                }
                Expression::Block { expressions, .. } => {
                    self.get_class_definitions(expressions, class_name, fields, init);
                }
                _ => ast_error!(self.base, element, "Unsupported definition in class body")
            }
        }
    }

    async fn get_arguments(&mut self, node: &Node<'_>, ctx: &mut reblessive::Stk) -> Vec<Expression> {
        assert_eq!("argument_list", node.kind());

        let mut output = Vec::new();

        let mut i = 1;
        loop {
            let arg_node = node.child(i).unwrap();
            let text = self.text_from_node(&arg_node);
            if text.as_ref() == ")" {
                break;
            }

            let arg = ctx.run(|ctx| self.transform_one(&arg_node, ctx)).await;
            output.push(arg);

            i += 1;
        }

        output
    }

    async fn transform_one(&mut self, node: &Node<'_>, ctx: &mut reblessive::Stk) -> Expression {
        let node_token = self.tok_from_node(node);
        
        if node.is_error() {
            token_error!(self.base, node_token, "Syntax error");
            return make_null();
        }

        // TODO
        println!("{}", node.kind());
        println!("{}", node.to_sexp());
        println!("{}", self.text_from_node(node));
        println!("----------------\n\n");
        
        match node.kind() {
            "null_literal" | "interface_declaration" => make_null(),
            "true" => Expression::Literal { value: Rc::from("1"), tok: node_token, kind: LiteralKind::Int },
            "false" => Expression::Literal { value: Rc::from("0"), tok: node_token, kind: LiteralKind::Int },
            "this" => Expression::Variable { name: node_token },
            "expression_statement" | "primary_expression" | "class_literal" => {
                let inner = node.child(0).unwrap();
                ctx.run(|ctx| self.transform_one(&inner, ctx)).await
            }
            "parenthesized_expression" | "superclass" => {
                let inner = node.child(1).unwrap();
                ctx.run(|ctx| self.transform_one(&inner, ctx)).await
            }
            "super" => {
                if !self.ignore_super {
                    token_error!(self.base, node_token, "Inheritance is not supported");
                }
                
                make_null()
            }
            "syncronized_statement" => {
                let body_node = node.child_by_field_name("body").unwrap();
                ctx.run(|ctx| self.transform_one(&body_node, ctx)).await
            }
            "decimal_integer_literal" => {
                let value = self.text_from_node(&node);
                Expression::Literal { value, tok: node_token, kind: LiteralKind::Int }
            }
            "hex_integer_literal" => {
                todo!()
            }
            "octal_integer_literal" => {
                todo!()
            }
            "binary_integer_literal" => {
                todo!()
            }
            "decimal_floating_point_literal" => {
                todo!()
            }
            "hex_floating_point_literal" => {
                todo!()
            }
            "character_literal" | "string_literal" => {
                let value = self.text_from_node(node);
                let value = substring(&value, 1, value.len() - 1).into();
                Expression::Literal { value, tok: node_token, kind: LiteralKind::String }
            }
            "void_type" => {
                let mut name = node_token;
                name.set_lexeme("Null");
                Expression::Variable { name }
            }
            "floating_point_type" => {
                let mut name = node_token;
                name.set_lexeme("Float");
                Expression::Variable { name }
            }
            "integral_type" | "boolean_type" => {
                let mut name = node_token;

                if self.text_from_node(node).as_ref() == "char" {
                    name.set_lexeme("String");
                } else {
                    name.set_lexeme("Int");
                }

                Expression::Variable { name }
            }
            "identifier" | "_reserved_identifier" | "type_identifier" => {
                let mut name = node_token;
                name.set_lexeme(&self.base.get_var_name(&name.lexeme));

                // if identifier is not in environment (meaning it's not a local) but is in class fields, get the property from `this` 
                if !self.environment.borrow().contains(&name.lexeme) {
                    if let Some(class_fields) = &mut self.curr_class_fields {
                        if class_fields.contains(&name.lexeme) {
                            return Expression::Get { 
                                object: Box::new(Expression::Variable { 
                                    name: self.tok_from_node_with_lexeme(node, "this") 
                                }), 
                                name
                            };
                        }
                    }  
                } 

                Expression::Variable { name }
            }
            "formal_parameter" => {
                let name_node = node.child(node.child_count() - 1).unwrap();
                let mut name = self.tok_from_node(&name_node);
                name.set_lexeme(&self.base.get_var_name(&name.lexeme));
                Expression::Variable { name }
            }
            "throw_statement" => {
                let value_node = node.child(1).unwrap();
                let value = ctx.run(|ctx| self.transform_one(&value_node, ctx)).await;

                Expression::Throw { 
                    kw: node_token, 
                    value: Some(Box::new(value))
                }
            }
            "array_access" => {
                let subscripted_node = node.child_by_field_name("array").unwrap();
                let index_node = node.child_by_field_name("index").unwrap();

                let subscripted = ctx.run(|ctx| self.transform_one(&subscripted_node, ctx)).await;
                let index =  ctx.run(|ctx| self.transform_one(&index_node, ctx)).await;

                Expression::Subscript { 
                    subscripted: Box::new(subscripted), 
                    paren: node_token, 
                    index: Box::new(index)
                }
            }
            "package_declaration" => {
                let package = self.text_from_node(&node.child(node.child_count() - 2).unwrap());
                if package.contains("io.github.arrayv") {
                    self.arrayv = true;
                }

                make_null()
            }
            "import_declaration" => {
                let mut import_idx = 1;
                if self.text_from_node(&node.child(import_idx).unwrap()).as_ref() == "static" {
                    import_idx += 1;
                }

                let import = self.text_from_node(&node.child(import_idx).unwrap());
                if import.as_ref() == "io.github.arrayv.main.ArrayVisualizer" ||
                   import.as_ref() == "main.ArrayVisualizer" 
                {
                    self.arrayv = true;
                }

                make_null()
            }
            "class_body" => {
                let mut expressions = Vec::new();

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if matches!(self.text_from_node(&child).as_ref(), "{" | "}") {
                        continue;
                    }

                    expressions.push(ctx.run(|ctx| self.transform_one(&child, ctx)).await);
                }

                Expression::Block { 
                    opening_brace: node_token, 
                    expressions
                }
            }
            "block" => {
                let mut expressions = Vec::new();

                let previous = Rc::clone(&self.environment);
                self.environment = Rc::new(RefCell::new(Environment::with_enclosing(Rc::clone(&previous))));

                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if matches!(self.text_from_node(&child).as_ref(), "{" | "}") {
                        continue;
                    }

                    expressions.push(ctx.run(|ctx| self.transform_one(&child, ctx)).await);
                }

                self.environment = previous;

                Expression::ScopedBlock { 
                    dollar: node_token, 
                    expressions
                }
            }
            "field_access" => {
                let object_node = node.child_by_field_name("object").unwrap();
                let name_node = node.child_by_field_name("field").unwrap();

                let second = node.child(1).unwrap();
                if self.text_from_node(&second).as_ref() != "." {
                    let tok = self.tok_from_node(&second);
                    token_error!(self.base, tok, "Unsupported syntax");
                    return make_null();
                }

                let object = ctx.run(|ctx| self.transform_one(&object_node, ctx)).await;
                let name = get_token_from_variable(
                    ctx.run(|ctx| self.transform_one(&name_node, ctx)).await
                );

                Expression::Get { 
                    object: Box::new(object), 
                    name
                }
            }
            "class_declaration" => {
                let first = node.child(0).unwrap();
                if first.kind() == "modifiers" {
                    let mut cursor = first.walk();
                    for modifier in first.children(&mut cursor) {    
                        if self.text_from_node(&modifier).as_ref() == "abstract" {
                            let tok = self.tok_from_node(&modifier);
                            token_warning!(tok, "Abstract classes are not supported. Ignoring");
                            return make_null();
                        }
                    }
                }

                let class_name = self.tok_from_node(&node.child_by_field_name("name").unwrap());
                let body_node = node.child_by_field_name("body").unwrap();

                if let Some(superclass) = node.child_by_field_name("superclass") {
                    let superclass_expr = ctx.run(|ctx| self.transform_one(&superclass, ctx)).await;

                    if let Expression::Variable { name: superclass_name } = superclass_expr {
                        if superclass_name.lexeme.as_ref() == "Thread" {
                            todo!();
                        } else if self.arrayv && superclass_name.lexeme.as_ref() == "Sort" {
                            self.in_sort_decl = Some(HashMap::new());
                        }
                    } 

                    let tok = self.tok_from_node(&superclass);
                    token_warning!(tok, "Inheritance is not supported. Ignoring");
                }

                let old_class_fields = self.curr_class_fields.replace(HashSet::new());
                ctx.run(|ctx| self.get_class_fields(&body_node, ctx)).await;

                let old_curr_name = self.base.curr_name.clone();
                self.base.curr_name.push_str(&class_name.lexeme);

                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;

                self.base.curr_name = old_curr_name;
                self.curr_class_fields = old_class_fields;

                let mut fields = Vec::new();
                let mut constructor = None;

                let mut definitions = get_vec_of_expr_from_block(body);
                self.get_class_definitions(&definitions, &class_name, &mut fields, &mut constructor);    
        
                let tok = self.tok_from_node_with_type(node, TokenType::Walrus);
                let object = Expression::AnonObject { kw: tok.clone(), fields };

                let info = self.in_sort_decl.take();

                if let Some(constructor_fn) = constructor {
                    if let Some(info) = info {
                        todo!("sort postprocessing"); 
                    }

                    if let Expression::Function { params, body, .. } = constructor_fn {
                        let this = Expression::Variable { 
                            name: self.tok_from_node_with_lexeme(node, "this") 
                        };

                        let mut fn_body: Vec<Expression> = [
                            Expression::Assign { 
                                target: Box::new(this.clone()), 
                                op: tok, 
                                value: Box::new(object), 
                                type_spec: None 
                            }
                        ].into_iter().chain(body.into_iter()).collect();
                        fn_body.push(this);

                        definitions.push(Expression::Function { 
                            name: class_name, 
                            params, 
                            return_type: Box::new(Expression::Variable { 
                                name: self.tok_from_node_with_lexeme(node, "any") 
                            }), 
                            body: fn_body
                        });
                    } else {
                        unreachable!()
                    }
                } else {
                    definitions.push(Expression::Function { 
                        name: class_name, 
                        params: Vec::new(), 
                        return_type: Box::new(Expression::Variable { 
                            name: self.tok_from_node_with_lexeme(node, "any") 
                        }), 
                        body: vec![object] 
                    });
                }

                // removes static variables declarations, which would get turned into globals without namespaces
                definitions.retain(|x| !matches!(x, Expression::Assign { .. }));

                Expression::Block { 
                    opening_brace: node_token, 
                    expressions: definitions 
                }
            }
            "method_declaration" => {
                if let Some(body_node) = node.child_by_field_name("body") {    
                    let header_node = node.child(1).unwrap();

                    // TODO: verify annotations, currently they're just ignored silently

                    let mut method_declarator_node = header_node.child(header_node.child_count() - 1).unwrap();
                    if method_declarator_node.kind() == "throws" {
                        method_declarator_node = header_node.child(header_node.child_count() - 2).unwrap();
                    }

                    let body = get_vec_of_expr_from_block(
                        ctx.run(|ctx| self.transform_one(&body_node, ctx)).await
                    );

                    let name_node = method_declarator_node.child_by_field_name("name").unwrap();
                    let name = get_token_from_variable(
                        ctx.run(|ctx| self.transform_one(&name_node, ctx)).await
                    );

                    let mut params = self.get_parameters(
                        &method_declarator_node.child_by_field_name("parameters").unwrap()
                    );

                    params.insert(0, NamedExpr {
                        name: self.tok_from_node_with_lexeme(node, "this"), 
                        expr: None 
                    });

                    Expression::Function { 
                        name, params, body, 
                        return_type: Box::new(Expression::Variable { 
                            name: self.tok_from_node_with_lexeme(node, "any") 
                        })
                    }
                } else {
                    make_null() // TODO: check if this is what it actually does
                }
            }
            "assignment_expression" => {
                let target_node = node.child_by_field_name("left").unwrap();
                let value_node = node.child_by_field_name("right").unwrap();

                let target = ctx.run(|ctx| self.transform_one(&target_node, ctx)).await;
                let value = ctx.run(|ctx| self.transform_one(&value_node, ctx)).await;
            
                let operator_node = node.child_by_field_name("operator").unwrap();
                let mut op = self.tok_from_node(&operator_node);
                op.set_type({
                    match self.text_from_node(&operator_node).as_ref() {
                        "="   => TokenType::Equal,
                        "+="  => TokenType::PlusEquals,
                        "-="  => TokenType::MinusEquals,
                        "*="  => TokenType::StarEquals,
                        "/="  => TokenType::SlashEquals,
                        "&="  => TokenType::AndEquals,
                        "|="  => TokenType::OrEquals,
                        "^="  => TokenType::XorEquals,
                        "%="  => TokenType::ModEquals,
                        "<<=" => TokenType::ShiftLeftEquals,
                        ">>=" | ">>>=" => TokenType::ShiftRightEquals,
                        _ => {
                            token_error!(self.base, op, "Invalid assignment operator");
                            TokenType::Null
                        }
                    }
                });

                Expression::Assign { 
                    target: Box::new(target), 
                    op, 
                    value: Box::new(value), 
                    type_spec: None
                }
            }
            "binary_expression" => {
                let left_node = node.child_by_field_name("left").unwrap();
                let right_node = node.child_by_field_name("right").unwrap();

                let left = ctx.run(|ctx| self.transform_one(&left_node, ctx)).await;
                let right = ctx.run(|ctx| self.transform_one(&right_node, ctx)).await;

                let operator_node = node.child_by_field_name("operator").unwrap();
                let mut op = self.tok_from_node(&operator_node);
                op.set_type({
                    match self.text_from_node(&operator_node).as_ref() {
                        ">"  => TokenType::Greater,
                        "<"  => TokenType::Less,
                        ">=" => TokenType::GreaterEqual,
                        "<=" => TokenType::LessEqual,
                        "==" => TokenType::EqualEqual,
                        "!=" => TokenType::BangEqual,
                        "&&" => TokenType::LogicAnd,
                        "||" => TokenType::LogicOr,
                        "+"  => TokenType::Plus,
                        "-"  => TokenType::Minus,
                        "*"  => TokenType::Star,
                        "/"  => TokenType::Slash,
                        "&"  => TokenType::BitwiseAnd,
                        "|"  => TokenType::BitwiseOr,
                        "^"  => TokenType::BitwiseXor,
                        "%"  => TokenType::Mod,
                        "<<" => TokenType::ShiftLeft,
                        ">>" | ">>>" => TokenType::ShiftRight,
                        _ => {
                            token_error!(self.base, op, "Invalid binary operator");
                            TokenType::Null
                        }
                    }
                });

                match op.type_ {
                    TokenType::Greater | TokenType::Less | 
                    TokenType::GreaterEqual | TokenType::LessEqual => {
                        Expression::Cmp { 
                            left: Box::new(left), 
                            op, 
                            right: Box::new(right) 
                        }
                    }
                    TokenType::LogicAnd | TokenType::LogicOr => {
                        Expression::Logic { 
                            left: Box::new(left), 
                            op, 
                            right: Box::new(right) 
                        }
                    }
                    _ => {
                        Expression::Binary { 
                            left: Box::new(left), 
                            op, 
                            right: Box::new(right) 
                        }
                    }
                }
            }
            "instanceof_expression" => {
                if let Some(right_node) = node.child_by_field_name("right") {
                    let right = ctx.run(|ctx| self.transform_one(&right_node, ctx)).await;

                    if let Expression::Variable { name } = right {
                        let left_node = node.child_by_field_name("left").unwrap();
                        let left = ctx.run(|ctx| self.transform_one(&left_node, ctx)).await;

                        return Expression::Cmp { 
                            left: Box::new(left), 
                            op: self.tok_from_node_with_type(node, TokenType::EqualEqual), 
                            right: Box::new(Expression::Literal { 
                                value: Rc::clone(&name.lexeme), 
                                tok: name, 
                                kind: LiteralKind::String 
                            }) 
                        }
                    }
                } 

                token_error!(self.base, node_token, "Unsupported syntax");
                make_null()
            }
            "ternary_expression" => {
                let condition_node = node.child_by_field_name("condition").unwrap();
                let then_branch_node = node.child_by_field_name("consequence").unwrap();
                let else_branch_node = node.child_by_field_name("alternative").unwrap();

                let condition = ctx.run(|ctx| self.transform_one(&condition_node, ctx)).await;
                let then_branch = ctx.run(|ctx| self.transform_one(&then_branch_node, ctx)).await;
                let else_branch = ctx.run(|ctx| self.transform_one(&else_branch_node, ctx)).await;

                Expression::Ternary { 
                    question_tok: node_token, 
                    condition: Box::new(condition), 
                    then_expr: Box::new(then_branch), 
                    else_expr: Box::new(else_branch), 
                }
            }
            "update_expression" => {
                let first = node.child(0).unwrap();
                let first_text = self.text_from_node(&first);

                if matches!(first_text.as_ref(), "++" | "--") {
                    let inner_node = node.child(1).unwrap();
                    let inner = ctx.run(|ctx| self.transform_one(&inner_node, ctx)).await;

                    let mut op = self.tok_from_node(&first);
                    op.set_type({
                        match first_text.as_ref() {
                            "++" => TokenType::PlusPlus,
                            "--" => TokenType::MinusMinus,
                            _ => {
                                token_error!(self.base, op, "Invalid update operator");
                                TokenType::Null
                            }
                        }
                    });

                    Expression::Unary { 
                        op, 
                        expr: Box::new(inner), 
                        is_prefix: true 
                    }
                } else {
                    let operator_node = node.child(1).unwrap();
                    let operator = self.text_from_node(&operator_node);
                    let inner = ctx.run(|ctx| self.transform_one(&first, ctx)).await;

                    let mut op = self.tok_from_node(&operator_node);
                    op.set_type({
                        match operator.as_ref() {
                            "++" => TokenType::PlusPlus,
                            "--" => TokenType::MinusMinus,
                            _ => {
                                token_error!(self.base, op, "Invalid update operator");
                                TokenType::Null
                            }
                        }
                    });

                    Expression::Unary { 
                        op, 
                        expr: Box::new(inner), 
                        is_prefix: false 
                    }
                }
            }
            "unary_expression" => {
                let operator_node = node.child(0).unwrap();
                let operator = self.text_from_node(&operator_node);
                let inner_node = node.child(1).unwrap();
                let inner = ctx.run(|ctx| self.transform_one(&inner_node, ctx)).await;

                let mut op = self.tok_from_node(&operator_node);
                op.set_type({
                    match operator.as_ref() {
                        "+" => TokenType::Plus,
                        "-" => TokenType::Minus,
                        "!" => TokenType::Bang,
                        "~" => TokenType::Tilde,
                        _ => {
                            token_error!(self.base, op, "Invalid unary operator");
                            TokenType::Null
                        }
                    }
                });

                Expression::Unary { 
                    op, 
                    expr: Box::new(inner), 
                    is_prefix: true 
                }
            }
            "if_statement" => {
                let condition_node = node.child_by_field_name("condition").unwrap();
                let then_branch_node = node.child_by_field_name("consequence").unwrap();

                let condition = ctx.run(|ctx| self.transform_one(&condition_node, ctx)).await;
                let then_branch = ctx.run(|ctx| self.transform_one(&then_branch_node, ctx)).await;

                let else_branch = {
                    if let Some(else_branch_node) = node.child_by_field_name("alternative") {
                        Some(Box::new(ctx.run(|ctx| self.transform_one(&else_branch_node, ctx)).await))
                    } else {
                        None
                    }
                };

                Expression::If { 
                    kw: node_token, 
                    condition: Box::new(condition), 
                    then_branch: Box::new(then_branch), 
                    else_branch, 
                }
            }
            "while_statement" => {
                let condition_node = node.child_by_field_name("condition").unwrap();
                let body_node = node.child_by_field_name("body").unwrap();

                let condition = ctx.run(|ctx| self.transform_one(&condition_node, ctx)).await;
                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;

                if let Some(label) = self.last_label.take() {
                    let body = self.handle_labeled_loop_interrupt(
                        body, "continue", 
                        &label, &node_token
                    );

                    self.handle_labeled_loop_interrupt(
                        Expression::While { 
                            kw: node_token.clone(), 
                            condition: Box::new(condition), 
                            body: Box::new(body),
                            increment: None
                        }, 
                        "break", 
                        &label, 
                        &node_token
                    )
                } else {
                    Expression::While { 
                        kw: node_token, 
                        condition: Box::new(condition), 
                        body: Box::new(body),
                        increment: None
                    }
                }
            }
            "do_statement" => {
                let body_node = node.child_by_field_name("body").unwrap();
                let condition_node = node.child_by_field_name("condition").unwrap();

                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;
                let condition = ctx.run(|ctx| self.transform_one(&condition_node, ctx)).await;

                if let Some(label) = self.last_label.take() {
                    let body = self.handle_labeled_loop_interrupt(
                        body, "continue", 
                        &label, &node_token
                    );

                    self.handle_labeled_loop_interrupt(
                        Expression::DoWhile { 
                            kw: node_token.clone(), 
                            condition: Box::new(condition), 
                            body: Box::new(body) 
                        }, 
                        "break", 
                        &label, 
                        &node_token
                    )
                } else {
                    Expression::DoWhile { 
                        kw: node_token, 
                        condition: Box::new(condition), 
                        body: Box::new(body) 
                    }
                }
            }
            "for_statement" => {
                let condition = Box::new({
                    if let Some(condition_node) = node.child_by_field_name("condition") {
                        ctx.run(|ctx| self.transform_one(&condition_node, ctx)).await
                    } else {
                        Expression::Literal { 
                            value: Rc::from("1"), 
                            tok: node_token.clone(), 
                            kind: LiteralKind::Int 
                        }
                    }
                });

                let increment = {
                    let mut increments = Vec::new();

                    let mut cursor = node.walk();
                    for increment_node in node.children_by_field_name("update", &mut cursor) {
                        increments.push(ctx.run(|ctx| self.transform_one(&increment_node, ctx)).await);
                    }

                    if increments.is_empty() {
                        None
                    } else if increments.len() == 1 {
                        Some(Box::new(increments.pop().unwrap()))
                    } else {
                        Some(Box::new(Expression::Block { 
                            opening_brace: node_token.clone(),
                            expressions: increments 
                        }))
                    }
                };

                let body_node = node.child_by_field_name("body").unwrap();
                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;

                let mut initializers = Vec::new();

                let mut cursor = node.walk();
                for initializer_node in node.children_by_field_name("init", &mut cursor) {
                    initializers.push(ctx.run(|ctx| self.transform_one(&initializer_node, ctx)).await);
                }

                let last_label = self.last_label.take();

                let mut for_loop = Expression::While { 
                    kw: node_token.clone(), 
                    condition, 
                    body: Box::new({
                        if let Some(label) = &last_label {
                            self.handle_labeled_loop_interrupt(
                                body, "continue", 
                                &label, &node_token
                            )
                        } else {
                            body
                        }
                    }), 
                    increment 
                };

                if !initializers.is_empty() {
                    initializers.push(for_loop);

                    for_loop = Expression::ScopedBlock { 
                        dollar: node_token.clone(), 
                        expressions: initializers
                    }
                }

                if let Some(label) = last_label {
                    self.handle_labeled_loop_interrupt(
                        for_loop, 
                        "break", 
                        &label, 
                        &node_token
                    )
                } else {
                    for_loop
                }
            }
            "enhanced_for_statement" => {
                let mut idx = 2; // skip 'for' and '('
                let first = node.child(idx).unwrap();
                if first.kind() == "modifiers" {
                    idx += 1;
                }

                idx += 1; // skip variable type
                
                let variable_node = node.child(idx).unwrap();
                let variable = get_token_from_variable(
                    ctx.run(|ctx| self.transform_one(&variable_node, ctx)).await
                );

                let iterator_node = node.child_by_field_name("value").unwrap();
                let iterator = ctx.run(|ctx| self.transform_one(&iterator_node, ctx)).await;

                let body_node = node.child_by_field_name("body").unwrap();
                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;

                if let Some(label) = self.last_label.take() {
                    let body = self.handle_labeled_loop_interrupt(
                        body, "continue", 
                        &label, &node_token
                    );

                    self.handle_labeled_loop_interrupt(
                        Expression::ScopedBlock { 
                            dollar: node_token.clone(), 
                            expressions: vec![
                                Expression::Foreach { 
                                    kw: node_token.clone(),
                                    variable, 
                                    iterator: Box::new(iterator), 
                                    body: Box::new(body) 
                                }
                            ]
                        }, 
                        "break", 
                        &label, 
                        &node_token
                    )
                } else {
                    Expression::ScopedBlock { 
                        dollar: node_token.clone(), 
                        expressions: vec![
                            Expression::Foreach { 
                                kw: node_token,
                                variable, 
                                iterator: Box::new(iterator), 
                                body: Box::new(body) 
                            }
                        ]
                    }
                }
            }
            "return_statement" => {
                let first = node.child(1).unwrap();

                Expression::Return { 
                    kw: node_token, 
                    value: {
                        if self.text_from_node(&first).as_ref() == ";" {
                            None
                        } else {
                            Some(Box::new(ctx.run(|ctx| self.transform_one(&first, ctx)).await))
                        }
                    }
                }                
            }
            "try_statement" => {
                let body_node = node.child_by_field_name("body").unwrap();
                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;

                let mut catch_branch = None;
                let mut catch_var = None;
                let mut finally_clause = None;

                let mut cursor = node.walk();
                for (i, child) in node.children(&mut cursor).enumerate() {
                    if i < 2 { // skip 'try' and body
                        continue;
                    }

                    if child.kind() == "catch_clause" {
                        if catch_branch.is_none() {
                            let catch_binding_node = child.child(2).unwrap(); // skip 'catch' and '('
                            let catch_body_node = child.child_by_field_name("body").unwrap();
                            
                            let mut idx = 0;
                            if catch_binding_node.child(idx).unwrap().kind() == "modifiers" {
                                idx += 1;
                            }

                            idx += 1; // skip catch variable type

                            let catch_variable_node = node.child(idx).unwrap();
                            let catch_variable = get_token_from_variable(
                                ctx.run(|ctx| self.transform_one(&catch_variable_node, ctx)).await
                            );

                            let catch_body = ctx.run(|ctx| self.transform_one(&catch_body_node, ctx)).await;
                            
                            catch_branch.replace(Box::new(catch_body));
                            catch_var.replace(catch_variable);
                        } else {
                            let tok = self.tok_from_node(&child);
                            token_error!(self.base, tok, "Only one exception handler is supported");
                        }
                    } else {
                        let finally_block_node = child.child(1).unwrap();
                        finally_clause.replace(
                            ctx.run(|ctx| self.transform_one(&finally_block_node, ctx)).await
                        );
                    }
                }

                let try_statement = Expression::Try { 
                    kw: node_token.clone(), 
                    try_branch: Box::new(body), 
                    catch_branch, 
                    catch_var 
                };

                if let Some(finally) = finally_clause {
                    Expression::Block { 
                        opening_brace: node_token,
                        expressions: vec![
                            try_statement,
                            finally
                        ] 
                    }
                } else {
                    try_statement
                }
            }
            "break_statement" => {
                let first = node.child(1).unwrap();
                let first_text = self.text_from_node(&first);
                if first_text.as_ref() == ";" {
                    Expression::Break { 
                        kw: node_token, 
                        value: None 
                    }
                } else {
                    Expression::Block { 
                        opening_brace: node_token.clone(), 
                        expressions: vec![
                            Expression::Throw { 
                                kw: node_token.clone(), 
                                value: Some(Box::new(Expression::AnonObject { 
                                    kw: node_token.clone(), 
                                    fields: vec![
                                        ObjectField {
                                            name: self.tok_from_node_with_lexeme(
                                                &first, 
                                                format!("__Java_break_{first_text}").as_str()
                                            ),
                                            expr: make_null(),
                                            type_: None
                                        }
                                    ]
                                }))
                            },
                            // allows UniL to detect this as a break, and give proper errors about it
                            Expression::Break { kw: node_token, value: None }
                        ]
                    }
                }
            }
            "continue_statement" => {
                let first = node.child(1).unwrap();
                let first_text = self.text_from_node(&first);
                if first_text.as_ref() == ";" {
                    Expression::Continue { 
                        kw: node_token, 
                        value: None 
                    }
                } else {
                    Expression::Block { 
                        opening_brace: node_token.clone(), 
                        expressions: vec![
                            Expression::Throw { 
                                kw: node_token.clone(), 
                                value: Some(Box::new(Expression::AnonObject { 
                                    kw: node_token.clone(), 
                                    fields: vec![
                                        ObjectField {
                                            name: self.tok_from_node_with_lexeme(
                                                &first, 
                                                format!("__Java_continue_{first_text}").as_str()
                                            ),
                                            expr: make_null(),
                                            type_: None
                                        }
                                    ]
                                }))
                            },
                            // allows UniL to detect this as a continue, and give proper errors about it
                            Expression::Continue { kw: node_token, value: None }
                        ]
                    }
                }
            }
            "labeled_statement" => {
                let name_node = node.child(0).unwrap();
                let name = get_token_from_variable(
                    ctx.run(|ctx| self.transform_one(&name_node, ctx)).await
                );

                let previous_label = self.last_label.take();
                self.last_label = Some(name.lexeme);

                let statement_node = node.child(2).unwrap();
                let statement = ctx.run(|ctx| self.transform_one(&statement_node, ctx)).await;

                self.last_label = previous_label;
                
                statement
            }
            "assert_statement" => {
                let expression_node = node.child(1).unwrap();
                let expression = ctx.run(|ctx| self.transform_one(&expression_node, ctx)).await;

                Expression::If {
                    kw: node_token.clone(),
                    condition: Box::new(Expression::Unary { 
                        op: self.tok_from_node_with_type(node, TokenType::Bang), 
                        expr: Box::new(expression), 
                        is_prefix: true 
                    }),
                    then_branch: Box::new(Expression::Throw { 
                        kw: node_token.clone(), 
                        value: Some(Box::new(Expression::Literal { 
                            value: Rc::from("Assertion failed"), 
                            tok: node_token, 
                            kind: LiteralKind::String
                        }))
                    }),
                    else_branch: None,
                }
            }
            "cast_expression" => {
                let type_node = node.child_by_field_name("type").unwrap();
                let type_ = ctx.run(|ctx| self.transform_one(&type_node, ctx)).await;

                if let Expression::Variable { name } = &type_ {
                    let value_node = node.child_by_field_name("value").unwrap();
                    let value = ctx.run(|ctx| self.transform_one(&value_node, ctx)).await;

                    if name.lexeme.as_ref() == "Int" || name.lexeme.as_ref() == "Float" {
                        return Expression::Call { 
                            callee: Box::new(Expression::Variable { 
                                name: self.tok_from_node_with_lexeme(
                                    &type_node, 
                                    &name.lexeme.to_lowercase()
                                ) 
                            }), 
                            paren: node_token, 
                            args: vec![value]
                        }
                    }
                }
                
                token_error!(self.base, node_token, "Unsupported cast");
                make_null()
            }
            "lambda_expression" => {
                let parameters_node = node.child_by_field_name("parameters").unwrap();
                let body_node = node.child_by_field_name("body").unwrap();

                let params = self.get_parameters(&parameters_node);
                let body = ctx.run(|ctx| self.transform_one(&body_node, ctx)).await;

                Expression::Function { 
                    name: self.tok_from_node_with_lexeme(node, "_"), 
                    params, 
                    return_type: Box::new(Expression::Variable { 
                        name: self.tok_from_node_with_lexeme(node, "any") 
                    }), 
                    body: vec![body]
                }
            }
            "constructor_declaration" => {
                let body_node = node.child_by_field_name("body").unwrap();

                let body = get_vec_of_expr_from_block(
                    ctx.run(|ctx| self.transform_one(&body_node, ctx)).await
                );

                let name_node = node.child_by_field_name("name").unwrap();
                let mut name = get_token_from_variable(
                    ctx.run(|ctx| self.transform_one(&name_node, ctx)).await
                );
                name.set_lexeme("__Java_constructor");

                let params = self.get_parameters(
                    &node.child_by_field_name("parameters").unwrap()
                );

                Expression::Function { 
                    name, params, body, 
                    return_type: Box::new(Expression::Variable { 
                        name: self.tok_from_node_with_lexeme(node, "any") 
                    })
                }
            }
            "constructor_body" => {
                self.ignore_super = self.in_sort_decl.is_some();

                let mut expressions = Vec::new();
                
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if matches!(self.text_from_node(&child).as_ref(), "{" | "}") || child.kind() == "explicit_constructor_invocation" {
                        continue;
                    }
                    
                    expressions.push(ctx.run(|ctx| self.transform_one(&child, ctx)).await);
                }

                self.ignore_super = false;

                Expression::Block { 
                    opening_brace: node_token, 
                    expressions 
                }
            }
            "switch_expression" => {
                todo!()
            }
            "object_creation_expression" => {
                todo!()
            }
            "method_invocation" => {
                if self.in_sort_decl.is_some() {
                    let name_node = node.child_by_field_name("name").unwrap();
                    let name_expr = ctx.run(|ctx| self.transform_one(&name_node, ctx)).await;
                    let name = get_token_from_variable(name_expr);

                    let args_node = node.child_by_field_name("arguments").unwrap();
                    let args: Vec<Expression> = ctx.run(|ctx| self.get_arguments(&args_node, ctx)).await;

                    if let Some(info) = &mut self.in_sort_decl {
                        // handles ArrayV declarations
                        match name.lexeme.as_ref() {
                            "setSortListName" | 
                            "setRunSortName" |
                            "setCategory" => {
                                if args.len() != 1 {
                                    todo!("error")
                                }

                                info.insert(
                                    Rc::from({
                                        match name.lexeme.as_ref() {
                                            "setSortListName" => "listName",
                                            "setRunSortName" => "name",
                                            "setCategory" => "category",
                                            _ => unreachable!()
                                        }
                                    }),
                                    args[0].clone()
                                );
                            }
                            // interactive:
                            "setUnreasonableLimit" => {
                                if args.len() != 1 {
                                    todo!("error")
                                }

                                if let Expression::Literal { value, kind, .. } = &args[0] {
                                    if matches!(kind, LiteralKind::Int) && value.as_ref() == "0" {
                                        return make_null();
                                    }
                                }

                                todo!()
                            }
                            "setQuestion" => {
                                todo!()
                            }
                            // ignored:
                            "setRunAllSortsName" | 
                            "setBucketSort" | 
                            "setRadixSort" |
                            "setUnreasonablySlow" |
                            "setBogoSort" => (),
                            _ => {
                                todo!("warning: unsupported method call");
                            }
                        }
                    } else {
                        unreachable!()
                    }
                    
                    make_null()
                } else {
                    todo!()
                }
            }
            "method_reference" => {
                todo!()
            }
            "array_creation_expression" => {
                todo!()
            }
            "dimensions_expr" => {
                todo!()
            }
            "argument_list" => {
                todo!()
            }
            "dimensions" => {
                todo!()
            }
            "switch_block" => {
                todo!()
            }
            "switch_block_statement_group" => {
                todo!()
            }
            "switch_rule" => {
                todo!()
            }
            "switch_label" => {
                todo!()
            }
            "pattern" => {
                todo!()
            }
            "type_pattern" => {
                todo!()
            }
            "record_pattern" => {
                todo!()
            }
            "record_pattern_body" => {
                todo!()
            }
            "record_pattern_component" => {
                todo!()
            }
            "underscore_pattern" => {
                todo!()
            }
            "guard" => {
                todo!()
            }
            "marker_annotation" => {
                todo!()
            }
            "annotation_argument_list" => {
                todo!()
            }
            "element_value_pair" => {
                todo!()
            }
            "element_value_array_initializer" => {
                todo!()
            }
            "declaration" => {
                todo!()
            }
            "type_parameters" => {
                todo!()
            }
            "type_parameter" => {
                todo!()
            }
            "type_bound" => {
                todo!()
            }
            "super_interfaces" => {
                todo!()
            }
            "type_list" => {
                todo!()
            }
            "explicit_constructor_invocation" => {
                todo!()
            }
            "record_declaration" => {
                todo!()
            }
            "extends_interfaces" => {
                todo!()
            }
            "constant_declaration" => {
                todo!()
            }
            "annotation_type_declaration" => {
                todo!()
            }
            "annotation_type_body" => {
                todo!()
            }
            "annotation_type_element_declaration" => {
                todo!()
            }
            "enum_declaration" => {
                todo!()
            }
            "enum_body" => {
                todo!()
            }
            "enum_body_declarations" => {
                todo!()
            }
            "enum_constant" => {
                todo!()
            }
            "scoped_identifier" => {
                todo!()
            }
            "field_declaration" => {
                todo!()
            }
            "variable_declarator" => {
                todo!()
            }
            "array_initializer" => {
                todo!()
            }
            "annotated_type" => {
                todo!()
            }
            "scoped_type_identifier" => {
                todo!()
            }
            "generic_type" => {
                todo!()
            }
            "array_type" => {
                todo!()
            }
            "local_variable_declaration" => {
                todo!()
            }
            "_variable_declarator_list" => {
                todo!()
            }
            "compact_constructor_declaration" => {
                todo!()
            }
            "yield_statement" | "module_declaration" | "template_expression" | 
            "try_with_resources_statement" | "static_initializer" => {
                token_error!(self.base, node_token, "Unsupported syntax");
                make_null()
            }
            _ => {
                if node.is_extra() {
                    make_null()
                } else {
                    unreachable!("Invalid or unhandled Java node type '{}'", node.kind());
                }
            }
        }
    }

    pub fn transform(&mut self, node: &Node) -> Vec<Expression> {
        assert_eq!(node.kind(), "program");

        let tok = self.tok_from_node(node);

        let mut expressions = Vec::new();

        if node.is_error() {
            token_error!(self.base, tok, "Syntax error");
            return expressions;
        }

        let mut stack = reblessive::Stack::new();

        let mut cursor = node.walk();
        for statement in node.children(&mut cursor) {
            expressions.push(
                stack.enter(
                    |ctx| ctx.run(|ctx| self.transform_one(&statement, ctx))
                ).finish()
            );
        }
        
        expressions
    }
}

language_layer! {
    extension = "java";

    java::process(source, filename) {
        use tree_sitter::Parser;
        use crate::language_layers::java;

        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_java::LANGUAGE.into())
            .expect("Unable to load Java grammar");

        let tree = parser.parse(&source, None).unwrap();
        let root = tree.root_node();

        let mut ast_transformer = java::ASTTransformer::new(source, filename);
        let ast = ast_transformer.transform(&root);

        if ast_transformer.base.errors.is_empty() {
            Ok(ast)
        } else {
            Err(ast_transformer.base.errors)
        }
    }
}