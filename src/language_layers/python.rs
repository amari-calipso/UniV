use std::rc::Rc;

use alanglib::ast::SourcePos;
use libcst_native::{deflated::{AnnAssign, Arg, Assert, Assign, AssignTarget, AssignTargetExpression, Attribute, AugAssign, AugOp, BaseSlice, BinaryOp, BinaryOperation, BooleanOp, BooleanOperation, Break, Call, ClassDef, CompOp, Comparison, CompoundStatement, ConcatenatedString, Continue, Del, DelTargetExpression, Dict, DictElement, Element, Expr, Float, For, FunctionDef, If, IfExp, IndentedBlock, Index, Integer, Lambda, List, Match, MatchCase, MatchPattern, Module, Name, OrElse, Parameters, Raise, Return, SimpleStatementLine, SimpleStatementSuite, SimpleString, SmallStatement, Statement, Subscript, SubscriptElement, Suite, Try, TryStar, Tuple, TypeAlias, UnaryOp, UnaryOperation, While, With}, tokenizer::TokType};

use crate::{error, language_layer, unil::{ast::{Expression, LiteralKind, NamedExpr, ObjectField, SwitchCase}, tokens::{Token, TokenType}}, utils::lang::{get_token_from_variable, get_vec_of_expr_from_block, make_null, BaseASTTransformer, Transform}, warning};

pub struct ASTTransformer {
    base: BaseASTTransformer,
    last_pos: SourcePos,
}

impl ASTTransformer {
    pub fn new(source: String, filename: Rc<str>) -> Self {
        ASTTransformer {
            last_pos: SourcePos::new(Rc::from(source.as_ref()), Rc::clone(&filename), 0, 0, 0),
            base: BaseASTTransformer::new(source, filename)
        }
    }

    pub fn tok_from_last_pos(&self) -> Token {
        self.tok_from_last_pos_with_lexeme("")
    }

    pub fn tok_from_last_pos_with_lexeme(&self, lexeme: &str) -> Token {
        self.tok_from_last_pos_with_type_and_lexeme(TokenType::Identifier, lexeme)
    }

    #[allow(dead_code)]
    pub fn tok_from_last_pos_with_type(&self, type_: TokenType) -> Token {
        self.tok_from_last_pos_with_type_and_lexeme(type_, "")
    }

    pub fn tok_from_last_pos_with_type_and_lexeme(&self, type_: TokenType, lexeme: &str) -> Token {
        Token {
            source: Rc::from(self.base.source.as_ref()),
            filename: Rc::clone(&self.base.filename),
            type_,
            lexeme: Rc::from(lexeme),
            pos: self.last_pos.start,
            end: self.last_pos.end,
            line: self.last_pos.line
        }
    }

    pub fn tok(&mut self, tok: &libcst_native::tokenizer::Token) -> Token {
        self.tok_with_lexeme(tok, tok.string)
    }

    pub fn tok_with_lexeme(&mut self, tok: &libcst_native::tokenizer::Token, lexeme: &str) -> Token {
        self.tok_with_type_and_lexeme(tok, {
            match tok.r#type {
                TokType::String => TokenType::String,
                TokType::Number => TokenType::Int,
                TokType::Op => TokenType::Equal,
                TokType::Name | TokType::Async | TokType::Await => TokenType::Identifier,
                _ => TokenType::Null
            }
        }, lexeme)
    }

    pub fn tok_with_type(&mut self, tok: &libcst_native::tokenizer::Token, type_: TokenType) -> Token {
        self.tok_with_type_and_lexeme(tok, type_, tok.string)
    }

    pub fn tok_with_type_and_lexeme(&mut self, tok: &libcst_native::tokenizer::Token, type_: TokenType, lexeme: &str) -> Token {
        self.last_pos.start = tok.start_pos.char_column_number();
        self.last_pos.end = tok.end_pos.char_column_number();
        self.last_pos.line = tok.start_pos.line_number() - 1;
        self.tok_from_last_pos_with_type_and_lexeme(type_, lexeme)
    }

    #[allow(dead_code)]
    pub fn todo(&mut self, tok: &libcst_native::tokenizer::Token) {
        let tok = self.tok(tok);
        error!(self.base, tok, "Not implemented yet");
    }

    #[allow(dead_code)]
    pub fn todo_ast(&mut self, expr: &Expression) {
        error!(self.base, expr, "Not implemented yet");
    }
}

impl Transform for Module<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Block {
            opening_brace: Token::empty(), // this is fine since it's ignored anyway
            expressions: self.body.iter().map(|x| x.to_ast(t)).collect()
        }
    }
}

impl Transform for Statement<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            Statement::Simple(x) => x.to_ast(t),
            Statement::Compound(x) => x.to_ast(t),
        }
    }
}

impl Transform for SimpleStatementLine<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if self.body.len() == 1 {
            self.body.first().unwrap().to_ast(t)
        } else {
            Expression::Block {
                opening_brace: t.tok(self.first_tok),
                expressions: self.body.iter().map(|x| x.to_ast(t)).collect()
            }
        }
    }
}

impl Transform for CompoundStatement<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            CompoundStatement::FunctionDef(x) => x.to_ast(t),
            CompoundStatement::If(x) => x.to_ast(t),
            CompoundStatement::For(x) => x.to_ast(t),
            CompoundStatement::While(x) => x.to_ast(t),
            CompoundStatement::Try(x) => x.to_ast(t),
            CompoundStatement::TryStar(x) => x.to_ast(t),
            CompoundStatement::With(x) => x.to_ast(t),
            CompoundStatement::Match(x) => x.to_ast(t),
            CompoundStatement::ClassDef(x) => x.to_ast(t),
        }
    }
}

fn params_to_vec_of_named_expr(params: &Parameters, t: &mut ASTTransformer, tok: &Token) -> Vec<NamedExpr> {
    if !params.kwonly_params.is_empty() {
        error!(t.base, tok, "Keyword-only parameters are not supported");
    }

    if params.star_arg.is_some() {
        error!(t.base, tok, "*args parameters are not supported");
    }

    if params.star_kwarg.is_some() {
        error!(t.base, tok, "**kwargs parameters are not supported");
    }

    let mut result = Vec::new();

    for param in params.posonly_params.iter().chain(params.params.iter()) {
        if param.default.is_some() {
            let token = t.tok(param.equal.as_ref().unwrap().tok);
            error!(t.base, token, "Default arguments are not supported");
        }

        result.push(NamedExpr {
            name: get_token_from_variable(param.name.to_ast(t)),
            expr: Some(Expression::Variable { 
                name: {
                    if let Some(x) = &param.comma {
                        t.tok_with_lexeme(x.tok, "any")
                    } else if let Some(x) = param.star_tok {
                        t.tok_with_lexeme(x, "any")
                    } else if let Some(x) = &param.equal {
                        t.tok_with_lexeme(x.tok, "any")
                    } else if let Some(x) = &param.annotation {
                        t.tok_with_lexeme(x.tok, "any")
                    } else {
                        t.tok_from_last_pos_with_lexeme("any")
                    }
                }
            })
        });
    }

    result
}

