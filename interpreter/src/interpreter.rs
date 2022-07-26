use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    fmt, mem,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, CallExpr, ClassStmt, ConditionExpr, Expr, ExprVisitor,
        ExpressionStmt, FunctionStmt, GetExpr, GroupingExpr, IfStmt, LiteralExpr, PrintStmt,
        ReturnStmt, SetExpr, Stmt, StmtVisitor, SuperExpr, ThisExpr, UnaryExpr, VarStmt,
        VariableExpr, VisitExpr, VisitStmt, WhileStmt,
    },
    environment::Environment,
    lox::ErrorCollector,
    token::{LiteralValue, Token, TokenType},
};

pub struct Interpreter {
    error_collector: Rc<RefCell<ErrorCollector>>,
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new(error_collector: Rc<RefCell<ErrorCollector>>) -> Interpreter {
        let mut globals = Environment::new();
        BuiltinFunction::clock().add_to_environment(&mut globals);

        let globals = Rc::new(RefCell::new(globals));

        Interpreter {
            error_collector,
            globals: globals.clone(),
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Rc<Stmt>>) {
        for statement in statements {
            if let Err(early_return) = self.execute(statement) {
                if let EarlyReturn::Error(error) = early_return {
                    self.error_collector.borrow_mut().runtime_error(error);
                    return;
                }
            }
        }
    }

    fn execute(&mut self, stmt: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        stmt.accept(self)
    }

    fn execute_optional(&mut self, stmt: &Option<Rc<Stmt>>) -> Result<(), EarlyReturn> {
        match &stmt {
            Some(stmt) => self.execute(stmt),
            None => Ok(()),
        }
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Rc<Stmt>>,
        environment: &Rc<RefCell<Environment>>,
    ) -> Result<(), EarlyReturn> {
        let enclosing = mem::replace(&mut self.environment, environment.clone());

        let mut result: Result<(), EarlyReturn> = Ok(());

        for statement in statements {
            if let Err(err) = self.execute(statement) {
                result = Err(err);
                break;
            }
        }

        self.environment = enclosing;

        result
    }

    fn evaluate(&mut self, expr: &Rc<Expr>) -> Result<RuntimeValue, EarlyReturn> {
        expr.accept(self)
    }

    fn evaluate_optional(&mut self, expr: &Option<Rc<Expr>>) -> Result<RuntimeValue, EarlyReturn> {
        match expr {
            None => Ok(RuntimeValue::Nil),
            Some(expr) => self.evaluate(expr),
        }
    }

    fn lookup_variable(
        &mut self,
        name: &Token,
        scope_index: &Option<usize>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        if let Some(scope_index) = scope_index {
            Ok(self
                .environment
                .borrow_mut()
                .get_at(&name.lexeme, *scope_index))
        } else {
            self.globals.borrow().get(name)
        }
    }
}

impl StmtVisitor<Result<(), EarlyReturn>> for Interpreter {
    fn visit_expression_stmt(
        &mut self,
        stmt: &ExpressionStmt,
        _: &Rc<Stmt>,
    ) -> Result<(), EarlyReturn> {
        self.evaluate(&stmt.expression).map(|_| ())
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        let environment = Rc::new(RefCell::new(Environment::new_enclosed(&self.environment)));
        self.execute_block(&stmt.statements, &environment)
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        let value = self.evaluate_optional(&stmt.initializer)?;
        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        stmt: &FunctionStmt,
        ptr: &Rc<Stmt>,
    ) -> Result<(), EarlyReturn> {
        let function = RuntimeValue::DeclaredFunction(Rc::new(DeclaredFunction {
            declaration: ptr.clone(),
            closure: self.environment.clone(),
            is_initializer: false,
        }));
        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, function);
        Ok(())
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        self.environment
            .borrow_mut()
            .define(&&stmt.name.lexeme, RuntimeValue::Nil);

        let mut method_environment = self.environment.clone();
        let mut super_class = None;

        if let Some(super_class_expr) = &stmt.super_class {
            match self.evaluate(super_class_expr)? {
                RuntimeValue::Class(class) => {
                    let mut environment = Environment::new_enclosed(&self.environment);
                    environment.define("super", RuntimeValue::Class(class.clone()));
                    method_environment = Rc::new(RefCell::new(environment));
                    super_class = Some(class);
                }
                _ => {
                    return RuntimeError {
                        message: "Super class must be a class".to_string(),
                        token: super_class_expr.as_variable().name.clone(),
                    }
                    .into()
                }
            }
        }

