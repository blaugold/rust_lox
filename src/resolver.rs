use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, CallExpr, ClassStmt, ConditionExpr, Expr, ExprVisitor,
        ExpressionStmt, FunctionStmt, GetExpr, GroupingExpr, IfStmt, LiteralExpr, PrintStmt,
        ReturnStmt, SetExpr, Stmt, StmtVisitor, ThisExpr, UnaryExpr, VarStmt, VariableExpr,
        WhileStmt,
    },
    interpreter::Interpreter,
    lox::ErrorCollector,
    token::Token,
};

#[derive(PartialEq, Eq, Copy, Clone)]
enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver {
    error_collector: Rc<RefCell<ErrorCollector>>,
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    function_type: FunctionType,
    class_type: ClassType,
}

impl Resolver {
    pub fn new(
        error_collector: Rc<RefCell<ErrorCollector>>,
        interpreter: Rc<RefCell<Interpreter>>,
    ) -> Resolver {
        Resolver {
            error_collector,
            interpreter,
            scopes: vec![],
            function_type: FunctionType::None,
            class_type: ClassType::None,
        }
    }

    pub fn resolve(mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, statement: &Stmt) {
        statement.accept(self);
    }

    fn resolve_stmt_vec(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) {
        expression.accept(self);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                self.error_collector
                    .borrow_mut()
                    .resolver_error(name, "Already a variable with this name in this scope.")
            }

            scope.insert(name.lexeme.to_string(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.to_string(), true);
        }
    }

    fn resolve_function(&mut self, stmt: &FunctionStmt, function_type: FunctionType) {
        let outer_function_type = self.function_type;
        self.function_type = function_type;

        self.begin_scope();

        for parameter in &stmt.parameters {
            self.declare(parameter);
            self.define(parameter);
        }

        self.resolve_stmt_vec(&stmt.body);

        self.end_scope();

        self.function_type = outer_function_type;
    }

    fn resolve_local(&mut self, name: &Token, expr: Expr) {
        for (scope_index, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .borrow_mut()
                    .resolve_local(expr, scope_index);
                return;
            }
        }
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> () {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> () {
        self.begin_scope();
        self.resolve_stmt_vec(&stmt.statements);
        self.end_scope();
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> () {
        self.declare(&stmt.name);

        if let Some(initializer) = &stmt.initializer {
            self.resolve_expr(initializer);
        }

        self.define(&stmt.name);
    }

    fn visit_function_stmt(&mut self, stmt: &Rc<FunctionStmt>) -> () {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function);
    }

    fn visit_class_stmt(&mut self, stmt: &Rc<ClassStmt>) -> () {
        let outer_class_type = self.class_type;
        self.class_type = ClassType::Class;

        self.declare(&stmt.name);
        self.define(&stmt.name);

        for method in &stmt.methods {
            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("this".to_string(), true);

            let declaration = FunctionType::Method;
            self.resolve_function(method, declaration);

            self.end_scope();
        }

        self.class_type = outer_class_type;
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> () {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> () {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.then_statement);
        if let Some(else_statement) = &stmt.else_statement {
            self.resolve_stmt(else_statement);
        }
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> () {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.body);
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> () {
        if self.function_type == FunctionType::None {
            self.error_collector
                .borrow_mut()
                .resolver_error(&stmt.token, "Can't return from top level code.");
        }

        if let Some(value) = &stmt.value {
            self.resolve_expr(value);
        }
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_literal_expr(&mut self, _: &LiteralExpr) -> () {}

    fn visit_variable_expr(&mut self, expr: &Rc<VariableExpr>) -> () {
        if let Some(scope) = self.scopes.last() {
            if let Some(defined) = scope.get(&expr.name.lexeme) {
                if !defined {
                    self.error_collector.borrow_mut().resolver_error(
                        &expr.name,
                        "Can't read local variable in it's own initializer.",
                    );
                }
            }
        }

        self.resolve_local(&expr.name, Expr::Variable(expr.clone()));
    }

    fn visit_assign_expr(&mut self, expr: &Rc<AssignExpr>) -> () {
        self.resolve_expr(&expr.value);
        self.resolve_local(&expr.name, Expr::Assign(expr.clone()))
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> () {
        self.resolve_expr(&expr.expression);
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> () {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_condition_expr(&mut self, expr: &ConditionExpr) -> () {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> () {
        self.resolve_expr(&expr.expression);
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> () {
        self.resolve_expr(&expr.callee);

        for argument in &expr.arguments {
            self.resolve_expr(argument);
        }
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> () {
        self.resolve_expr(&expr.object);
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> () {
        self.resolve_expr(&expr.object);
        self.resolve_expr(&expr.value);
    }

    fn visit_this_expr(&mut self, expr: &Rc<ThisExpr>) -> () {
        match self.class_type {
            ClassType::Class => {
                self.resolve_local(&expr.token, Expr::This(expr.clone()));
            }
            _ => {
                self.error_collector
                    .borrow_mut()
                    .resolver_error(&expr.token, "Can't use 'this' outside of a class.");
            }
        }
    }
}