impl Transform for FunctionDef<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let def_tok = t.tok(self.def_tok);

        if t.base.depth != 0 {
            warning!(&def_tok, "Closures are not supported. This function will be defined in the global scope");
        }

        t.base.inc_depth();
        let body = get_vec_of_expr_from_block(self.body.to_ast(t));
        t.base.dec_depth();

        let function = {
            Expression::Function { 
                name: t.tok_with_lexeme(self.def_tok, &t.base.get_fn_name(self.name.value)), 
                params: params_to_vec_of_named_expr(&self.params, t, &def_tok), 
                return_type: Box::new(Expression::Variable { name: t.tok_with_lexeme(self.def_tok, "any") }), 
                body
            }
        };

        if self.decorators.is_empty() {
            function
        } else if self.decorators.len() == 1 {
            let dec = self.decorators.first().unwrap();
            let decorator = dec.decorator.to_ast(t);

            if let Expression::Call { callee, args, .. } = &decorator {
                if let Expression::Variable { name } = &**callee {
                    
                    match name.lexeme.to_lowercase().as_ref() {
                        "sort" => {
                            if args.len() < 3 || args.len() > 4 {
                                error!(t.base, decorator, format!("Expecting 3 or 4 arguments for sort decorator but got {}", args.len()).as_str());
                                return make_null();
                            }

                            let mut fields = vec![
                                ObjectField {
                                    name: t.tok_with_lexeme(dec.at_tok, "category"),
                                    expr: args[0].clone(),
                                    type_: None
                                },
                                ObjectField {
                                    name: t.tok_with_lexeme(dec.at_tok, "name"),
                                    expr: args[1].clone(),
                                    type_: None
                                },
                                ObjectField { 
                                    name: t.tok_with_lexeme(dec.at_tok, "listName"),
                                    expr: args[2].clone(),
                                    type_: None
                                },
                            ];
                            
                            if args.len() == 4 {
                                fields.push(ObjectField { 
                                    name: t.tok_with_lexeme(dec.at_tok, "killers"), 
                                    expr: args[3].clone(), 
                                    type_: None 
                                });
                            } 
                            
                            return Expression::AlgoDecl { 
                                name: t.tok_with_lexeme(dec.at_tok, "sort"), 
                                object: Box::new(Expression::AnonObject { 
                                    kw: t.tok(dec.at_tok), 
                                    fields
                                }), 
                                function: Box::new(function) 
                            }
                        }
                        "shuffle" => {
                            if args.len() != 1 {
                                error!(t.base, decorator, format!("Expecting 1 argument for shuffle decorator but got {}", args.len()).as_str());
                                return make_null();
                            }

                            return Expression::AlgoDecl { 
                                name: t.tok_with_lexeme(dec.at_tok, "shuffle"), 
                                object: Box::new(Expression::AnonObject { 
                                    kw: t.tok(dec.at_tok), 
                                    fields: vec![
                                        ObjectField { 
                                            name: t.tok_with_lexeme(dec.at_tok, "name"),
                                            expr: args[0].clone(),
                                            type_: None
                                        }
                                    ]
                                }), 
                                function: Box::new(function) 
                            }
                        }
                        "distribution" => {
                            if args.len() != 1 {
                                error!(t.base, decorator, format!("Expecting 1 argument for distribution decorator but got {}", args.len()).as_str());
                                return make_null();
                            }

                            return Expression::AlgoDecl { 
                                name: t.tok_with_lexeme(dec.at_tok, "distribution"), 
                                object: Box::new(Expression::AnonObject { 
                                    kw: t.tok(dec.at_tok), 
                                    fields: vec![
                                        ObjectField { 
                                            name: t.tok_with_lexeme(dec.at_tok, "name"),
                                            expr: args[0].clone(),
                                            type_: None
                                        }
                                    ]
                                }), 
                                function: Box::new(function) 
                            }
                        }
                        "pivotselection" => {
                            if args.len() != 1 {
                                error!(t.base, decorator, format!("Expecting 1 argument for pivot selection decorator but got {}", args.len()).as_str());
                                return make_null();
                            }

                            return Expression::AlgoDecl { 
                                name: t.tok_with_lexeme(dec.at_tok, "pivotSelection"), 
                                object: Box::new(Expression::AnonObject { 
                                    kw: t.tok(dec.at_tok), 
                                    fields: vec![
                                        ObjectField { 
                                            name: t.tok_with_lexeme(dec.at_tok, "name"),
                                            expr: args[0].clone(),
                                            type_: None
                                        }
                                    ]
                                }), 
                                function: Box::new(function) 
                            }
                        }
                        "rotation" => {
                            if args.len() == 0 || args.len() > 2 {
                                error!(t.base, decorator, format!("Expecting 1 or 2 arguments for rotation decorator but got {}", args.len()).as_str());
                                return make_null();
                            }

                            let mut name = None;
                            if args.len() == 2 {
                                if let Expression::Get { name: get_name, .. } = &args[1] {
                                    match get_name.lexeme.to_lowercase().as_str() {
                                        "indexed" => name = Some(t.tok_with_lexeme(dec.at_tok, "indexedRotation")),
                                        "lengths" => name = Some(t.tok_with_lexeme(dec.at_tok, "lengthsRotation")),
                                        _ => ()
                                    }
                                }
                            } else {
                                name = Some(t.tok_with_lexeme(dec.at_tok, "rotation"));
                            }

                            if let Some(name) = name {
                                return Expression::AlgoDecl { 
                                    name, 
                                    object: Box::new(Expression::AnonObject { 
                                        kw: t.tok(dec.at_tok), 
                                        fields: vec![
                                            ObjectField { 
                                                name: t.tok_with_lexeme(dec.at_tok, "name"),
                                                expr: args[0].clone(),
                                                type_: None
                                            }
                                        ]
                                    }), 
                                    function: Box::new(function) 
                                }
                            }
                        }
                        _ => ()
                    }
                }
            } 

            error!(t.base, decorator, "Unsupported decorator");
            function
        } else {
            let tok = t.tok(self.decorators.first().unwrap().at_tok);
            error!(t.base, tok, "Only one decorator is supported");
            function
        }
    }
}

impl Transform for Suite<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            Suite::IndentedBlock(x) => x.to_ast(t),
            Suite::SimpleStatementSuite(x) => x.to_ast(t),
        }
    }
}

impl Transform for IndentedBlock<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Block {
            opening_brace: t.tok(self.indent_tok),
            expressions: self.body.iter().map(|x| x.to_ast(t)).collect()
        }
    }
}

impl Transform for SimpleStatementSuite<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Block {
            opening_brace: t.tok(self.first_tok),
            expressions: self.body.iter().map(|x| x.to_ast(t)).collect()
        }
    }
}

impl Transform for If<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::If { 
            kw: t.tok(self.if_tok), 
            condition: Box::new(self.test.to_ast(t)), 
            then_branch: Box::new(self.body.to_ast(t)), 
            else_branch: self.orelse.as_ref().map(|x| Box::new(x.to_ast(t)))
        }
    }
}

impl Transform for OrElse<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            OrElse::Elif(x) => x.to_ast(t),
            OrElse::Else(x) => x.body.to_ast(t),
        }
    }
}

fn handle_loop_break(loop_: Expression, then_branch: Expression, tok: &libcst_native::tokenizer::Token<'_>, t: &mut ASTTransformer) -> Expression {
    let catch_tmp = t.base.tmp_var();
    let catch_tmp_tok = t.tok_with_lexeme(tok, &catch_tmp);
    let catch_tmp_var = Expression::Variable { name: catch_tmp_tok.clone() };

    Expression::Try {
        kw: t.tok(tok),
        try_branch: Box::new(loop_),
        catch_branch: Some(Box::new(Expression::If { 
            kw: t.tok(tok), 
            condition: Box::new(Expression::Call { 
                callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "hasAttribute") }), 
                paren: t.tok(tok), 
                args: vec![
                    catch_tmp_var.clone(),
                    Expression::Literal { 
                        value: Rc::from("__Python_break"), 
                        tok: t.tok(tok), 
                        kind: LiteralKind::String 
                    }
                ] 
            }), 
            then_branch: Box::new(then_branch), 
            else_branch: Some(Box::new(Expression::Throw { 
                kw: t.tok(tok), 
                value: Some(Box::new(catch_tmp_var))
            }))
        })),
        catch_var: Some(catch_tmp_tok),
    }  
}

impl Transform for For<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let for_loop = {
            Expression::Foreach { 
                kw: t.tok(self.for_tok), 
                variable: {
                    let expr = self.target.to_ast(t);
                    if let Expression::Variable { name } = expr {
                        name
                    } else {
                        error!(t.base, expr, "Only variables are supported as target of for loop");
                        t.tok(self.for_tok)
                    }
                }, 
                iterator: Box::new(self.iter.to_ast(t)), 
                body: Box::new(self.body.to_ast(t))
            }
        };

        if let Some(orelse) = &self.orelse {
            handle_loop_break(for_loop, orelse.body.to_ast(t), self.for_tok, t)
        } else {
            handle_loop_break(for_loop, make_null(), self.for_tok, t)
        }
    }
}

impl Transform for While<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let while_loop = {
            Expression::While { 
                kw: t.tok(self.while_tok), 
                condition: Box::new(self.test.to_ast(t)), 
                body: Box::new(self.body.to_ast(t)),
                increment: None
            }
        };
        
        if let Some(orelse) = &self.orelse {
            handle_loop_break(while_loop, orelse.body.to_ast(t), self.while_tok, t)
        } else {
            handle_loop_break(while_loop, make_null(), self.while_tok, t)
        }
    }
}

macro_rules! transform_try {
    ($node: expr, $t: ident) => {
        {
            if $node.handlers.len() > 1 {
                let tok = $t.tok($node.handlers.first().unwrap().except_tok);
                error!($t.base, tok, "Only one exception handler is supported");
            }
    
            let try_branch;
            if let Some(orelse) = &$node.orelse {
                try_branch = Expression::Block { 
                    opening_brace: $t.tok(orelse.else_tok), 
                    expressions: vec![$node.body.to_ast($t), orelse.body.to_ast($t)] 
                };
            } else {
                try_branch = $node.body.to_ast($t);
            }
    
            let try_expr;
            if let Some(except) = $node.handlers.first() {
                try_expr = Expression::Try { 
                    kw: $t.tok($node.try_tok), 
                    try_branch: Box::new(try_branch), 
                    catch_branch: Some(Box::new(except.body.to_ast($t))), 
                    catch_var: except.name.as_ref().map(|x| get_token_from_variable(x.name.to_ast($t)))
                };
            } else {
                try_expr = Expression::Try { 
                    kw: $t.tok($node.try_tok), 
                    try_branch: Box::new(try_branch), 
                    catch_branch: None, 
                    catch_var: None
                };
            }
    
            if let Some(finally) = &$node.finalbody {
                Expression::Block { 
                    opening_brace: $t.tok(finally.finally_tok), 
                    expressions: vec![try_expr, finally.body.to_ast($t)] 
                }
            } else {
                try_expr
            }
        }
    };
}

