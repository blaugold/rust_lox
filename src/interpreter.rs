use std::{error::Error, fmt};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, Expr, ExprVisitor, ExpressionStmt, GroupingExpr,
        LiteralExpr, PrintStmt, Stmt, StmtVisitor, UnaryExpr, VarStmt, VariableExpr,
    },
    token::{LiteralValue, Token},
};

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {}
    }

    pub fn interpret<'a>(&mut self, statements: &Vec<Stmt<'a>>) -> Result<(), RuntimeError<'a>> {
        for statement in statements {
            self.execute(statement)?;
        }

        Ok(())
    }

    fn execute<'a>(&mut self, stmt: &Stmt<'a>) -> Result<(), RuntimeError<'a>> {
        stmt.accept(self)
    }

    fn evaluate<'a>(&mut self, expr: &Expr<'a>) -> Result<RuntimeValue, RuntimeError<'a>> {
        expr.accept(self)
    }
}

impl<'a> StmtVisitor<'a, Result<(), RuntimeError<'a>>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt<'a>) -> Result<(), RuntimeError<'a>> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt<'a>) -> Result<(), RuntimeError<'a>> {
        todo!()
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt<'a>) -> Result<(), RuntimeError<'a>> {
        todo!()
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt<'a>) -> Result<(), RuntimeError<'a>> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }
}

impl<'a> ExprVisitor<'a, Result<RuntimeValue, RuntimeError<'a>>> for Interpreter {
    fn visit_literal_expr(
        &mut self,
        expr: &LiteralExpr<'a>,
    ) -> Result<RuntimeValue, RuntimeError<'a>> {
        use LiteralValue::*;
        Ok(match expr.value {
            Nil => RuntimeValue::Nil,
            Bool(value) => RuntimeValue::Bool(value),
            Number(value) => RuntimeValue::Number(value),
            String(value) => RuntimeValue::String(value.into()),
        })
    }

    fn visit_variable_expr(
        &mut self,
        expr: &VariableExpr<'a>,
    ) -> Result<RuntimeValue, RuntimeError<'a>> {
        todo!()
    }

    fn visit_assign_expr(
        &mut self,
        expr: &AssignExpr<'a>,
    ) -> Result<RuntimeValue, RuntimeError<'a>> {
        todo!()
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr<'a>) -> Result<RuntimeValue, RuntimeError<'a>> {
        todo!()
    }

    fn visit_binary_expr(
        &mut self,
        expr: &BinaryExpr<'a>,
    ) -> Result<RuntimeValue, RuntimeError<'a>> {
        todo!()
    }

    fn visit_grouping_expr(
        &mut self,
        expr: &GroupingExpr<'a>,
    ) -> Result<RuntimeValue, RuntimeError<'a>> {
        self.evaluate(&expr.expression)
    }
}

enum RuntimeValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Nil => write!(f, "nil"),
            RuntimeValue::Bool(value) => write!(f, "{}", value),
            RuntimeValue::Number(value) => match value.round() == *value {
                // If the value is an integer don't show decimal point.
                true => write!(f, "{:0}", value),
                false => write!(f, "{}", value),
            },
            RuntimeValue::String(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError<'a> {
    pub message: String,
    pub token: &'a Token<'a>,
}

impl<'a> fmt::Display for RuntimeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl<'a> Error for RuntimeError<'a> {}
