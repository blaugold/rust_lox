use std::{
    error::Error,
    fmt::{self},
    mem,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, CallExpr, Expr, ExprVisitor, ExpressionStmt,
        FunctionStmt, GroupingExpr, IfStmt, LiteralExpr, PrintStmt, ReturnStmt, Stmt, StmtVisitor,
        UnaryExpr, VarStmt, VariableExpr, WhileStmt,
    },
    environment::Environment,
    token::{LiteralValue, Token, TokenType},
};

pub struct Interpreter {
    environment: Box<Environment>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = Box::new(Environment::new());
        BuiltinFunction::clock().add_to_environment(&mut globals);

        Interpreter {
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), RuntimeError> {
        for statement in statements {
            if let Err(early_return) = self.execute(statement) {
                match early_return {
                    EarlyReturn::Return(_) => {
                        // We discard the value of the return statement at the top level.
                        return Ok(());
                    }
                    EarlyReturn::Error(error) => {
                        return Err(error);
                    }
                }
            }
        }

        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), EarlyReturn> {
        stmt.accept(self)
    }

    fn execute_optional(&mut self, stmt: &Option<Stmt>) -> Result<(), EarlyReturn> {
        match &stmt {
            Some(stmt) => self.execute(stmt),
            None => Ok(()),
        }
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Box<Environment>,
    ) -> Result<(), EarlyReturn> {
        let enclosing = mem::replace(&mut self.environment, environment);
        self.environment.set_enclosing(enclosing);

        let mut result: Result<(), EarlyReturn> = Ok(());

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

    fn evaluate(&mut self, expr: &Expr) -> Result<RuntimeValue, EarlyReturn> {
        expr.accept(self)
    }

    fn evaluate_optional(&mut self, expr: &Option<Expr>) -> Result<RuntimeValue, EarlyReturn> {
        match expr {
            None => Ok(RuntimeValue::Nil),
            Some(expr) => self.evaluate(expr),
        }
    }
}

impl StmtVisitor<Result<(), EarlyReturn>> for Interpreter {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> Result<(), EarlyReturn> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Result<(), EarlyReturn> {
        let environment = Box::new(Environment::new());
        self.execute_block(&stmt.statements, environment)
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> Result<(), EarlyReturn> {
        let value = self.evaluate_optional(&stmt.initializer)?;
        self.environment.define(&stmt.name.lexeme, value)
    }

    fn visit_function_stmt(&mut self, stmt: &Rc<FunctionStmt>) -> Result<(), EarlyReturn> {
        let function = RuntimeValue::DeclaredFunction(Rc::new(DeclaredFunction {
            declaration: stmt.clone(),
        }));
        self.environment.define(&stmt.name.lexeme, function)
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> Result<(), EarlyReturn> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Result<(), EarlyReturn> {
        if self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.then_statement)
        } else {
            self.execute_optional(&stmt.else_statement)
        }
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Result<(), EarlyReturn> {
        while self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.body)?;
        }
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Result<(), EarlyReturn> {
        self.evaluate_optional(&stmt.value)?.into()
    }
}

impl ExprVisitor<Result<RuntimeValue, EarlyReturn>> for Interpreter {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Result<RuntimeValue, EarlyReturn> {
        use LiteralValue::*;
        Ok(match &expr.value {
            Nil => RuntimeValue::Nil,
            Bool(value) => RuntimeValue::Bool(*value),
            Number(value) => RuntimeValue::Number(*value),
            String(value) => RuntimeValue::String(Rc::new(value.into())),
        })
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Result<RuntimeValue, EarlyReturn> {
        self.environment.get(&expr.name)
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Result<RuntimeValue, EarlyReturn> {
        let value = self.evaluate(&expr.value)?;
        let result = value.clone();
        self.environment.assign(&expr.name, value)?;
        Ok(result)
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Result<RuntimeValue, EarlyReturn> {
        let operand = self.evaluate(&expr.expression)?;
        Ok(match expr.operator.token_type {
            TokenType::Bang => RuntimeValue::Bool(!operand.is_truthy()),
            TokenType::Minus => {
                let operand = check_numeric_operand(&expr.operator, &operand)?;
                RuntimeValue::Number(-operand)
            }
            _ => panic!(),
        })
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Result<RuntimeValue, EarlyReturn> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        Ok(match expr.operator.token_type {
            TokenType::Plus => {
                let result = match left {
                    RuntimeValue::Number(left) => match right {
                        RuntimeValue::Number(right) => Some(RuntimeValue::Number(left + right)),
                        _ => None,
                    },
                    RuntimeValue::String(left) => match right {
                        RuntimeValue::String(right) => {
                            Some(RuntimeValue::String(Rc::new(format!("{}{}", left, right))))
                        }
                        _ => None,
                    },
                    _ => None,
                };

                match result {
                    Some(result) => result,
                    None => {
                        return RuntimeError {
                            message: format!(
                                "Operands must either both be numbers or both be strings."
                            ),
                            token: expr.operator.clone(),
                        }
                        .into();
                    }
                }
            }
            TokenType::Minus => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Number(left - right)
            }
            TokenType::Slash => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Number(left / right)
            }
            TokenType::Star => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Number(left * right)
            }
            TokenType::EqualEqual => RuntimeValue::Bool(left == right),
            TokenType::BangEqual => RuntimeValue::Bool(left != right),
            TokenType::Less => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Bool(left < right)
            }
            TokenType::LessEqual => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Bool(left <= right)
            }
            TokenType::Greater => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Bool(left > right)
            }
            TokenType::GreaterEqual => {
                let (left, right) = check_numeric_operands(&expr.operator, &left, &right)?;
                RuntimeValue::Bool(left >= right)
            }
            _ => panic!(),
        })
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Result<RuntimeValue, EarlyReturn> {
        self.evaluate(&expr.expression)
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Result<RuntimeValue, EarlyReturn> {
        let callee = self.evaluate(&expr.callee)?;

        let callable: &dyn LoxCallable = match &callee {
            RuntimeValue::BuiltinFunction(function) => &**function,
            RuntimeValue::DeclaredFunction(function) => &**function,
            _ => {
                return RuntimeError {
                    message: "Can only call functions and classes.".to_string(),
                    token: expr.paren.clone(),
                }
                .into();
            }
        };

        if expr.arguments.len() != callable.arity() as usize {
            return RuntimeError {
                message: format!(
                    "Expected {} arguments but got {}.",
                    callable.arity(),
                    expr.arguments.len()
                ),
                token: expr.paren.clone(),
            }
            .into();
        };

        let mut arguments = vec![];
        for argument in &expr.arguments {
            arguments.push(self.evaluate(argument)?);
        }

        callable.call(self, arguments)
    }
}

