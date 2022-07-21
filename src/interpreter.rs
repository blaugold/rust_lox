use std::{error::Error, fmt, mem, rc::Rc};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, Expr, ExprVisitor, ExpressionStmt, GroupingExpr, IfStmt,
        LiteralExpr, PrintStmt, Stmt, StmtVisitor, UnaryExpr, VarStmt, VariableExpr,
    },
    environment::Environment,
    token::{LiteralValue, Token, TokenType},
};

pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Box::new(Environment::new()),
        }
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

    fn execute_optional<'a>(&mut self, stmt: &Option<Stmt<'a>>) -> Result<(), RuntimeError<'a>> {
        match &stmt {
            Some(stmt) => self.execute(stmt),
            None => Ok(()),
        }
    }

    fn execute_block<'a>(&mut self, statements: &Vec<Stmt<'a>>) -> Result<(), RuntimeError<'a>> {
        let environment = Box::new(Environment::new());
        let enclosing = mem::replace(&mut self.environment, environment);
        self.environment.set_enclosing(enclosing);

        let mut result: Result<(), RuntimeError<'a>> = Ok(());

        for statement in statements {
            match self.execute(statement) {
                Err(err) => {
                    result = Err(err);
                    break;
                }
                Ok(_) => {}
            }
        }

        self.environment = self.environment.take_enclosing();

        result
    }

    fn evaluate<'a>(&mut self, expr: &Expr<'a>) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        expr.accept(self)
    }

    fn evaluate_optional<'a>(
        &mut self,
        expr: &Option<Expr<'a>>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        match expr {
            None => Ok(Rc::new(RuntimeValue::Nil)),
            Some(expr) => self.evaluate(expr),
        }
    }
}

impl<'a> StmtVisitor<'a, Result<(), RuntimeError<'a>>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt<'a>) -> Result<(), RuntimeError<'a>> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt<'a>) -> Result<(), RuntimeError<'a>> {
        self.execute_block(&stmt.statements)
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt<'a>) -> Result<(), RuntimeError<'a>> {
        let value = self.evaluate_optional(&stmt.initializer)?;
        self.environment.define(stmt.name.lexeme, value)
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt<'a>) -> Result<(), RuntimeError<'a>> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt<'a>) -> Result<(), RuntimeError<'a>> {
        if self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.then_statement)
        } else {
            self.execute_optional(&stmt.else_statement)
        }
    }
}

impl<'a> ExprVisitor<'a, Result<Rc<RuntimeValue>, RuntimeError<'a>>> for Interpreter {
    fn visit_literal_expr(
        &mut self,
        expr: &LiteralExpr<'a>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        use LiteralValue::*;
        Ok(Rc::new(match expr.value {
            Nil => RuntimeValue::Nil,
            Bool(value) => RuntimeValue::Bool(value),
            Number(value) => RuntimeValue::Number(value),
            String(value) => RuntimeValue::String(value.into()),
        }))
    }

    fn visit_variable_expr(
        &mut self,
        expr: &VariableExpr<'a>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        self.environment.get(expr.name)
    }

    fn visit_assign_expr(
        &mut self,
        expr: &AssignExpr<'a>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        let value = self.evaluate(&expr.value)?;
        let result = value.clone();
        self.environment.assign(expr.name, value)?;
        Ok(result)
    }

    fn visit_unary_expr(
        &mut self,
        expr: &UnaryExpr<'a>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        let operand = self.evaluate(&expr.expression)?;
        Ok(Rc::new(match expr.operator.token_type {
            TokenType::Bang => RuntimeValue::Bool(!operand.is_truthy()),
            TokenType::Minus => {
                let operand = check_numeric_operand(expr.operator, &operand)?;
                RuntimeValue::Number(-operand)
            }
            _ => panic!(),
        }))
    }

    fn visit_binary_expr(
        &mut self,
        expr: &BinaryExpr<'a>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        Ok(Rc::new(match expr.operator.token_type {
            TokenType::Plus => {
                let result = match &*left {
                    RuntimeValue::Number(left) => match &*right {
                        RuntimeValue::Number(right) => Some(RuntimeValue::Number(left + right)),
                        _ => None,
                    },
                    RuntimeValue::String(left) => match &*right {
                        RuntimeValue::String(right) => {
                            Some(RuntimeValue::String(format!("{}{}", left, right)))
                        }
                        _ => None,
                    },
                    _ => None,
                };

                match result {
                    Some(result) => result,
                    None => {
                        return Err(RuntimeError {
                            message: format!(
                                "Operands must either both be numbers or both be strings."
                            ),
                            token: &expr.operator,
                        })
                    }
                }
            }
            TokenType::Minus => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Number(left + right)
            }
            TokenType::Slash => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Number(left / right)
            }
            TokenType::Star => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Number(left * right)
            }
            TokenType::EqualEqual => RuntimeValue::Bool(left == right),
            TokenType::BangEqual => RuntimeValue::Bool(left != right),
            TokenType::Less => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Bool(left < right)
            }
            TokenType::LessEqual => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Bool(left <= right)
            }
            TokenType::Greater => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Bool(left > right)
            }
            TokenType::GreaterEqual => {
                let (left, right) = check_numeric_operands(expr.operator, &left, &right)?;
                RuntimeValue::Bool(left >= right)
            }
            _ => panic!(),
        }))
    }

    fn visit_grouping_expr(
        &mut self,
        expr: &GroupingExpr<'a>,
    ) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        self.evaluate(&expr.expression)
    }
}

#[derive(PartialEq)]
pub enum RuntimeValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

impl RuntimeValue {
    fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(value) => *value,
            _ => false,
        }
    }
}

fn check_numeric_operand<'a>(
    operator: &'a Token<'a>,
    operand: &RuntimeValue,
) -> Result<f64, RuntimeError<'a>> {
    match *operand {
        RuntimeValue::Number(value) => return Ok(value),
        _ => {}
    }

    Err(RuntimeError {
        message: format!("Operand must be a number."),
        token: operator,
    })
}

fn check_numeric_operands<'a>(
    operator: &'a Token<'a>,
    left_operand: &RuntimeValue,
    right_operand: &RuntimeValue,
) -> Result<(f64, f64), RuntimeError<'a>> {
    match *left_operand {
        RuntimeValue::Number(left_value) => match *right_operand {
            RuntimeValue::Number(right_value) => return Ok((left_value, right_value)),
            _ => {}
        },
        _ => {}
    }

    Err(RuntimeError {
        message: format!("Operands must both be numbers."),
        token: operator,
    })
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
