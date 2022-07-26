use std::{borrow::BorrowMut, collections::HashMap, rc::Rc};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, CallExpr, ClassStmt, ConditionExpr, Expr, ExprVisitor,
        ExpressionStmt, FunctionStmt, GetExpr, GroupingExpr, IfStmt, LiteralExpr, PrintStmt,
        ReturnStmt, SetExpr, Stmt, StmtVisitor, SuperExpr, ThisExpr, UnaryExpr, VarStmt,
        VariableExpr, VisitExpr, VisitStmt, WhileStmt,
    },
    lox::ErrorCollector,
    token::Token,
};

#[derive(PartialEq, Eq, Copy, Clone)]
enum FunctionType {
    None,
    Function,
    Initialize,
    Method,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'a> {
    error_collector: &'a mut ErrorCollector,
    scopes: Vec<HashMap<String, bool>>,
    function_type: FunctionType,
    class_type: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(error_collector: &'a mut ErrorCollector) -> Resolver<'a> {
        Resolver {
            error_collector,
            scopes: vec![],
            function_type: FunctionType::None,
            class_type: ClassType::None,
        }
    }

    pub fn resolve(mut self, statements: &Vec<Rc<Stmt>>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_stmt(&mut self, statement: &Rc<Stmt>) {
        statement.accept(self);
    }

    fn resolve_stmt_vec(&mut self, statements: &Vec<Rc<Stmt>>) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_expr(&mut self, expression: &Rc<Expr>) {
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

    fn resolve_local_scope_index(&mut self, name: &Token) -> Option<usize> {
        for (scope_index, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                return Some(scope_index);
            }
        }

        None
    }
}

impl<'a> StmtVisitor<()> for Resolver<'a> {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt, _: &Rc<Stmt>) -> () {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt, _: &Rc<Stmt>) -> () {
        self.begin_scope();
        self.resolve_stmt_vec(&stmt.statements);
        self.end_scope();
    }

    fn visit_var_stmt(&mut self, stmt: &VarStmt, _: &Rc<Stmt>) -> () {
        self.declare(&stmt.name);

        if let Some(initializer) = &stmt.initializer {
            self.resolve_expr(initializer);
        }

        self.define(&stmt.name);
    }

    fn visit_function_stmt(&mut self, stmt: &FunctionStmt, _: &Rc<Stmt>) -> () {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function);
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt, _: &Rc<Stmt>) -> () {
        let outer_class_type = self.class_type;
        self.class_type = ClassType::Class;

        self.declare(&stmt.name);
        self.define(&stmt.name);

        if let Some(super_class_ptr) = &stmt.super_class {
            self.class_type = ClassType::SubClass;

            let super_class = &super_class_ptr.as_variable();
            if &stmt.name.lexeme == &super_class.name.lexeme {
                self.error_collector
                    .resolver_error(&super_class.name, "Class cannot extend itself.");
            }

            self.resolve_expr(super_class_ptr);

            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("super".to_string(), true);
        }

        for method in &stmt.methods {
            let method = method.as_function();

            self.begin_scope();
            self.scopes
                .last_mut()
                .unwrap()
                .insert("this".to_string(), true);

            let declaration = match method.name.lexeme == "init" {
                true => FunctionType::Initialize,
                false => FunctionType::Method,
            };
            self.resolve_function(method, declaration);

            self.end_scope();
        }

        if let Some(_) = &stmt.super_class {
            self.end_scope();
        }

        self.class_type = outer_class_type;
    }

    fn visit_print_stmt(&mut self, stmt: &PrintStmt, _: &Rc<Stmt>) -> () {
        self.resolve_expr(&stmt.expression);
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt, _: &Rc<Stmt>) -> () {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.then_statement);
        if let Some(else_statement) = &stmt.else_statement {
            self.resolve_stmt(else_statement);
        }
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt, _: &Rc<Stmt>) -> () {
        self.resolve_expr(&stmt.condition);
        self.resolve_stmt(&stmt.body);
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt, _: &Rc<Stmt>) -> () {
        match self.function_type {
            FunctionType::None => self
                .error_collector
                .resolver_error(&stmt.token, "Can't return from top level code."),
            FunctionType::Initialize => {
                if let Some(_) = stmt.value {
                    self.error_collector
                        .resolver_error(&stmt.token, "Can't return value from initializer.");
                }
            }
            _ => {}
        };

        if let Some(value) = &stmt.value {
            self.resolve_expr(value);
        }
    }
}

impl<'a> ExprVisitor<()> for Resolver<'a> {
    fn visit_literal_expr(&mut self, _: &LiteralExpr, _: &Rc<Expr>) -> () {}

    fn visit_variable_expr(&mut self, expr: &VariableExpr, _: &Rc<Expr>) -> () {
        if let Some(scope) = self.scopes.last() {
            if let Some(defined) = scope.get(&expr.name.lexeme) {
                if !defined {
                    self.error_collector.resolver_error(
                        &expr.name,
                        "Can't read local variable in it's own initializer.",
                    );
                }
            }
        }

        expr.scope_index
            .set(self.resolve_local_scope_index(&expr.name));
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.value);
        expr.scope_index
            .set(self.resolve_local_scope_index(&expr.name));
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.expression);
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_condition_expr(&mut self, expr: &ConditionExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.left);
        self.resolve_expr(&expr.right);
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.expression);
    }

    fn visit_call_expr(&mut self, expr: &CallExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.callee);

        for argument in &expr.arguments {
            self.resolve_expr(argument);
        }
    }

    fn visit_get_expr(&mut self, expr: &GetExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.object);
    }

    fn visit_set_expr(&mut self, expr: &SetExpr, _: &Rc<Expr>) -> () {
        self.resolve_expr(&expr.object);
        self.resolve_expr(&expr.value);
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr, _: &Rc<Expr>) -> () {
        match self.class_type {
            ClassType::Class | ClassType::SubClass => {
                expr.scope_index
                    .set(self.resolve_local_scope_index(&expr.token));
            }
            ClassType::None => {
                self.error_collector
                    .resolver_error(&expr.token, "Can't use 'this' outside of a class.");
            }
        }
    }

    fn visit_super_expr(&mut self, expr: &SuperExpr, _: &Rc<Expr>) -> () {
        match self.class_type {
            ClassType::None => self
                .error_collector
                .borrow_mut()
                .resolver_error(&expr.keyword, "Cannot use super outside of a class."),
            ClassType::Class => self.error_collector.borrow_mut().resolver_error(
                &expr.keyword,
                "Cannot use super in a class that is not a sub class.",
            ),
            ClassType::SubClass => {}
        };

        expr.scope_index
            .set(self.resolve_local_scope_index(&expr.keyword));
    }
}
