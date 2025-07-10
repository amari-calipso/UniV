use crate::{unil::{ast::Expression, tokens::TokenType}, utils::lang::make_null};

async fn process_multiple(expressions: &mut Vec<Expression>, ctx: &mut reblessive::Stk) {
    let mut i = 0;
    while i < expressions.len() {
        ctx.run(|ctx| process_internal(&mut expressions[i], ctx)).await;

        if i + 2 >= expressions.len() {
            i += 1;
            continue;
        }

        // expression[0] is target0 assign value0
        if let Expression::Assign { target: target0, op: op0, value: value0, type_spec } = expressions[i].get_inner() {
            // expression[0] is target0 = value0 (or target0 := value0)
            if !matches!(op0.type_, TokenType::Equal | TokenType::Walrus) {
                i += 1;
                continue;
            }

            // expression[0] is target0 = subscripted0[index0]
            if let Expression::Subscript { subscripted: subscripted0, index: index0, .. } = &*value0 {
                // expression[1] is target1 assign value1
                if let Expression::Assign { target: target1, op: op1, value: value1, .. } = expressions[i + 1].get_inner() {
                    // expression[1] is subscripted0[index0] = value1
                    if !matches!(op1.type_, TokenType::Equal | TokenType::Walrus) || !target1.equals(&value0) {
                        i += 1; 
                        continue;
                    }
                    
                    // expression[1] is subscripted0[index0] = subscripted1[index1]
                    if let Expression::Subscript { subscripted: subscripted1, index: index1, .. } = &*value1 {
                        // expression[1] is subscripted0[index0] = subscripted0[index1]
                        if !subscripted0.equals(&subscripted1) {
                            i += 1;
                            continue;
                        }
                        
                        // expression[2] is target2 assign value2
                        if let Expression::Assign { target: target2, op: op2, value: value2, .. } = expressions[i + 2].get_inner() {
                            // expression[2] is subscripted0[index1] = target0 -> swap is found
                            if !matches!(op2.type_, TokenType::Equal | TokenType::Walrus) || !target2.equals(&value1) || !value2.equals(&target0) {
                                i += 1;
                                continue;
                            }

                            // remove first expression
                            expressions[i] = make_null();

                            // replace second expression with code that keeps side effects
                            expressions[i + 1] = Expression::Block { 
                                opening_brace: op1, 
                                expressions: vec![
                                    // subscript operation would normally be evaluated 4 times, so clone it to keep side effects (if any)
                                    *subscripted0.clone(),
                                    *subscripted0.clone(),
                                    *subscripted0.clone(),
                                    // left and right indices would be evaluated 1 additional time each, so do the same
                                    *index0.clone(),
                                    *index1.clone()
                                ] 
                            };

                            // replace third expression with output call (with assignment, to keep variable validity)
                            expressions[i + 2] = Expression::Assign { 
                                target: target0, op: op0, type_spec,
                                value: Box::new(Expression::Call { 
                                    callee: Box::new(Expression::Variable { 
                                        name: {
                                            let mut name = op2.clone();
                                            name.set_lexeme("swap");
                                            name
                                        }
                                    }),
                                    paren: op2, 
                                    args: vec![*subscripted0.clone(), *index0.clone(), *index1.clone()]
                                })
                            };

                            i += 2;
                        }
                    }
                }
            }
        }

        i += 1;
    }
}

async fn process_internal(expression: &mut Expression, ctx: &mut reblessive::Stk) {
    match expression {
        Expression::Grouping { inner } |
        Expression::Unary { expr: inner, .. } |
        Expression::Get { object: inner, .. } => {
            ctx.run(|ctx| process_internal(inner, ctx)).await;
        }
        Expression::List { items: expressions, .. } |
        Expression::Block { expressions, .. } |
        Expression::ScopedBlock { expressions, .. } => {
            ctx.run(|ctx| process_multiple(expressions, ctx)).await;
        }
        Expression::Binary { left, right, .. } |
        Expression::Cmp { left, right, .. } |
        Expression::Logic { left, right, .. } | 
        Expression::Assign { target: left, value: right, .. } |
        Expression::Subscript { subscripted: left, index: right, .. } |
        Expression::DoWhile { condition: left, body: right, .. } |
        Expression::Foreach { iterator: left, body: right, .. } |
        Expression::AlgoDecl { object: left, function: right, .. } => {
            ctx.run(|ctx| process_internal(left, ctx)).await;
            ctx.run(|ctx| process_internal(right, ctx)).await;
        }
        Expression::Call { callee, args, .. } => {
            ctx.run(|ctx| process_internal(callee, ctx)).await;
            ctx.run(|ctx| process_multiple(args, ctx)).await;
        }
        Expression::Break { value, .. } |
        Expression::Continue { value, .. } |
        Expression::Return { value, .. } |
        Expression::Throw { value, .. } => {
            if let Some(value) = value {
                ctx.run(|ctx| process_internal(value, ctx)).await;
            }
        }
        Expression::Ternary { condition, then_expr, else_expr, .. } => {
            ctx.run(|ctx| process_internal(condition, ctx)).await;
            ctx.run(|ctx| process_internal(then_expr, ctx)).await;
            ctx.run(|ctx| process_internal(else_expr, ctx)).await;
        }
        Expression::Try { try_branch, catch_branch, .. } => {
            ctx.run(|ctx| process_internal(try_branch, ctx)).await;

            if let Some(catch_branch) = catch_branch {
                ctx.run(|ctx| process_internal(catch_branch, ctx)).await;
            }
        }
        Expression::If { condition, then_branch: body, else_branch: opt, .. } |
        Expression::While { condition, body, increment: opt, .. } => {
            ctx.run(|ctx| process_internal(condition, ctx)).await;
            ctx.run(|ctx| process_internal(body, ctx)).await;

            if let Some(opt) = opt {
                ctx.run(|ctx| process_internal(opt, ctx)).await;
            }
        }
        Expression::Function { params, return_type, body, .. } => {
            ctx.run(|ctx| process_internal(return_type, ctx)).await;
            ctx.run(|ctx| process_multiple(body, ctx)).await;

            for param in params {
                if let Some(expr) = &mut param.expr {
                    ctx.run(|ctx| process_internal(expr, ctx)).await;
                }
            }
        }
        Expression::AnonObject { fields, .. } => {
            for field in fields {
                ctx.run(|ctx| process_internal(&mut field.expr, ctx)).await;

                if let Some(type_) = &mut field.type_ {
                    ctx.run(|ctx| process_internal(type_, ctx)).await;
                }
            }
        }
        Expression::Switch { expr, cases, .. } => {
            ctx.run(|ctx| process_internal(expr, ctx)).await;

            for branch in cases {
                ctx.run(|ctx| process_multiple(&mut branch.code, ctx)).await;

                if let Some(cases) = &mut branch.cases {
                    ctx.run(|ctx| process_multiple(cases, ctx)).await;
                }
            }
        }
        _ => ()
    }
}

pub fn process(expressions: &mut Vec<Expression>) {
    let mut stack = reblessive::Stack::new();
    stack.enter(|ctx| process_multiple(expressions, ctx)).finish();
}