pub enum EarlyReturn {
    Return(RuntimeValue),
    Error(RuntimeError),
}

impl Error for EarlyReturn {}

impl fmt::Debug for EarlyReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EarlyReturn")
    }
}

impl fmt::Display for EarlyReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EarlyReturn")
    }
}

impl<T> Into<Result<T, EarlyReturn>> for RuntimeValue {
    fn into(self) -> Result<T, EarlyReturn> {
        Err(EarlyReturn::Return(self))
    }
}

impl<T> Into<Result<T, EarlyReturn>> for RuntimeError {
    fn into(self) -> Result<T, EarlyReturn> {
        Err(EarlyReturn::Error(self))
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub token: Token,
}

impl Error for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(PartialEq, Clone)]
pub enum RuntimeValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(Rc<String>),
    BuiltinFunction(Rc<BuiltinFunction>),
    DeclaredFunction(Rc<DeclaredFunction>),
}

impl RuntimeValue {
    fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(value) => *value,
            _ => false,
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuntimeValue::*;
        match self {
            Nil => write!(f, "nil"),
            Bool(value) => write!(f, "{}", value),
            Number(value) => match value.round() == *value {
                // If the value is an integer don't show decimal point.
                true => write!(f, "{:0}", value),
                false => write!(f, "{}", value),
            },
            String(value) => write!(f, "{}", value),
            BuiltinFunction(value) => write!(f, "{}", value),
            DeclaredFunction(value) => write!(f, "{}", value),
        }
    }
}

fn check_numeric_operand(operator: &Token, operand: &RuntimeValue) -> Result<f64, EarlyReturn> {
    if let RuntimeValue::Number(value) = *operand {
        return Ok(value);
    }

    RuntimeError {
        message: format!("Operand must be a number."),
        token: operator.clone(),
    }
    .into()
}

fn check_numeric_operands(
    operator: &Token,
    left_operand: &RuntimeValue,
    right_operand: &RuntimeValue,
) -> Result<(f64, f64), EarlyReturn> {
    if let RuntimeValue::Number(left_value) = *left_operand {
        if let RuntimeValue::Number(right_value) = *right_operand {
            return Ok((left_value, right_value));
        }
    }

    RuntimeError {
        message: format!("Operands must both be numbers."),
        token: operator.clone(),
    }
    .into()
}

trait LoxCallable: fmt::Display {
    fn arity(&self) -> u8;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, EarlyReturn>;
}

pub struct BuiltinFunction {
    name: &'static str,
    arity: u8,
    function: fn(arguments: Vec<RuntimeValue>) -> RuntimeValue,
}

impl LoxCallable for BuiltinFunction {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(
        &self,
        _: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        Ok((self.function)(arguments))
    }
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fun {}>", self.name)
    }
}

impl BuiltinFunction {
    fn clock() -> BuiltinFunction {
        BuiltinFunction {
            name: "clock",
            arity: 0,
            function: |_| {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as f64
                    / 1000.0;
                RuntimeValue::Number(now)
            },
        }
    }

    fn add_to_environment(self, environment: &mut Environment) {
        environment
            .define(self.name, RuntimeValue::BuiltinFunction(Rc::new(self)))
            .unwrap();
    }
}

pub struct DeclaredFunction {
    declaration: Rc<FunctionStmt>,
}

impl LoxCallable for DeclaredFunction {
    fn arity(&self) -> u8 {
        self.declaration.parameters.len() as u8
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let mut environment = Box::new(Environment::new());

        for (parameter, argument) in self.declaration.parameters.iter().zip(arguments) {
            environment.define(&parameter.lexeme, argument)?;
        }

        if let Err(early_return) = interpreter.execute_block(&self.declaration.body, environment) {
            match early_return {
                EarlyReturn::Return(value) => return Ok(value),
                EarlyReturn::Error(error) => return error.into(),
            }
        }

        Ok(RuntimeValue::Nil)
    }
}

impl PartialEq for DeclaredFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl fmt::Display for DeclaredFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fun {}>", self.declaration.name.lexeme)
    }
}