impl Transform for Try<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        transform_try!(self, t)
    }
}

impl Transform for TryStar<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        transform_try!(self, t)
    }
}

impl Transform for With<'_, '_> {
    type Transformer = ASTTransformer;

    // https://peps.python.org/pep-0343/
    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if self.items.len() != 1 {
            let expr = self.items.first().unwrap().item.to_ast(t);
            error!(t.base, expr, "Only one with item is supported");

            if self.items.len() == 0 {
                return make_null();
            }
        }

        let t0 = t.base.tmp_var();
        let t1 = t.base.tmp_var();
        let t2 = t.base.tmp_var();

        let mgr = Expression::Variable { name: t.tok_with_lexeme(self.with_tok, &t0) };
        let exit = Expression::Variable { name: t.tok_with_lexeme(self.with_tok, &t1) };
        let value = Expression::Variable { name: t.tok_with_lexeme(self.with_tok, &t2) };

        let tok = t.tok_with_type(self.with_tok, TokenType::Walrus);

        Expression::Block { 
            opening_brace: tok.clone(), 
            expressions: vec![
                Expression::Assign { 
                    target: Box::new(mgr.clone()), 
                    op: tok.clone(), 
                    value: Box::new(self.items.first().unwrap().item.to_ast(t)), 
                    type_spec: None
                },
                Expression::Assign { 
                    target: Box::new(exit.clone()), 
                    op: tok.clone(), 
                    value: Box::new(Expression::Get { 
                        object: Box::new(mgr.clone()), 
                        name: t.tok_with_lexeme(self.with_tok, "__exit__")
                    }), 
                    type_spec: None
                },
                Expression::Assign { 
                    target: Box::new(value.clone()), 
                    op: tok.clone(), 
                    value: Box::new(Expression::Call { 
                        callee: Box::new(Expression::Get { 
                            object: Box::new(mgr.clone()), 
                            name: t.tok_with_lexeme(self.with_tok, "__enter__")
                        }), 
                        paren: tok.clone(), 
                        args: vec![mgr.clone()] 
                    }), 
                    type_spec: None
                },
                Expression::Try { 
                    kw: tok.clone(), 
                    try_branch: Box::new({
                        if let Some(var) = &self.items.first().unwrap().asname {
                            Expression::Block { 
                                opening_brace: tok.clone(), 
                                expressions: vec![
                                    Expression::Assign { 
                                        target: Box::new(var.name.to_ast(t)), 
                                        op: tok.clone(), 
                                        value: Box::new(value), 
                                        type_spec: Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(self.with_tok, "any") }))
                                    },
                                    self.body.to_ast(t)
                                ]
                            }
                        } else {
                            self.body.to_ast(t)
                        }
                    }), 
                    catch_branch: None, 
                    catch_var: None 
                },
                Expression::Call { 
                    callee: Box::new(exit), 
                    paren: tok, 
                    args: vec![mgr, make_null(), make_null(), make_null()] 
                }
            ]
        }
    }
}

fn transform_pattern(pattern: &MatchPattern, t: &mut ASTTransformer, tok: &Token) -> Option<Vec<Expression>> {
    match &pattern {
        MatchPattern::Value(x) => Some(vec![x.value.to_ast(t)]),
        MatchPattern::Singleton(x) => {
            if x.value.value == "_" {
                None
            } else {
                error!(t.base, tok, "Unsupported kind of pattern");
                None
            }
        }
        MatchPattern::As(x) => {
            if let Some(inner_pattern) = &x.pattern {
                transform_pattern(inner_pattern, t, tok)
            } else if let Some(name) = &x.name {
                if name.value == "_" {
                    None
                } else {
                    error!(t.base, tok, "Unsupported kind of pattern");
                    None
                }
            } else {
                None
            }
        }
        MatchPattern::Or(x) => {
            Some(x.patterns.iter().map(|x| {
                if let Some(patterns) = transform_pattern(&x.pattern, t, tok) {
                    if patterns.len() == 1 {
                        return patterns.first().unwrap().clone();
                    }
                }

                error!(t.base, tok, "Unsupported kind of pattern");
                make_null()
            }).collect())
        }
        MatchPattern::Mapping(_) |
        MatchPattern::Sequence(_) | 
        MatchPattern::Class(_) => {
            error!(t.base, tok, "Unsupported kind of pattern");
            None
        }
    }
}

fn transform_match_case(case: &MatchCase, subject: &Expression, t: &mut ASTTransformer) -> SwitchCase {
    let tok = t.tok(case.case_tok);
    SwitchCase { 
        cases: transform_pattern(&case.pattern, t, &tok), 
        code: {
            let mut code = Vec::new();

            if let MatchPattern::As(x) = &case.pattern {
                if let Some(name) = &x.name {
                    code.push(Expression::Assign { 
                        target: Box::new(name.to_ast(t)), 
                        op: t.tok_with_type(case.case_tok, TokenType::Walrus), 
                        value: Box::new(subject.clone()), 
                        type_spec: Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(case.case_tok, "any") })) 
                    });
                }
            }

            if let Some(guard) = &case.guard {
                code.push(Expression::If { 
                    kw: t.tok(case.case_tok), 
                    condition: Box::new(guard.to_ast(t)), 
                    then_branch: Box::new(case.body.to_ast(t)), 
                    else_branch: None 
                });
            } else {
                code.push(case.body.to_ast(t));
            }

            code
        } 
    }
}

impl Transform for Match<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let t0 = t.base.tmp_var();
        let subject = Expression::Variable { name: t.tok_with_lexeme(self.match_tok, &t0) };
        let op = t.tok_with_type(self.match_tok, TokenType::Walrus);

        Expression::Block { 
            opening_brace: op.clone(), 
            expressions: vec![
                Expression::Assign { 
                    target: Box::new(subject.clone()), 
                    op: op.clone(), 
                    value: Box::new(self.subject.to_ast(t)), 
                    type_spec: None 
                },
                Expression::Switch { 
                    kw: op, 
                    expr: Box::new(subject.clone()), 
                    cases: self.cases.iter().map(|x| transform_match_case(x, &subject, t)).collect()
                }
            ] 
        }
    }
}

fn get_class_definitions(body: &Vec<Expression>, class_name: &Token, fields: &mut Vec<ObjectField>, init: &mut Option<Expression>, t: &mut ASTTransformer) {
    for element in body {
        match &element {
            Expression::Assign { target, op, value, .. } => {
                if !matches!(op.type_, TokenType::Walrus) {
                    error!(t.base, op, "Only '=' assignments are allowed in class body");
                }

                if let Expression::Variable { name } = &**target {
                    let mut any = op.clone();
                    any.set_lexeme("any");

                    fields.push(ObjectField::new(name.clone(), *value.clone(), Some(Expression::Variable { name: any })));
                } else {
                    error!(t.base, target, "Only variables are supported as assignment targets in class body");
                }
            }
            Expression::Function { name, params, .. } => {
                let actual_name = name.lexeme.strip_prefix(format!("{}__", class_name.lexeme).as_str()).unwrap_or("_");
                if actual_name == "__init__" {
                    *init = Some(element.clone());
                }

                if params.len() == 0 {
                    error!(t.base, name, "Methods must take at least 1 parameter (static methods are not supported)");
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
                get_class_definitions(expressions, class_name, fields, init, t);
            }
            _ => error!(t.base, element, "Unsupported definition in class body")
        }
    }
}

impl Transform for ClassDef<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if !self.decorators.is_empty() {
            let tok = t.tok(self.decorators.first().unwrap().at_tok);
            warning!(&tok, "Decorators are not supported on classes. Ignoring");
        }

        if !self.keywords.is_empty() || !self.bases.is_empty() {
            let tok = t.tok(self.class_tok);
            warning!(&tok, "Inheritance is not supported. Ignoring");
        }

        let class_name = get_token_from_variable(self.name.to_ast(t));

        let old_curr_name = t.base.curr_name.clone();
        t.base.curr_name.push_str(&class_name.lexeme);
        let body = self.body.to_ast(t);
        t.base.curr_name = old_curr_name;

        let mut fields = Vec::new();
        let mut init = None;

        let mut definitions = get_vec_of_expr_from_block(body);
        get_class_definitions(&definitions, &class_name, &mut fields, &mut init, t);

        let tok = t.tok_with_type(self.class_tok, TokenType::Walrus);
        let object = Expression::AnonObject { kw: tok.clone(), fields };

        if let Some(init_fn) = init {
            if let Expression::Function { mut params, body, .. } = init_fn {
                let slf = Expression::Variable { name: params[0].name.clone() };
                params.remove(0); // removes "self" from constructor

                let mut fn_body: Vec<Expression> = [
                    Expression::Assign { 
                        target: Box::new(slf.clone()), 
                        op: tok, 
                        value: Box::new(object), 
                        type_spec: None 
                    }
                ].into_iter().chain(body.into_iter()).collect();
                fn_body.push(slf);

                definitions.push(Expression::Function { 
                    name: class_name, 
                    params, 
                    return_type: Box::new(Expression::Variable { name: t.tok_with_lexeme(self.class_tok, "any") }), 
                    body: fn_body
                });
            } else {
                unreachable!()
            }
        } else {
            definitions.push(Expression::Function { 
                name: class_name, 
                params: Vec::new(), 
                return_type: Box::new(Expression::Variable { name: t.tok_with_lexeme(self.class_tok, "any") }), 
                body: vec![object] 
            });
        }

        // removes static variables declarations, which would get turned into globals without namespaces
        definitions.retain(|x| !matches!(x, Expression::Assign { .. }));

        Expression::Block { 
            opening_brace: t.tok(self.class_tok), 
            expressions: definitions 
        }
    }
}

