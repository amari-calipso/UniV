use std::rc::Rc;

use crate::{ast_error, token_error, unil::{
    ast::{Expression, NamedExpr, LiteralKind, SwitchCase},
    tokens::{Token, TokenType}
}};

use super::ast::ObjectField;

macro_rules! match_literal {
    ($parser: ident, $type_: tt) => {
        if $parser.match_(&[TokenType::$type_]) {
            let prev = $parser.previous();
            return Some(Expression::Literal {
                value: prev.lexeme.clone(),
                tok: prev.clone(),
                kind: LiteralKind::$type_,
            });
        }
    };
}

macro_rules! left_associativity_binary {
    ($name: tt, $next: tt, $types: expr) => {
        fn $name(&mut self) -> Option<Expression> {
            let mut expr = self.$next()?;

            while self.match_($types) {
                let op = self.previous().clone();
                let right = self.$next()?;
                expr = Expression::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }

            Some(expr)
        }
    };
}

macro_rules! left_associativity_logic {
    ($name: tt, $next: tt, $types: expr) => {
        fn $name(&mut self) -> Option<Expression> {
            let mut expr = self.$next()?;

            while self.match_($types) {
                let op = self.previous().clone();
                let right = self.$next()?;
                expr = Expression::Logic {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }

            Some(expr)
        }
    };
}

macro_rules! left_associativity_cmp {
    ($name: tt, $next: tt, $types: expr) => {
        fn $name(&mut self) -> Option<Expression> {
            let mut expr = self.$next()?;

            while self.match_($types) {
                let op = self.previous().clone();
                let right = self.$next()?;
                expr = Expression::Cmp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            }

            Some(expr)
        }
    };
}

pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
    pub errors: Vec<String>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, curr: 0, errors: Vec::new() }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.curr]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.curr - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::EOF
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.curr += 1;
        }

        self.previous()
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().type_ == type_
    }

    fn match_(&mut self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(*type_) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn syncronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().type_ == TokenType::Semicolon {
                return;
            }

            match self.peek().type_ {
                TokenType::Fn | TokenType::If | TokenType::Return |
                TokenType::Foreach | TokenType::While | TokenType::Do |
                TokenType::Throw | TokenType::Switch |
                TokenType::Continue | TokenType::Break | 
                TokenType::Try => return,
                _ => ()
            }

            self.advance();
        }
    }

    fn consume(&mut self, type_: TokenType, msg: &str) -> Option<&Token> {
        if self.check(type_) {
            Some(self.advance())
        } else {
            let tok = self.peek();
            token_error!(self, tok, msg);
            None
        }
    }

    fn anon_object(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();
        self.consume(TokenType::LeftBrace, "Expecting '{' after '#'")?;

        let mut fields = Vec::new();
        if !self.check(TokenType::RightBrace) {
            loop {
                let name = {
                    if self.match_(&[TokenType::Identifier, TokenType::String]) {
                        self.previous().clone()
                    } else {
                        let tok = self.peek();
                        token_error!(self, tok, "Expecting field name");
                        return None;
                    }
                };
                
                let type_ = {
                    if self.match_(&[TokenType::LeftParen]) {
                        let ret = self.expression()?;
                        self.consume(TokenType::RightParen, "Expecting ')' after field type specification")?;
                        Some(ret)
                    } else {
                        None
                    }
                };
                
                self.consume(TokenType::Colon, "Expecting ':' after field name")?;
                let init = self.expression()?;

                fields.push(ObjectField::new(name, init, type_));

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after anonymous object body")?;
        Some(Expression::AnonObject { kw, fields })
    }

    fn primary(&mut self) -> Option<Expression> {
        if self.match_(&[TokenType::Null]) {
            return Some(Expression::Literal { value: Rc::from(""), tok: self.previous().clone(), kind: LiteralKind::Null });
        }

        if self.match_(&[TokenType::True]) {
            return Some(Expression::Literal { value: Rc::from("1"), tok: self.previous().clone(), kind: LiteralKind::Int });
        }

        if self.match_(&[TokenType::False]) {
            return Some(Expression::Literal { value: Rc::from("0"), tok: self.previous().clone(), kind: LiteralKind::Int });
        }

        if self.match_(&[TokenType::Identifier]) {
            return Some(Expression::Variable { name: self.previous().clone() });
        }

        if self.match_(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expecting ')' after expression")?;
            return Some(Expression::Grouping { inner: Box::new(expr) });
        }

        if self.match_(&[TokenType::LeftSquare]) {
            let opening_brace = self.previous().clone();
            let items = self.get_expressions(TokenType::RightSquare)?;
            self.consume(TokenType::RightSquare, "Expecting ']' after list")?;
            return Some(Expression::List { opening_brace, items });
        }

        if self.match_(&[TokenType::LeftBrace]) {
            return self.get_block();
        }

        if self.match_(&[TokenType::Dollar]) {
            let tok = self.previous().clone();
            return self.scoped_block(tok);
        }

        if self.match_(&[TokenType::Hash]) {
            return self.anon_object();
        }

        match_literal!(self, Int);
        match_literal!(self, Float);
        match_literal!(self, String);

        let last = self.peek();
        token_error!(self, last, "Expecting expression");

        None
    }

    fn get_expressions(&mut self, tok: TokenType) -> Option<Vec<Expression>> {
        let mut expressions = Vec::new();

        if !self.check(tok) {
            loop {
                expressions.push(self.expression()?);

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        Some(expressions)
    }

    fn finish_call(&mut self, callee: Expression) -> Option<Expression> {
        let arguments = self.get_expressions(TokenType::RightParen)?;
        let paren = self.consume(TokenType::RightParen, "Expecting ')' after arguments.")?.clone();
        Some(Expression::Call { callee: Box::new(callee), paren, args: arguments })
    }

    fn finish_subscript(&mut self, subscripted: Expression) -> Option<Expression> {
        let index = self.expression()?;
        let paren = self.consume(TokenType::RightSquare, "Expecting ']' after subscript operation")?.clone();
        Some(Expression::Subscript { subscripted: Box::new(subscripted), paren, index: Box::new(index) })
    }

    fn call(&mut self) -> Option<Expression> {
        let mut expr = self.primary()?;

        loop {
            if self.match_(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_(&[TokenType::LeftSquare]) {
                expr = self.finish_subscript(expr)?;
            } else if self.match_(&[TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expecting property name after '.'")?.clone();
                expr = Expression::Get { object: Box::new(expr), name };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn suffix_unary(&mut self) -> Option<Expression> {
        let expr = self.call()?;

        if self.match_(&[TokenType::PlusPlus, TokenType::MinusMinus]) {
            let op = self.previous().clone();

            if !expr.is_valid_assignment_target() {
                ast_error!(self, expr, "Invalid assignment target");
            }

            return Some(Expression::Unary {
                op,
                expr: Box::new(expr),
                is_prefix: false
            });
        }

        Some(expr)
    }

    fn prefix_unary(&mut self) -> Option<Expression> {
        if self.match_(&[
            TokenType::PlusPlus, TokenType::MinusMinus, TokenType::Minus,
            TokenType::Tilde, TokenType::Bang
        ]) {
            let op = self.previous().clone();
            let right = self.prefix_unary()?;

            if matches!(op.type_, TokenType::PlusPlus | TokenType::MinusMinus) && !right.is_valid_assignment_target() {
                ast_error!(self, right, "Invalid assignment target");
            }

            return Some(Expression::Unary {
                op,
                expr: Box::new(right),
                is_prefix: true
            });
        }

        self.suffix_unary()
    }

    left_associativity_binary!(factor, prefix_unary, &[TokenType::Slash, TokenType::Star, TokenType::Mod]);
    left_associativity_binary!(term, factor, &[TokenType::Minus, TokenType::Plus]);
    left_associativity_binary!(shift, term, &[TokenType::ShiftLeft, TokenType::ShiftRight]);
    left_associativity_cmp!(comparison, shift, &[TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]);
    left_associativity_cmp!(equality, comparison, &[TokenType::BangEqual, TokenType::EqualEqual]);
    left_associativity_binary!(bitwise_and, equality, &[TokenType::BitwiseAnd]);
    left_associativity_binary!(bitwise_xor, bitwise_and, &[TokenType::BitwiseXor]);
    left_associativity_binary!(bitwise_or, bitwise_xor, &[TokenType::BitwiseOr]);
    left_associativity_logic!(logic_and, bitwise_or, &[TokenType::LogicAnd]);
    left_associativity_logic!(logic_or, logic_and, &[TokenType::LogicOr]);

    fn ternary(&mut self) -> Option<Expression> {
        let cond = self.logic_or()?;

        if !self.match_(&[TokenType::Question]) {
            return Some(cond);
        }


        let question = self.previous().clone();
        let then = self.logic_or()?;
        self.consume(TokenType::Colon, "Expecting ':' after 'then' branch of ternary expression")?;
        let else_ = self.logic_or()?;

        Some(Expression::Ternary { question_tok: question, condition: Box::new(cond), then_expr: Box::new(then), else_expr: Box::new(else_) })
    }

    fn assignment(&mut self) -> Option<Expression> {
        let expr = self.ternary()?;

        let type_ = {
            if self.match_(&[TokenType::Colon]) {
                Some(Box::new(self.ternary()?))
            } else {
                None
            }
        };

        if self.match_(&[
            TokenType::Equal, TokenType::PlusEquals, TokenType::MinusEquals,
            TokenType::StarEquals, TokenType::SlashEquals, TokenType::ModEquals,
            TokenType::ShiftLeftEquals, TokenType::ShiftRightEquals, TokenType::AndEquals,
            TokenType::XorEquals, TokenType::OrEquals, TokenType::Walrus
        ]) {
            let op = self.previous().clone();
            let value = self.assignment()?;

            if !expr.is_valid_assignment_target() {
                ast_error!(self, expr, "Invalid assignment target");
            }

            if let Some(type_) = &type_ {
                if !matches!(op.type_, TokenType::Walrus) {
                    ast_error!(self, type_, "Can only use type notation with ':=' operator");
                }
            }

            return Some(Expression::Assign { target: Box::new(expr), op, value: Box::new(value), type_spec: type_ });
        }

        Some(expr)
    }

    fn block(&mut self) -> Option<Vec<Expression>> {
        let mut statements = Vec::new();

        while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after block")?;
        Some(statements)
    }

    fn scoped_block(&mut self, dollar: Token) -> Option<Expression> {
        self.consume(TokenType::LeftBrace, "Expecting '{' after '$' for scoped block")?;

        let mut expressions = Vec::new();

        while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
            expressions.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after scoped block")?;
        Some(Expression::ScopedBlock { dollar, expressions })
    }

    fn get_block(&mut self) -> Option<Expression> {
        let bracket = self.previous().clone();
        Some(Expression::Block { opening_brace: bracket, expressions: self.block()? })
    }

    fn block_or_scoped_block(&mut self, msg: &str) -> Option<Expression> {
        if self.match_(&[TokenType::LeftBrace]) {
            self.get_block()
        } else if self.match_(&[TokenType::Dollar]) {
            let tok = self.previous().clone();
            self.scoped_block(tok)
        } else {
            let tok = self.peek();
            token_error!(self, tok, msg);
            None
        }
    }

    fn if_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let cond = self.expression()?;

        let then_branch = {
            if let Expression::Grouping { .. } = cond {
                self.statement()?
            } else {
                self.block_or_scoped_block("Expecting '{' or '${' after if condition")?
            }
        };

        let else_branch = {
            if self.match_(&[TokenType::Else]) {
                Some(Box::new(self.statement()?))
            } else {
                None
            }
        };

        Some(Expression::If { kw, condition: Box::new(cond), then_branch: Box::new(then_branch), else_branch })
    }

    fn while_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let cond = self.expression()?;

        let body = {
            if let Expression::Grouping { .. } = cond {
                if self.match_(&[TokenType::Semicolon]) {
                    Expression::Block { opening_brace: self.previous().clone(), expressions: Vec::new() }
                } else {
                    self.statement()?
                }
            } else {
                self.block_or_scoped_block("Expecting '{' or '${' after while condition")?
            }
        };

        Some(Expression::While { kw, condition: Box::new(cond), body: Box::new(body), increment: None })
    }

    fn do_while_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let body = self.statement()?;
        self.consume(TokenType::While, "Expecting 'while' after 'do' body")?;

        let cond = self.expression()?;
        self.consume(TokenType::Semicolon, "Expecting ';' after condition")?;

        Some(Expression::DoWhile { kw, condition: Box::new(cond), body: Box::new(body) })
    }

    fn for_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let has_paren = self.match_(&[TokenType::LeftParen]);

        let initializer = {
            if self.match_(&[TokenType::Semicolon]) {
                None
            } else {
                Some(Box::new(self.expression_statement()?))
            }
        };

        let condition = Box::new(
            if self.check(TokenType::Semicolon) {
                Expression::Literal { value: Rc::from("1"), tok: self.previous().clone(), kind: LiteralKind::Int }
            } else {
                self.expression_statement()?
            }
        );

        let increment = {
            if has_paren {
                let r = {
                    if self.check(TokenType::RightParen) {
                        Vec::new()
                    } else {
                        self.get_expressions(TokenType::RightParen)?
                    }
                };

                self.consume(TokenType::RightParen, "Expecting ')' after for increment")?;
                r
            } else if self.check(TokenType::LeftBrace) {
                Vec::new()
            } else {
                self.get_expressions(TokenType::LeftBrace)?
            }
        };

        let mut body = {
            if has_paren {
                if self.match_(&[TokenType::Semicolon]) {
                    Expression::Block { opening_brace: kw.clone(), expressions: Vec::new() }
                } else {
                    self.statement()?
                }
            } else {
                self.block_or_scoped_block("Expecting '{' or '${' after for loop")?
            }
        };

        body = Expression::While {
            kw: kw.clone(),
            condition,
            body: Box::new(body),
            increment: Some(Box::new(Expression::Block { opening_brace: kw.clone(), expressions: increment }))
        };

        if let Some(init) = initializer {
            body = Expression::Block { opening_brace: kw.clone(), expressions: vec![*init, body] };
        }

        Some(body)
    }

    fn foreach_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let has_paren = self.match_(&[TokenType::LeftParen]);

        let variable = self.consume(TokenType::Identifier, "Expecting variable name after 'foreach'")?.clone();
        self.consume(TokenType::Colon, "Expecting ':' after variable name")?;

        let iterator = self.expression()?;

        if has_paren {
            self.consume(TokenType::RightParen, "Expecting ')' after foreach iterator")?;
        }

        let body = {
            if has_paren {
                if self.match_(&[TokenType::Semicolon]) {
                    Expression::Block { opening_brace: kw.clone(), expressions: Vec::new() }
                } else {
                    self.statement()?
                }
            } else {
                self.block_or_scoped_block("Expecting '{' or '${' after foreach loop")?
            }
        };

        Some(Expression::Foreach { kw, variable, iterator: Box::new(iterator), body: Box::new(body) })
    }

    fn return_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let value = {
            if self.check(TokenType::Semicolon) {
                None
            } else {
                Some(Box::new(self.expression()?))
            }
        };

        self.consume(TokenType::Semicolon, "Expecting ';' after return value")?;
        Some(Expression::Return { kw, value })
    }

    fn switch_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();
        let expr = self.expression()?;

        self.consume(TokenType::LeftBrace, "Expecting '{' before switch body")?;
        let mut cases = Vec::new();
        if !self.check(TokenType::RightBrace) {
            loop {
                let expressions = {
                    if self.match_(&[TokenType::Default]) {
                        None
                    } else {
                        let mut exprs = Vec::new();
                        loop {
                            exprs.push(self.expression()?);

                            if !self.match_(&[TokenType::Or]) {
                                break;
                            }
                        }

                        Some(exprs)
                    }
                };

                self.consume(TokenType::LeftBrace, "Expecting '{' after case expression")?;
                cases.push(SwitchCase::new(expressions, self.block()?));

                if self.check(TokenType::RightBrace) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after switch body")?;
        Some(Expression::Switch { kw, expr: Box::new(expr), cases })
    }

    fn break_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();
        let value = {
            if self.check(TokenType::Semicolon) {
                None
            } else {               
                Some(Box::new(self.expression()?))
            }
        };

        self.consume(TokenType::Semicolon, "Expecting ';' after break")?;
        Some(Expression::Break { kw, value })
    }

    fn continue_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();
        let value = {
            if self.check(TokenType::Semicolon) {
                None
            } else {               
                Some(Box::new(self.expression()?))
            }
        };
        
        self.consume(TokenType::Semicolon, "Expecting ';' after continue")?;
        Some(Expression::Continue { kw, value })
    }

    fn expression_statement(&mut self) -> Option<Expression> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expecting ';' after expression")?;
        Some(expr)
    }

    fn throw_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let expr = {
            if self.check(TokenType::Semicolon) {
                None
            } else {
                Some(Box::new(self.expression()?))
            }
        };

        self.consume(TokenType::Semicolon, "Expecting ';' after throw")?;
        Some(Expression::Throw { kw, value: expr })
    }

    fn drop_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();

        let variable = self.consume(TokenType::Identifier, "Expecting variable name after drop")?.clone();
        self.consume(TokenType::Semicolon, "Expecting ';' after drop")?;
        Some(Expression::Drop { kw, variable })
    }

    fn try_statement(&mut self) -> Option<Expression> {
        let kw = self.previous().clone();
        let try_branch = self.declaration()?;

        let catch_branch;
        let catch_var;

        if self.match_(&[TokenType::Catch]) {
            if self.match_(&[TokenType::LeftParen]) {
                catch_var = Some(self.consume(TokenType::Identifier, "Expecting variable name after 'catch ('")?.clone());
            } else {
                catch_var = None;
            }

            catch_branch = Some(Box::new(self.declaration()?));
        } else {
            catch_branch = None;
            catch_var = None;
        }

        Some(Expression::Try { kw, try_branch: Box::new(try_branch), catch_branch, catch_var })
    }

    fn statement(&mut self) -> Option<Expression> {
        if self.match_(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_(&[TokenType::Do]) {
            return self.do_while_statement();
        }

        if self.match_(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.match_(&[TokenType::Foreach]) {
            return self.foreach_statement();
        }

        if self.match_(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.match_(&[TokenType::Switch]) {
            return self.switch_statement();
        }

        if self.match_(&[TokenType::Break]) {
            return self.break_statement();
        }

        if self.match_(&[TokenType::Continue]) {
            return self.continue_statement();
        }

        if self.match_(&[TokenType::LeftBrace]) {
            return self.get_block();
        }

        if self.match_(&[TokenType::Dollar]) {
            let tok = self.previous().clone();
            return self.scoped_block(tok);
        }

        if self.match_(&[TokenType::Throw]) {
            return self.throw_statement();
        }

        if self.match_(&[TokenType::Try]) {
            return self.try_statement();
        }

        if self.match_(&[TokenType::Drop]) {
            return self.drop_statement();
        }

        self.expression_statement()
    }

    fn function(&mut self) -> Option<Expression> {
        let name = self.consume(TokenType::Identifier, "Expecting function name")?.clone();
        self.consume(TokenType::LeftParen, "Expecting '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let p_name = self.consume(TokenType::Identifier, "Expecting parameter name")?.clone();

                let type_ = {
                    if self.match_(&[TokenType::Colon]) {
                        Some(self.expression()?)
                    } else {
                        None
                    }
                };

                params.push(NamedExpr::new(p_name, type_));

                if !self.match_(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let mut closing_paren = self.consume(TokenType::RightParen, "Expecting ')' after function parameters")?.clone();

        let return_type = {
            if self.check(TokenType::LeftBrace) {
                closing_paren.set_type(TokenType::Identifier);
                closing_paren.set_lexeme("any");
                Expression::Variable { name: closing_paren }
            } else {
                self.expression()?
            }
        };

        self.consume(TokenType::LeftBrace, "Expecting '{' after function declaration")?;
        let body = self.block()?;
        Some(Expression::Function { name, params, return_type: Box::new(return_type), body })
    }

    fn algo_decl(&mut self) -> Option<Expression> {
        let name = self.consume(TokenType::Identifier, "Expecting algorithm type after '@'")?.clone();

        let mut fields = Vec::new();
        if self.match_(&[TokenType::LeftBrace]) {
            if !self.check(TokenType::RightBrace) {
                loop {
                    let name = self.consume(TokenType::Identifier, "Expecting field name")?.clone();
                    self.consume(TokenType::Colon, "Expecting ':' after field name")?;
                    let init = self.expression()?;
    
                    fields.push(ObjectField::new(name, init, None));
    
                    if !self.match_(&[TokenType::Comma]) {
                        break;
                    }
                }
            }
    
            self.consume(TokenType::RightBrace, "Expecting '}' after algorithm spec body")?;
        }

        self.consume(TokenType::Fn, "Expecting 'fn' after algorithm declaration")?;
        let function = self.function()?;

        Some(Expression::AlgoDecl {
            name: name.clone(),
            object: Box::new(Expression::AnonObject { kw: name, fields }),
            function: Box::new(function)
        })
    }

    fn declaration(&mut self) -> Option<Expression> {
        if self.match_(&[TokenType::Fn]) {
            return self.function();
        }

        if self.match_(&[TokenType::At]) {
            return self.algo_decl();
        }

        self.statement()
    }

    pub fn expression(&mut self) -> Option<Expression> {
        self.assignment()
    }

    pub fn parse(&mut self) -> Vec<Expression> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(expr) = self.declaration() {
                statements.push(expr);
            } else {
                self.syncronize();
            }
        }

        statements
    }
}