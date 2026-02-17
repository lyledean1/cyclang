use parser::Expression;
use parser::Type;

pub fn desugar_program(exprs: Vec<Expression>) -> Vec<Expression> {
    exprs.into_iter().map(desugar_expr).collect()
}

fn desugar_expr(expr: Expression) -> Expression {
    match expr {
        Expression::ForStmt(var, start, end, step, body) => {
            let init = Expression::LetStmt(
                var.clone(),
                Type::i32,
                Box::new(Expression::Number(start)),
            );

            let cond = Expression::Binary(
                Box::new(Expression::Variable(var.clone())),
                if step >= 0 { "<".to_string() } else { ">".to_string() },
                Box::new(Expression::Number(end)),
            );

            let incr = Expression::LetStmt(
                var.clone(),
                Type::None,
                Box::new(Expression::Binary(
                    Box::new(Expression::Variable(var.clone())),
                    if step >= 0 { "+".to_string() } else { "-".to_string() },
                    Box::new(Expression::Number(step.abs())),
                )),
            );

            let body = match *body {
                Expression::BlockStmt(mut stmts) => {
                    stmts.push(incr);
                    Expression::BlockStmt(stmts)
                }
                other => Expression::BlockStmt(vec![other, incr]),
            };

            Expression::BlockStmt(vec![
                init,
                Expression::WhileStmt(Box::new(cond), Box::new(body)),
            ])
        }
        Expression::BlockStmt(stmts) => {
            Expression::BlockStmt(stmts.into_iter().map(desugar_expr).collect())
        }
        Expression::IfStmt(cond, then_branch, else_branch) => {
            let else_branch = *else_branch;
            Expression::IfStmt(
                Box::new(desugar_expr(*cond)),
                Box::new(desugar_expr(*then_branch)),
                Box::new(else_branch.map(desugar_expr)),
            )
        }
        Expression::WhileStmt(cond, body) => {
            Expression::WhileStmt(Box::new(desugar_expr(*cond)), Box::new(desugar_expr(*body)))
        }
        Expression::BreakStmt => Expression::BreakStmt,
        Expression::FuncStmt(name, args, return_type, body) => Expression::FuncStmt(
            name,
            args,
            return_type,
            Box::new(desugar_expr(*body)),
        ),
        other => other,
    }
}