        let mut methods: HashMap<String, Rc<DeclaredFunction>> = HashMap::new();
        for method in &stmt.methods {
            let name = &method.as_function().name.lexeme;
            let function = Rc::new(DeclaredFunction {
                declaration: method.clone(),
                closure: method_environment.clone(),
                is_initializer: name == "init",
            });
            methods.insert(name.clone(), function);
        }

        let class = RuntimeValue::Class(Rc::new(Class {
            name: stmt.name.lexeme.to_string(),
            super_class,
            methods,
        }));

        self.environment.borrow_mut().assign(&stmt.name, class)
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        if self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.then_statement)
        } else {
            self.execute_optional(&stmt.else_statement)
        }
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        while self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(&stmt.body)?;
        }
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt, _: &Rc<Stmt>) -> Result<(), EarlyReturn> {
        self.evaluate_optional(&stmt.value)?.into()
    }
}

impl ExprVisitor<Result<RuntimeValue, EarlyReturn>> for Interpreter {
    fn visit_literal_expr(
        &mut self,
        expr: &LiteralExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        use LiteralValue::*;
        Ok(match &expr.value {
            Nil => RuntimeValue::Nil,
            Bool(value) => RuntimeValue::Bool(*value),
            Number(value) => RuntimeValue::Number(*value),
            String(value) => RuntimeValue::String(Rc::new(value.into())),
        })
    }

    fn visit_variable_expr(
        &mut self,
        expr: &VariableExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        self.lookup_variable(&expr.name, expr.scope_index.get().unwrap())
    }

    fn visit_assign_expr(
        &mut self,
        expr: &AssignExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let value = self.evaluate(&expr.value)?;
        let result = value.clone();

        if let Some(scope_index) = expr.scope_index.get().unwrap() {
            self.environment
                .borrow_mut()
                .assign_at(&expr.name.lexeme, *scope_index, value);
        } else {
            self.globals.borrow_mut().assign(&expr.name, value)?;
        }

        Ok(result)
    }