impl Transform for SmallStatement<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            SmallStatement::Pass(_) | SmallStatement::Import(_) | 
            SmallStatement::ImportFrom(_) |SmallStatement::Nonlocal(_) => make_null(),
            SmallStatement::Assert(x) => x.to_ast(t),
            SmallStatement::Break(x) => x.to_ast(t),
            SmallStatement::Continue(x) => x.to_ast(t),
            SmallStatement::Return(x) => x.to_ast(t),
            SmallStatement::Expr(x) => x.to_ast(t),
            SmallStatement::Assign(x) => x.to_ast(t),
            SmallStatement::AnnAssign(x) => x.to_ast(t),
            SmallStatement::Raise(x) => x.to_ast(t),
            SmallStatement::AugAssign(x) => x.to_ast(t),
            SmallStatement::Del(x) => x.to_ast(t),
            SmallStatement::TypeAlias(x) => x.to_ast(t),
            SmallStatement::Global(x) => {
                let tok = t.tok(x.tok);
                error!(t.base, tok, "Editing globals from a local scope is not supported");
                make_null()
            } 
        }
    }
}

impl Transform for Assert<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let tok = t.tok(self.assert_tok);

        Expression::If {
            kw: tok.clone(),
            condition: Box::new(Expression::Unary { 
                op: t.tok_with_type(self.assert_tok, TokenType::Bang), 
                expr: Box::new(self.test.to_ast(t)), 
                is_prefix: true 
            }),
            then_branch: Box::new(Expression::Throw { 
                kw: tok.clone(), 
                value: Some(Box::new(Expression::Literal { 
                    value: Rc::from("Assertion failed"), 
                    tok: tok.clone(), 
                    kind: LiteralKind::String
                }))
            }),
            else_branch: None,
        }
    }
}

impl Transform for Break<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let tok = t.tok(self.tok);

        let mut name = tok.clone();
        name.set_lexeme("__Python_break");

        // an exception is thrown because of the else clause in loops:
        // that becomes the catch clause of a try statement that contains the loop
        Expression::Block { 
            opening_brace: tok.clone(), 
            expressions: vec![
                Expression::Throw { 
                    kw: tok.clone(), 
                    value: Some(Box::new(Expression::AnonObject { 
                        kw: tok.clone(), 
                        fields: vec![
                            ObjectField {
                                name,
                                expr: make_null(),
                                type_: None
                            }
                        ]
                    }))
                },
                // allows UniL to detect this as a break, and give proper errors about it
                Expression::Break { kw: tok, value: None }
            ]
        }
    }
}

impl Transform for Continue<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Continue { kw: t.tok(self.tok), value: None }
    }
}

impl Transform for Return<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Return {
            kw: t.tok(self.return_tok),
            value: self.value.as_ref().map(|x| Box::new(x.to_ast(t)))
        }
    }
}

impl Transform for Expr<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        self.value.to_ast(t)
    }
}

macro_rules! tuple_assign {
    ($slf: ident, $t: ident, $op: ident, $tuple: ident) => {
        {
            let value = $slf.value.to_ast($t);

            // look for array[a], array[b] = array[b], array[a] pattern and replace it with swap(array, a, b)
            if let Expression::List { items, .. } = &value {
                if $tuple.elements.len() == 2 && items.len() == 2 {
                    let element0 = $tuple.elements[0].to_ast($t);
                    let element1 = $tuple.elements[1].to_ast($t);

                    if let Expression::Subscript { subscripted: left_sub, index: left_idx, .. } = &element0 {
                        if let Expression::Subscript { subscripted: right_sub, index: right_idx, .. } = &element1 {
                            if element0.equals(&items[1]) && element1.equals(&items[0]) && left_sub.equals(&right_sub) {
                                return Expression::Block { 
                                    opening_brace: $op.clone(), 
                                    expressions: vec![
                                        // subscript operation would normally be evaluated 4 times, so clone it to keep side effects (if any)
                                        *left_sub.clone(),
                                        *left_sub.clone(),
                                        *left_sub.clone(), 
                                        // left and right indices would be evaluated 1 additional time each, so do the same
                                        *left_idx.clone(),
                                        *right_idx.clone(),
                                        // now output the call itself
                                        Expression::Call { 
                                            callee: Box::new(Expression::Variable { 
                                                name: {
                                                    let mut name = $op.clone();
                                                    name.set_lexeme("swap");
                                                    name
                                                }
                                            }),
                                            paren: $op, 
                                            args: vec![*left_sub.clone(), *left_idx.clone(), *right_idx.clone()] 
                                        }
                                    ]
                                }
                            }
                        }
                    }
                }
            }
            
            let tmp = $t.base.tmp_var();

            let mut name = $op.clone();
            name.set_lexeme(&tmp);

            let mut any = $op.clone();
            any.set_lexeme("any");

            let tmp_var = Box::new(Expression::Variable { name });

            Expression::Block {
                opening_brace: $op.clone(),
                expressions: [
                    Expression::Assign {
                        target: tmp_var.clone(),
                        op: $op.clone(),
                        value: Box::new(value),
                        type_spec: None
                    }
                ].into_iter().chain($tuple.elements.iter().enumerate().map(|(i, x)| {
                    Expression::Assign {
                        target: Box::new(x.to_ast($t)),
                        op: $op.clone(),
                        value: Box::new(Expression::Subscript { 
                            subscripted: tmp_var.clone(), 
                            paren: $op.clone(), 
                            index: Box::new(Expression::Literal { 
                                value: i.to_string().into(), 
                                tok: $op.clone(), 
                                kind: LiteralKind::Int 
                            })
                        }),
                        type_spec: Some(Box::new(Expression::Variable { name: any.clone() })) 
                    }
                })).collect()
            }
        }
    };
}

impl Transform for Assign<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let op = t.tok_with_type(self.targets.first().unwrap().equal_tok, TokenType::Walrus);
        
        if self.targets.len() == 1 {
            let target = &self.targets.first().unwrap().target;
            match target {
                AssignTargetExpression::Tuple(x) => tuple_assign!(self, t, op, x),
                AssignTargetExpression::List(x) => tuple_assign!(self, t, op, x),
                _ => {
                    let target = target.to_ast(t);

                    Expression::Assign {
                        type_spec: {
                            if let Expression::Subscript { .. } = &target {
                                None
                            } else {
                                let mut name = op.clone();
                                name.set_lexeme("any");

                                Some(Box::new(Expression::Variable { name })) 
                            }
                        },
                        target: Box::new(target),
                        op,
                        value: Box::new(self.value.to_ast(t)),
                    }
                }
            }
        } else {
            let tmp = t.base.tmp_var();

            let mut name = op.clone();
            name.set_lexeme(&tmp);

            let tmp_var = Box::new(Expression::Variable { name });

            Expression::Block {
                opening_brace: op.clone(),
                expressions: [
                    Expression::Assign {
                        target: tmp_var.clone(),
                        op: op.clone(),
                        value: Box::new(self.value.to_ast(t)),
                        type_spec: None
                    }
                ].into_iter().chain(self.targets.iter().map(|x| {
                    let target = x.to_ast(t);

                    Expression::Assign {
                        type_spec: {
                            if let Expression::Subscript { .. } = &target {
                                None
                            } else {
                                let mut any = op.clone();
                                any.set_lexeme("any");

                                Some(Box::new(Expression::Variable { name: any }))
                            }
                        },
                        target: Box::new(target),
                        op: op.clone(),
                        value: tmp_var.clone(),
                    }
                })).collect()
            }
        }
    }
}

impl Transform for AssignTarget<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        self.target.to_ast(t)
    }
}

impl Transform for AssignTargetExpression<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            AssignTargetExpression::Name(x) => x.to_ast(t),
            AssignTargetExpression::Attribute(x) => x.to_ast(t),
            AssignTargetExpression::Subscript(x) => x.to_ast(t),
            AssignTargetExpression::Tuple(x) => {
                let expr = x.to_ast(t);
                error!(t.base, expr, "Unsupported assignment target");
                make_null()
            }
            AssignTargetExpression::List(x) => {
                let expr = x.to_ast(t);
                error!(t.base, expr, "Unsupported assignment target");
                make_null()
            }
            AssignTargetExpression::StarredElement(x) => {
                let tok = t.tok(x.star_tok);
                error!(t.base, tok, "Unpacking is not supported");
                make_null()
            }
        }
    }
}

impl Transform for Name<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let name = t.base.get_var_name(self.value);
        Expression::Variable { name: t.tok_with_lexeme(self.tok, &name) }
    }
}

impl Transform for Attribute<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Get {
            object: Box::new(self.value.to_ast(t)),
            name: get_token_from_variable(self.attr.to_ast(t))
        }
    }
}

impl Transform for Subscript<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let paren = t.tok(self.lbracket.tok);

        if self.slice.len() != 1 {
            error!(t.base, paren, "Slicing is not supported");
        }

        Expression::Subscript {
            subscripted: Box::new(self.value.to_ast(t)),
            paren,
            index: Box::new(self.slice.first().unwrap().to_ast(t))
        }
    }
}

impl Transform for SubscriptElement<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        self.slice.to_ast(t)
    }
}

impl Transform for BaseSlice<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            BaseSlice::Index(x) => x.to_ast(t),
            BaseSlice::Slice(x) => {
                let tok = t.tok(x.first_colon.tok);
                error!(t.base, tok, "Slicing is not supported");
                make_null()
            }
        }
    }
}

impl Transform for Index<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if self.star.is_some() {
            let tok = t.tok(self.star_tok.unwrap());
            error!(t.base, tok, "Unsupported syntax");
        }

        self.value.to_ast(t)
    }
}

impl Transform for AnnAssign<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if let Some(value) = &self.value {
            let target = self.target.to_ast(t);

            Expression::Assign {
                type_spec: {
                    if let Expression::Subscript { .. } = &target {
                        None
                    } else {
                        Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(self.equal.as_ref().unwrap().tok, "any") })) 
                    }
                },
                target: Box::new(target),
                op: t.tok_with_type(self.equal.as_ref().unwrap().tok, TokenType::Walrus),
                value: Box::new(value.to_ast(t)),
            }
        } else {
            make_null()
        }
    }
}

impl Transform for Raise<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Throw {
            kw: t.tok(self.raise_tok),
            value: self.exc.as_ref().map(|x| Box::new(x.to_ast(t)))
        }
    }
}

impl Transform for AugAssign<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let mut output = Vec::new();

        let mut target = self.target.to_ast(t);
        let mut value = self.value.to_ast(t);

        let tok = {
            match self.operator {
                AugOp::AddAssign { tok } |
                AugOp::SubtractAssign { tok } |
                AugOp::MultiplyAssign { tok } |
                AugOp::MatrixMultiplyAssign { tok } |
                AugOp::DivideAssign { tok } |
                AugOp::ModuloAssign { tok } |
                AugOp::BitAndAssign { tok } |
                AugOp::BitOrAssign { tok } |
                AugOp::BitXorAssign { tok } |
                AugOp::LeftShiftAssign { tok } |
                AugOp::RightShiftAssign { tok } |
                AugOp::PowerAssign { tok } |
                AugOp::FloorDivideAssign { tok } => t.tok(tok),
            }
        };

        // avoids double evaluation
        match target {
            Expression::Get { object, name } => {
                let mut tmp = tok.clone();
                tmp.set_lexeme(&t.base.tmp_var());

                let mut op = tok.clone();
                op.set_type(TokenType::Walrus);

                let tmp_var = Box::new(Expression::Variable { name: tmp });

                output.push(
                    Expression::Assign { 
                        target: tmp_var.clone(), 
                        op, 
                        value: object, 
                        type_spec: None
                    }
                );

                target = Expression::Get {
                    object: tmp_var,
                    name
                };
            }
            Expression::Subscript { subscripted, index, paren } => {
                let mut tmp0 = tok.clone();
                tmp0.set_lexeme(&t.base.tmp_var());

                let mut tmp1 = tok.clone();
                tmp1.set_lexeme(&t.base.tmp_var());

                let mut op = tok.clone();
                op.set_type(TokenType::Walrus);

                let subscripted_tmp_var = Box::new(Expression::Variable { name: tmp0 });
                let index_tmp_var = Box::new(Expression::Variable { name: tmp1 });

                output.push(
                    Expression::Assign { 
                        target: subscripted_tmp_var.clone(), 
                        op: op.clone(), 
                        value: subscripted, 
                        type_spec: None
                    }
                );

                output.push(
                    Expression::Assign { 
                        target: index_tmp_var.clone(), 
                        op, 
                        value: index, 
                        type_spec: None
                    }
                );

                target = Expression::Subscript { 
                    subscripted: subscripted_tmp_var, 
                    paren, 
                    index: index_tmp_var
                };
            }
            _ => ()
        }

        let type_spec;
        let op = {
            match self.operator {
                AugOp::AddAssign { tok } |
                AugOp::SubtractAssign { tok } |
                AugOp::MultiplyAssign { tok } |
                AugOp::ModuloAssign { tok } |
                AugOp::BitAndAssign { tok } |
                AugOp::BitOrAssign { tok } |
                AugOp::BitXorAssign { tok } |
                AugOp::LeftShiftAssign { tok } |
                AugOp::RightShiftAssign { tok } => {
                    type_spec = Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "any") }));

                    let type_ = {
                        match self.operator {
                            AugOp::AddAssign { .. } => TokenType::Plus,
                            AugOp::SubtractAssign { .. } => TokenType::Minus,
                            AugOp::MultiplyAssign { .. } => TokenType::Star,
                            AugOp::ModuloAssign { .. } => TokenType::Mod,
                            AugOp::BitAndAssign { .. } => TokenType::BitwiseAnd,
                            AugOp::BitOrAssign { .. } => TokenType::BitwiseOr,
                            AugOp::BitXorAssign { .. } => TokenType::BitwiseXor,
                            AugOp::LeftShiftAssign { .. } => TokenType::ShiftLeft,
                            AugOp::RightShiftAssign { .. } => TokenType::ShiftRight,
                            _ => unreachable!()
                        }
                    };

                    value = Expression::Binary { 
                        left: Box::new(target.clone()), 
                        op: t.tok_with_type(tok, type_), 
                        right: Box::new(value)
                    };

                    t.tok_with_type(tok, TokenType::Walrus)
                }
                AugOp::PowerAssign { tok } => {
                    type_spec = Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "any") }));

                    let op = t.tok_with_type(tok, TokenType::Walrus);
                    let name = t.tok_with_lexeme(tok, "math_pow");

                    value = Expression::Call {
                        callee: Box::new(Expression::Variable { name }),
                        paren: op.clone(),
                        args: vec![target.clone(), value]
                    };
    
                    op
                }
                AugOp::DivideAssign { tok } => {
                    type_spec = Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "any") }));

                    value = Expression::Binary { 
                        left: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "float") }), 
                            paren: t.tok(tok), 
                            args: vec![target.clone()]
                        }), 
                        op: t.tok_with_type(tok, TokenType::Slash), 
                        right: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "float") }), 
                            paren: t.tok(tok), 
                            args: vec![value]
                        })
                    };

                    t.tok_with_type(tok, TokenType::Walrus)
                }
                AugOp::FloorDivideAssign { tok } => {
                    type_spec = Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "any") }));

                    value = Expression::Binary { 
                        left: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "int") }), 
                            paren: t.tok(tok), 
                            args: vec![target.clone()]
                        }), 
                        op: t.tok_with_type(tok, TokenType::Slash), 
                        right: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "int") }), 
                            paren: t.tok(tok), 
                            args: vec![value]
                        })
                    };

                    t.tok_with_type(tok, TokenType::Walrus)
                }
                AugOp::MatrixMultiplyAssign { tok } => {
                    type_spec = Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "any") }));

                    let tok = t.tok(tok);
                    error!(t.base, tok, "Unsupported operator");
                    tok
                }
            }
        };

        if output.len() == 0 {
            Expression::Assign {
                target: Box::new(target),
                op,
                value: Box::new(value),
                type_spec
            }
        } else {
            Expression::Block { 
                opening_brace: op.clone(), 
                expressions: output.into_iter().chain([
                    Expression::Assign {
                        target: Box::new(target),
                        op,
                        value: Box::new(value),
                        type_spec
                    }
                ]).collect() 
            }
        }
    }
}