    fn visit_unary_expr(
        &mut self,
        expr: &UnaryExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
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

    fn visit_binary_expr(
        &mut self,
        expr: &BinaryExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
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

    fn visit_condition_expr(
        &mut self,
        expr: &ConditionExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let left = self.evaluate(&expr.left)?;

        Ok(RuntimeValue::Bool(match expr.operator.token_type {
            TokenType::Or => {
                if left.is_truthy() {
                    true
                } else {
                    self.evaluate(&expr.right)?.is_truthy()
                }
            }
            TokenType::And => {
                if !left.is_truthy() {
                    false
                } else {
                    self.evaluate(&expr.right)?.is_truthy()
                }
            }
            _ => panic!(),
        }))
    }

    fn visit_grouping_expr(
        &mut self,
        expr: &GroupingExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        self.evaluate(&expr.expression)
    }

    fn visit_call_expr(
        &mut self,
        expr: &CallExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let callee = self.evaluate(&expr.callee)?;

        let callable: &dyn Callable = match &callee {
            RuntimeValue::BuiltinFunction(function) => function,
            RuntimeValue::DeclaredFunction(function) => function,
            RuntimeValue::Class(function) => function,
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

    fn visit_get_expr(
        &mut self,
        expr: &GetExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let object = self.evaluate(&expr.object)?;

        match object {
            RuntimeValue::Instance(instance) => instance.get(&expr.name),
            _ => RuntimeError {
                message: "Only instances have properties.".to_string(),
                token: expr.name.clone(),
            }
            .into(),
        }
    }

    fn visit_set_expr(
        &mut self,
        expr: &SetExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let object = self.evaluate(&expr.object)?;

        match object {
            RuntimeValue::Instance(instance) => {
                let value = self.evaluate(&expr.value)?;
                let result = value.clone();
                instance.borrow_mut().set(&expr.name.lexeme, value);
                Ok(result)
            }
            _ => RuntimeError {
                message: "Only instances have properties.".to_string(),
                token: expr.name.clone(),
            }
            .into(),
        }
    }

    fn visit_this_expr(
        &mut self,
        expr: &ThisExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        self.lookup_variable(&expr.token, expr.scope_index.get().unwrap())
    }

    fn visit_super_expr(
        &mut self,
        expr: &SuperExpr,
        _: &Rc<Expr>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let mut environment = self.environment.borrow_mut();
        let scope_index = expr.scope_index.get().unwrap().unwrap();
        let super_class = environment
            .get_at(&expr.keyword.lexeme, scope_index)
            .unwrap_class();

        match super_class.find_method(&expr.method.lexeme) {
            Some(method) => {
                let instance = environment
                    .get_at("this", scope_index - 1)
                    .unwrap_instance();
                Ok(RuntimeValue::DeclaredFunction(method.bind(&instance)))
            }
            None => RuntimeError {
                message: format!("Undefined property '{}'.", expr.method.lexeme),
                token: expr.method.clone(),
            }
            .into(),
        }
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
    Class(Rc<Class>),
    Instance(Rc<RefCell<Instance>>),
}

impl RuntimeValue {
    fn unwrap_class(self) -> Rc<Class> {
        match self {
            RuntimeValue::Class(value) => value,
            _ => panic!(),
        }
    }

    fn unwrap_instance(self) -> Rc<RefCell<Instance>> {
        match self {
            RuntimeValue::Instance(value) => value,
            _ => panic!(),
        }
    }

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
            Class(value) => write!(f, "{}", value),
            Instance(value) => write!(f, "{}", value.borrow()),
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

trait Callable: fmt::Display {
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

impl Callable for Rc<BuiltinFunction> {
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
        environment.define(self.name, RuntimeValue::BuiltinFunction(Rc::new(self)));
    }
}

pub struct DeclaredFunction {
    declaration: Rc<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl DeclaredFunction {
    fn bind(&self, instance: &Rc<RefCell<Instance>>) -> Rc<DeclaredFunction> {
        let mut environment = Environment::new_enclosed(&self.closure);

        environment.define("this", RuntimeValue::Instance(instance.clone()));

        Rc::new(DeclaredFunction {
            declaration: self.declaration.clone(),
            closure: Rc::new(RefCell::new(environment)),
            is_initializer: self.is_initializer,
        })
    }
}

impl Callable for Rc<DeclaredFunction> {
    fn arity(&self) -> u8 {
        self.declaration.as_function().parameters.len() as u8
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let mut environment = Environment::new_enclosed(&self.closure);
        let function = &self.declaration.as_function();

        for (parameter, argument) in function.parameters.iter().zip(arguments) {
            environment.define(&parameter.lexeme, argument);
        }

        if let Err(early_return) =
            interpreter.execute_block(&function.body, &Rc::new(RefCell::new(environment)))
        {
            match early_return {
                EarlyReturn::Return(value) => {
                    return Ok(match self.is_initializer {
                        true => self.closure.borrow_mut().get_at("this", 0),
                        false => value,
                    })
                }
                EarlyReturn::Error(error) => return error.into(),
            }
        }

        Ok(match self.is_initializer {
            true => self.closure.borrow_mut().get_at("this", 0),
            false => RuntimeValue::Nil,
        })
    }
}

impl PartialEq for DeclaredFunction {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl fmt::Display for DeclaredFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fun {}>", self.declaration.as_function().name.lexeme)
    }
}

pub struct Class {
    name: String,
    super_class: Option<Rc<Class>>,
    methods: HashMap<String, Rc<DeclaredFunction>>,
}

impl Class {
    fn find_method(&self, name: &str) -> Option<Rc<DeclaredFunction>> {
        if let x @ Some(_) = self.methods.get(name).map(|method| method.clone()) {
            return x;
        };

        self.super_class
            .as_ref()
            .map(|super_class| super_class.find_method(name))
            .flatten()
    }
}

impl Callable for Rc<Class> {
    fn arity(&self) -> u8 {
        self.find_method("init")
            .map(|init| init.arity())
            .unwrap_or(0)
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<RuntimeValue>,
    ) -> Result<RuntimeValue, EarlyReturn> {
        let instance = Rc::new(RefCell::new(Instance::new(self.clone())));

        if let Some(init) = self.find_method("init") {
            init.bind(&instance).call(interpreter, arguments)?;
        }

        Ok(RuntimeValue::Instance(instance))
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}

pub struct Instance {
    class: Rc<Class>,
    fields: HashMap<String, RuntimeValue>,
}

impl Instance {
    fn new(class: Rc<Class>) -> Instance {
        Instance {
            class,
            fields: HashMap::new(),
        }
    }

    fn set(&mut self, name: &str, value: RuntimeValue) {
        self.fields.insert(name.to_string(), value);
    }
}

trait InstanceGet {
    fn get(&self, name: &Token) -> Result<RuntimeValue, EarlyReturn>;
}

impl InstanceGet for Rc<RefCell<Instance>> {
    fn get(&self, name: &Token) -> Result<RuntimeValue, EarlyReturn> {
        if let Some(value) = self.borrow().fields.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(method) = self.borrow().class.find_method(&name.lexeme) {
            return Ok(RuntimeValue::DeclaredFunction(method.bind(self)));
        }

        RuntimeError {
            message: format!("Undefined property '{}'.", name.lexeme),
            token: name.clone(),
        }
        .into()
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} instance>", self.class.name)
    }
}