impl Transform for Del<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let kw = t.tok(self.tok);

        if let DelTargetExpression::Name(name) = &self.target {
            Expression::Drop {
                kw,
                variable: get_token_from_variable(name.to_ast(t))
            }
        } else {
            error!(t.base, kw, "Only deletion of variables is supported");
            make_null()
        }
    }
}

impl Transform for TypeAlias<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let tok = t.tok(self.type_tok);
        error!(t.base, tok, "Type aliases are not supported");
        make_null()
    }
}

impl Transform for libcst_native::deflated::Expression<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            Self::Name(x) => x.to_ast(t),
            Self::Integer(x) => x.to_ast(t),
            Self::Float(x) => x.to_ast(t),
            Self::Comparison(x) => x.to_ast(t),
            Self::UnaryOperation(x) => x.to_ast(t),
            Self::BinaryOperation(x) => x.to_ast(t),
            Self::BooleanOperation(x) => x.to_ast(t),
            Self::Attribute(x) => x.to_ast(t),
            Self::Tuple(x) => x.to_ast(t),
            Self::List(x) => x.to_ast(t),
            Self::Call(x) => x.to_ast(t),
            Self::Subscript(x) => x.to_ast(t),
            Self::IfExp(x) => x.to_ast(t),
            Self::Lambda(x) => x.to_ast(t),
            Self::SimpleString(x) => x.to_ast(t),
            Self::ConcatenatedString(x) => x.to_ast(t),
            Self::NamedExpr(x) => x.to_ast(t),
            Self::Dict(x) => x.to_ast(t),

            Self::Imaginary(_) |
            Self::GeneratorExp(_) |
            Self::ListComp(_) |
            Self::SetComp(_) |
            Self::DictComp(_) |
            Self::Set(_) |
            Self::StarredElement(_) |
            Self::Yield(_) |
            Self::Await(_) |
            Self::FormattedString(_) => {
                let tok = t.tok_from_last_pos();
                error!(t.base, tok, "Unsupported expression");
                make_null()
            }
            _ => make_null()
        }
    }
}

impl Transform for Integer<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Literal {
            value: Rc::from(self.value), // TODO: we should convert this to a valid rust-parsable int literal
            tok: t.tok(self.tok),
            kind: LiteralKind::Int
        }
    }
}

impl Transform for Float<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Literal {
            value: Rc::from(self.value), // TODO: we should convert this to a valid rust-parsable float literal
            tok: t.tok(self.tok),
            kind: LiteralKind::Float
        }
    }
}

impl Transform for Comparison<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if self.comparisons.len() == 1 {
            let op = {
                match self.comparisons.first().unwrap().operator {
                    CompOp::LessThan { tok } => t.tok_with_type(tok, TokenType::Less),
                    CompOp::GreaterThan { tok } => t.tok_with_type(tok, TokenType::Greater),
                    CompOp::LessThanEqual { tok } => t.tok_with_type(tok, TokenType::LessEqual),
                    CompOp::GreaterThanEqual { tok } => t.tok_with_type(tok, TokenType::GreaterEqual),
                    CompOp::Equal { tok } | 
                    CompOp::Is { tok } => t.tok_with_type(tok, TokenType::EqualEqual),
                    CompOp::NotEqual { tok: is_tok } | 
                    CompOp::IsNot { is_tok, .. } => t.tok_with_type(is_tok, TokenType::BangEqual),
                    CompOp::In { tok } => {
                        return Expression::Call {
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "contains") }),
                            paren: t.tok(tok),
                            args: vec![self.comparisons.first().unwrap().comparator.to_ast(t), self.left.to_ast(t)]
                        };
                    }
                    CompOp::NotIn { in_tok, .. } => {
                        return Expression::Unary { 
                            op: t.tok_with_type(in_tok, TokenType::Bang), 
                            expr: Box::new(Expression::Call {
                                callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(in_tok, "contains") }),
                                paren: t.tok(in_tok),
                                args: vec![self.comparisons.first().unwrap().comparator.to_ast(t), self.left.to_ast(t)]
                            }), 
                            is_prefix: true 
                        };
                    }
                }
            };

            Expression::Cmp {
                left: Box::new(self.left.to_ast(t)),
                op,
                right: Box::new(self.comparisons.first().unwrap().comparator.to_ast(t))
            }
        } else {
            let mut last = Some(self.left.to_ast(t));
            let mut expr = None;
            for expression in &self.comparisons {
                let op = {
                    match expression.operator {
                        CompOp::LessThan { tok } => t.tok_with_type(tok, TokenType::Less),
                        CompOp::GreaterThan { tok } => t.tok_with_type(tok, TokenType::Greater),
                        CompOp::LessThanEqual { tok } => t.tok_with_type(tok, TokenType::LessEqual),
                        CompOp::GreaterThanEqual { tok } => t.tok_with_type(tok, TokenType::GreaterEqual),
                        CompOp::Equal { tok } | 
                        CompOp::Is { tok } => t.tok_with_type(tok, TokenType::EqualEqual),
                        CompOp::NotEqual { tok: is_tok } | 
                        CompOp::IsNot { is_tok, .. } => t.tok_with_type(is_tok, TokenType::BangEqual),
                        CompOp::In { tok } | CompOp::NotIn { in_tok: tok, .. } => {
                            let tok = t.tok(tok);
                            error!(t.base, tok, "Unsupported syntax");
                            continue;
                        }
                    }
                };

                let mut tmp = op.clone();
                tmp.set_lexeme(&t.base.tmp_var());

                let tmp_var = Expression::Variable { name: tmp };

                let mut eq_op = op.clone();
                eq_op.set_type(TokenType::Walrus);

                let right = Expression::Assign { 
                    target: Box::new(tmp_var.clone()), 
                    op: eq_op, 
                    value: Box::new(expression.comparator.to_ast(t)), 
                    type_spec: None 
                };

                let inner = Expression::Cmp { 
                    left: Box::new(last.take().unwrap()), 
                    op: op.clone(), 
                    right: Box::new(right) 
                };

                if expr.is_none() {
                    expr = Some(inner);
                } else {
                    let mut and = op;
                    and.set_type(TokenType::LogicAnd);

                    expr = Some(Expression::Logic { 
                        left: Box::new(expr.take().unwrap()), 
                        op: and, 
                        right: Box::new(inner) 
                    });
                }

                last = Some(tmp_var);
            }

            expr.unwrap()
        }
    }
}

impl Transform for UnaryOperation<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let op = {
            match self.operator {
                UnaryOp::Plus { tok } => t.tok_with_type(tok, TokenType::Plus),
                UnaryOp::Minus { tok } => t.tok_with_type(tok, TokenType::Minus),
                UnaryOp::BitInvert { tok } => t.tok_with_type(tok, TokenType::Tilde),
                UnaryOp::Not { tok } => t.tok_with_type(tok, TokenType::Bang),
            }
        };

        Expression::Unary { 
            op, 
            expr: Box::new(self.expression.to_ast(t)), 
            is_prefix: true
        }
    }
}

impl Transform for BinaryOperation<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let op = {
            match self.operator {
                BinaryOp::Add { tok } => t.tok_with_type(tok, TokenType::Plus),
                BinaryOp::Subtract { tok } => t.tok_with_type(tok, TokenType::Minus),
                BinaryOp::Multiply { tok } => t.tok_with_type(tok, TokenType::Star),
                BinaryOp::Modulo { tok } => t.tok_with_type(tok, TokenType::Mod),
                BinaryOp::LeftShift { tok } => t.tok_with_type(tok, TokenType::ShiftLeft),
                BinaryOp::RightShift { tok } => t.tok_with_type(tok, TokenType::ShiftRight),
                BinaryOp::BitOr { tok } => t.tok_with_type(tok, TokenType::BitwiseOr),
                BinaryOp::BitAnd { tok } => t.tok_with_type(tok, TokenType::BitwiseAnd),
                BinaryOp::BitXor { tok } => t.tok_with_type(tok, TokenType::BitwiseXor),
                BinaryOp::Divide { tok } => {
                    return Expression::Binary { 
                        left: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "float") }), 
                            paren: t.tok(tok), 
                            args: vec![self.left.to_ast(t)]
                        }), 
                        op: t.tok_with_type(tok, TokenType::Slash), 
                        right: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "float") }), 
                            paren: t.tok(tok), 
                            args: vec![self.right.to_ast(t)]
                        })
                    };
                }
                BinaryOp::FloorDivide { tok } => {
                    return Expression::Binary { 
                        left: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "int") }), 
                            paren: t.tok(tok), 
                            args: vec![self.left.to_ast(t)]
                        }), 
                        op: t.tok_with_type(tok, TokenType::Slash), 
                        right: Box::new(Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "int") }), 
                            paren: t.tok(tok), 
                            args: vec![self.right.to_ast(t)]
                        })
                    };
                }
                BinaryOp::Power { tok } => {
                    return Expression::Call {
                        callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(tok, "math_pow") }),
                        paren: t.tok(tok),
                        args: vec![self.left.to_ast(t), self.right.to_ast(t)]
                    };
                }
                BinaryOp::MatrixMultiply { tok } => {
                    let tok = t.tok(tok);
                    error!(t.base, tok, "Unsupported operator");
                    tok
                }
            }
        };

        Expression::Binary { 
            left: Box::new(self.left.to_ast(t)), 
            op, 
            right: Box::new(self.right.to_ast(t))
        }
    }
}

impl Transform for BooleanOperation<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let op = {
            match self.operator {
                BooleanOp::And { tok } => t.tok_with_type(tok, TokenType::LogicAnd),
                BooleanOp::Or { tok } => t.tok_with_type(tok, TokenType::LogicOr),
            }
        };

        Expression::Logic { 
            left: Box::new(self.left.to_ast(t)), 
            op, 
            right: Box::new(self.right.to_ast(t))
        }
    }
}

impl Transform for Tuple<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::List { 
            opening_brace: t.tok_from_last_pos(), 
            items: self.elements.iter().map(|x| x.to_ast(t)).collect()
        }
    }
}

impl Transform for List<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::List { 
            opening_brace: t.tok(self.lbracket.tok), 
            items: self.elements.iter().map(|x| x.to_ast(t)).collect()
        }
    }
}

impl Transform for Element<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            Element::Simple { value, .. } => value.to_ast(t),
            Element::Starred(x) => {
                let tok = t.tok(x.star_tok);
                error!(t.base, tok, "Unpacking is not supported");
                make_null()
            }
        }
    }
}

impl Transform for Dict<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::AnonObject { 
            kw: t.tok(self.lbrace.tok), 
            fields: self.elements.iter().map(|x| {
                match x {
                    DictElement::Simple { key, value, colon_tok, .. } => {
                        let key_expr = key.to_ast(t);
                        
                        let name = {
                            if let Expression::Variable { name } = key_expr {
                                name
                            } else if let Expression::Literal { value, mut tok, .. } = key_expr {
                                tok.set_lexeme(&value);
                                tok
                            } else {
                                error!(t.base, key_expr, "Unsupported dictionary key");
                                t.tok(colon_tok)
                            }
                        };

                        ObjectField {
                            name, 
                            expr: value.to_ast(t),
                            type_: Some(Expression::Variable { name: t.tok_with_lexeme(colon_tok, "any") })
                        }
                    }
                    DictElement::Starred(x) => {
                        let tok = t.tok(x.star_tok);
                        error!(t.base, tok, "Unpacking is not supported");
                        
                        ObjectField {
                            name: tok,
                            expr: make_null(),
                            type_: None
                        }
                    }
                }
            }).collect()
        }
    }
}

impl Transform for Call<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let func = self.func.to_ast(t);

        // method calls
        if let Expression::Get { object, name } = &func {
            // handles particularly weird to translate oSV methods
            if let Expression::Subscript { subscripted, index: a, .. } = &**object {
                match name.lexeme.as_ref() {
                    "swap" => {
                        if self.args.len() == 1 {
                            let first_arg = self.args.first().unwrap().to_ast(t);
                            if let Expression::Subscript { index: b, .. } = first_arg {
                                return Expression::Call {
                                    callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(self.lpar_tok, "swap") }),
                                    paren: t.tok(self.lpar_tok),
                                    args: vec![*subscripted.clone(), *a.clone(), *b.clone()],
                                };
                            }
                        }
                    }
                    "write" | "writeRestoreIdx" => {
                        if self.args.len() == 1 || self.args.len() == 2 {
                            return Expression::Assign { 
                                target: object.clone(), 
                                op: t.tok_with_type(self.lpar_tok, TokenType::Walrus), 
                                value: Box::new(self.args.first().unwrap().to_ast(t)), 
                                type_spec: None
                            };
                        }
                    }
                    _ => ()
                }
            }

            match name.lexeme.as_ref() {
                "append" => {
                    if self.args.len() == 1 {
                        return Expression::Call { 
                            callee: Box::new(Expression::Variable { name: t.tok_with_lexeme(self.lpar_tok, "List_push") }), 
                            paren: t.tok(self.lpar_tok), 
                            args: vec![*object.clone(), self.args.first().unwrap().to_ast(t)]
                        };
                    }
                }
                "pop" => {
                    match self.args.len() {
                        0 => {
                            return Expression::Call { 
                                callee: Box::new(Expression::Variable { 
                                    name: t.tok_with_lexeme(self.lpar_tok, "List_pop") 
                                }), 
                                paren: t.tok(self.lpar_tok), 
                                args: vec![*object.clone()]
                            };
                        }
                        1 => {
                            return Expression::Call { 
                                callee: Box::new(Expression::Variable { 
                                    name: t.tok_with_lexeme(self.lpar_tok, "List_removeIdx") 
                                }), 
                                paren: t.tok(self.lpar_tok), 
                                args: vec![*object.clone(), self.args.first().unwrap().to_ast(t)]
                            };
                        }
                        _ => ()
                    }
                }
                "clear" => {
                    if self.args.len() == 0 {
                        return Expression::Call { 
                            callee: Box::new(Expression::Variable { 
                                name: t.tok_with_lexeme(self.lpar_tok, "List_clear") 
                            }), 
                            paren: t.tok(self.lpar_tok), 
                            args: vec![*object.clone()]
                        };
                    }
                }
                _ => ()
            }

            if matches!(&**object, Expression::Variable { .. }) {
                return Expression::Call {
                    callee: Box::new(Expression::Get {
                        object: object.clone(),
                        name: name.clone()
                    }), 
                    paren: t.tok(self.lpar_tok), 
                    args: [*object.clone()].into_iter().chain(self.args.iter().map(|x| x.to_ast(t))).collect()
                }
            } else {
                let op = t.tok_with_type(self.lpar_tok, TokenType::Walrus);
                let t0 = t.base.tmp_var();
                let tmp = Expression::Variable { name: t.tok_with_lexeme(self.lpar_tok, &t0) };

                return Expression::Block { 
                    opening_brace: op.clone(), 
                    expressions: vec![
                        Expression::Assign { 
                            target: Box::new(tmp.clone()), 
                            op: op.clone(), 
                            value: object.clone(), 
                            type_spec: None 
                        },
                        Expression::Call { 
                            callee: Box::new(Expression::Get {
                                object: Box::new(tmp.clone()),
                                name: name.clone()
                            }), 
                            paren: op, 
                            args: [tmp].into_iter().chain(self.args.iter().map(|x| x.to_ast(t))).collect()
                        }
                    ] 
                };
            }
        }

        let mut callee = Box::new(func.clone());
        let mut args = self.args.iter().map(|x| x.to_ast(t)).collect();

        if let Expression::Variable { name } = &func {
            match name.lexeme.as_ref() {
                "int" | "float"  | "abs" => {
                    callee = Box::new(Expression::Variable { 
                        name: t.tok_with_lexeme(self.lpar_tok, format!("Python_{}", name.lexeme).as_str()) 
                    });
                }
                "max" | "min" => {
                    if self.args.len() != 2 {
                        callee = Box::new(Expression::Variable { 
                            name: t.tok_with_lexeme(self.lpar_tok, format!("List_{}", name.lexeme).as_str()) 
                        });

                        args = vec![
                            Expression::List { 
                                opening_brace: t.tok(self.lpar_tok), 
                                items: args 
                            }
                        ];
                    }
                }
                "range" => {
                    callee = Box::new(Expression::Variable { name: t.tok_with_lexeme(self.lpar_tok, {
                        match self.args.len() {
                            1 => "Python_range1",
                            2 => "Python_range2",
                            3 => "Python_range3",
                            _ => "range"
                        }
                    }) });
                }
                "input" => {
                    callee = Box::new(Expression::Variable { name: t.tok_with_lexeme(self.lpar_tok, {
                        match self.args.len() {
                            0 => "Python_input0",
                            1 => "Python_input1",
                            _ => "input"
                        }
                    }) });
                }
                _ => ()
            }
        }

        Expression::Call { 
            paren: t.tok(self.lpar_tok), 
            callee, args
        }
    }
}

impl Transform for Arg<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        if self.keyword.is_some() {
            let tok = t.tok(self.equal.as_ref().unwrap().tok);
            error!(t.base, tok, "Keyword arguments are not supported");
        }

        if let Some(tok) = self.star_tok {
            let tok = t.tok(tok);
            error!(t.base, tok, "Unpacking is not supported");
        }

        self.value.to_ast(t)
    }
}

impl Transform for IfExp<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Ternary { 
            question_tok: t.tok(self.if_tok), 
            condition: Box::new(self.test.to_ast(t)), 
            then_expr: Box::new(self.body.to_ast(t)), 
            else_expr: Box::new(self.orelse.to_ast(t))
        }
    }
}

impl Transform for Lambda<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let lambda_tok = t.tok(self.lambda_tok);

        if t.base.depth != 0 {
            warning!(&lambda_tok, "Closures are not supported. This lambda will be defined in the global scope");
        }

        t.base.inc_depth();
        let body = self.body.to_ast(t);
        t.base.dec_depth();

        Expression::Function { 
            name: t.tok_with_lexeme(self.lambda_tok, "_"), 
            params: params_to_vec_of_named_expr(&self.params, t, &lambda_tok), 
            return_type: Box::new(Expression::Variable { name: t.tok_with_lexeme(self.lambda_tok, "any") }), 
            body: vec![body] 
        }
    }
}

impl Transform for SimpleString<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Literal { 
            value: Rc::from(self.value.trim_matches(|x| x == '"' || x == '\'')), 
            tok: t.tok(self.tok), 
            kind: LiteralKind::String 
        }
    }
}

impl Transform for ConcatenatedString<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        Expression::Binary { 
            left: Box::new(self.left.to_ast(t)), 
            op: t.tok_with_type(self.right_tok, TokenType::Plus), 
            right: Box::new(self.right.to_ast(t))
        }
    }
}

impl Transform for libcst_native::deflated::String<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        match self {
            Self::Simple(x) => x.to_ast(t),
            Self::Concatenated(x) => x.to_ast(t),
            Self::Formatted(_) => {
                let tok = t.tok_from_last_pos();
                error!(t.base, tok, "Formatted strings are not supported");
                make_null()
            }
        }
    }
}

impl Transform for libcst_native::deflated::NamedExpr<'_, '_> {
    type Transformer = ASTTransformer;

    fn to_ast(&self, t: &mut Self::Transformer) -> Expression {
        let target  = self.target.to_ast(t);
        Expression::Assign { 
            type_spec: {
                if let Expression::Subscript { .. } = &target {
                    None
                } else {
                    Some(Box::new(Expression::Variable { name: t.tok_with_lexeme(self.walrus_tok, "any") }))
                }
            },
            target: Box::new(target), 
            op: t.tok_with_type(self.walrus_tok, TokenType::Walrus), 
            value: Box::new(self.value.to_ast(t)), 
        }
    }
}

mod headers {
    use std::{collections::HashMap, rc::Rc};

    use crate::{compiler::type_system::UniLType, utils::lang::push_indent};

    fn stringify_type(type_: &UniLType) -> String {
        match type_ {
            UniLType::Type(inner) => stringify_type(&inner),
            UniLType::Any | UniLType::Object { .. } => String::from("object"),
            UniLType::Null => String::from("None"),
            UniLType::Int => String::from("int"),
            UniLType::Float => String::from("float"),
            UniLType::Value => String::from("Value"),
            UniLType::String => String::from("str"),
            UniLType::List => String::from("list"),
            UniLType::Callable { args, return_type } => {
                let mut buf = String::from("Callable[[");

                for (i, param) in args.iter().enumerate() {
                    buf.push_str(&stringify_type(param));

                    if i + 1 != args.len() {
                        buf.push_str(", ")
                    }
                }

                buf.push_str("], ");
                buf.push_str(&stringify_type(&return_type));
                buf.push(']');
                buf
            }
            UniLType::Group(types) => {
                let mut buf = String::new();

                for (i, type_) in types.iter().enumerate() {
                    buf.push_str(&stringify_type(type_));

                    if i + 1 != types.len() {
                        buf.push_str(" | ")
                    }
                }

                buf
            }
        }
    }

    pub fn make(globals: &HashMap<Rc<str>, UniLType>, buf: &mut String, indent: usize) {
        for (name, type_) in globals {
            match type_ {
                UniLType::Type(inner) => {
                    if matches!(&**inner, UniLType::Null) {
                        continue;
                    }

                    let stringified = stringify_type(inner);

                    if name.as_ref() == stringified.as_str() {
                        continue;
                    }

                    push_indent(buf, indent);
                    buf.push_str(name);
                    buf.push_str(" = ");
                    buf.push_str(&stringified);
                    buf.push('\n');
                }
                UniLType::Object { fields } => {
                    push_indent(buf, indent);
                    buf.push_str("class ");
                    buf.push_str(name);
                    buf.push_str(":\n");
                    make(&fields.borrow(), buf, indent + 1);
                }
                UniLType::Callable { args, return_type } => {
                    match name.as_ref() {
                        "str" | "int" | "float" => continue,
                        _ => ()
                    }

                    push_indent(buf, indent);
                    buf.push_str("def ");
                    buf.push_str(name);
                    buf.push('(');

                    for (i, arg) in args.iter().enumerate() {
                        // if indent is not 0, we are in a class. methods have an extra field that 
                        // is an artifact of transpilation (every call to a method is translated to object.method(object, args...)).
                        // but because to the IDEs the methods will look like they're static when you call them, 
                        // we skip the first one when we're in a class, so the IDE gives proper hints
                        if indent != 0 && i == 0 {
                            continue;
                        }

                        buf.push_str("arg");
                        buf.push_str(&i.to_string());
                        buf.push_str(": ");
                        buf.push_str(&stringify_type(arg));

                        if i + 1 != args.len() {
                            buf.push_str(", ")
                        }
                    }

                    buf.push_str(") -> ");
                    buf.push_str(&stringify_type(return_type));
                    buf.push_str(": ...\n");
                }
                _ => {
                    match name.as_ref() {
                        "True" | "False" | "None" | "__name__" => continue,
                        _ => ()
                    }

                    let stringified = stringify_type(type_);

                    if name.as_ref() == stringified.as_str() {
                        continue;
                    }

                    push_indent(buf, indent);
                    buf.push_str(name);
                    buf.push_str(": ");
                    buf.push_str(&stringified);
                    buf.push('\n');
                }
            }
        }
    }
}

language_layer! {
    language = python;
    extension = "py";

    process(source, filename) {
        use crate::language_layers::python;
        use crate::utils::lang::Transform;
        use crate::unil::ast::Expression;

        let source_cloned = source.clone();

        let mut source = source;
        if let Some(stripped) = source.strip_prefix('\u{feff}') {
            source = stripped.to_string();
        }

        let tokens = {
            match libcst_native::tokenize(&source_cloned) {
                Ok(toks) => toks.into(),
                Err(e) => return Err(vec![e.to_string()])
            }
        };

        let root_node = {
            match libcst_native::parse_tokens_without_whitespace(&tokens, &source_cloned, None) {
                Ok(node) => node,
                Err(e) => return Err(vec![e.to_string()])
            }
        };

        let mut ast_transformer = python::ASTTransformer::new(source, std::rc::Rc::clone(&filename));
        let ast = root_node.to_ast(&mut ast_transformer);

        if !ast_transformer.base.errors.is_empty() {
            return Err(ast_transformer.base.errors);
        }

        if let Expression::Block { expressions, .. } = ast {
            Ok(expressions)
        } else {
            unreachable!()
        }
    }

    generate_headers(globals) {
        let mut declarations = String::from(r#"from typing import Callable, Tuple
from enum import IntEnum

class Value:
    def copy(self) -> "Value": ...
    def noMark(self) -> "Value": ...
    def read(self) -> "Value": ...
    def getInt(self) -> int: ...
    def readInt(self) -> int: ...
    def readNoMark(self) -> Tuple["Value", None]: ...
    def readDigit(self) -> int: ...
    def swap(self, other: "Value") -> "Value": ...
    def write(self, other: "Value" | int) -> "Value": ...
    def writeRestoreIdx(self, other: "Value" | int, idx: None) -> "Value": ...

class RotationMode(IntEnum):
    INDEXED, LENGTHS = range(2)

def Sort(category: str, name: str, listName: str, killers: str | None = None): ...
def Shuffle(name: str): ...
def Distribution(name: str): ...
def PivotSelection(name: str): ...
def Rotation(name: str, mode: RotationMode = RotationMode.INDEXED): ...

"#);
        headers::make(globals, &mut declarations, 0);
        declarations
    }
}