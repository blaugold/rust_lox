use std::{hash::Hash, rc::Rc};

use crate::token::{LiteralValue, Token};

pub enum Stmt {
    Expression(Box<ExpressionStmt>),
    Block(Box<BlockStmt>),
    Var(Box<VarStmt>),
    Function(Rc<FunctionStmt>),
    Class(Rc<ClassStmt>),
    Print(Box<PrintStmt>),
    If(Box<IfStmt>),
    While(Box<WhileStmt>),
    Return(Box<ReturnStmt>),
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> T;
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> T;
    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> T;
    fn visit_function_stmt(&mut self, stmt: &Rc<FunctionStmt>) -> T;
    fn visit_class_stmt(&mut self, stmt: &Rc<ClassStmt>) -> T;
    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> T;
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> T;
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> T;
    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> T;
}

impl Stmt {
    pub fn accept<T, V: StmtVisitor<T>>(&self, visitor: &mut V) -> T {
        use Stmt::*;
        match self {
            Expression(expr) => visitor.visit_expression_stmt(expr),
            Block(expr) => visitor.visit_block_stmt(expr),
            Var(expr) => visitor.visit_var_stmt(expr),
            Function(expr) => visitor.visit_function_stmt(expr),
            Class(expr) => visitor.visit_class_stmt(expr),
            Print(expr) => visitor.visit_print_stmt(expr),
            If(expr) => visitor.visit_if_stmt(expr),
            While(expr) => visitor.visit_while_stmt(expr),
            Return(expr) => visitor.visit_return_stmt(expr),
        }
    }
}

pub struct ExpressionStmt {
    pub expression: Expr,
}

pub struct BlockStmt {
    pub statements: Vec<Stmt>,
}

pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

pub struct FunctionStmt {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Stmt>,
}

pub struct ClassStmt {
    pub name: Token,
    pub methods: Vec<Stmt>,
}

pub struct PrintStmt {
    pub expression: Expr,
}

pub struct IfStmt {
    pub condition: Expr,
    pub then_statement: Stmt,
    pub else_statement: Option<Stmt>,
}

pub struct WhileStmt {
    pub condition: Expr,
    pub body: Stmt,
}

pub struct ReturnStmt {
    pub token: Token,
    pub value: Option<Expr>,
}

pub enum Expr {
    Literal(Box<LiteralExpr>),
    Variable(Rc<VariableExpr>),
    Assign(Rc<AssignExpr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Condition(Box<ConditionExpr>),
    Grouping(Box<GroupingExpr>),
    Call(Box<CallExpr>),
    Get(Box<GetExpr>),
    Set(Box<SetExpr>),
}

impl Eq for Expr {}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        use Expr::*;
        match (self, other) {
            (Literal(l), Literal(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Variable(l), Variable(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Assign(l), Assign(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Unary(l), Unary(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Binary(l), Binary(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Condition(l), Condition(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Grouping(l), Grouping(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Call(l), Call(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Get(l), Get(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            (Set(l), Set(r)) => std::ptr::eq(l.as_ref(), r.as_ref()),
            _ => false,
        }
    }
}

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Expr::*;

        match self {
            Literal(v) => std::ptr::hash(v.as_ref(), state),
            Variable(v) => std::ptr::hash(v.as_ref(), state),
            Assign(v) => std::ptr::hash(v.as_ref(), state),
            Unary(v) => std::ptr::hash(v.as_ref(), state),
            Binary(v) => std::ptr::hash(v.as_ref(), state),
            Condition(v) => std::ptr::hash(v.as_ref(), state),
            Grouping(v) => std::ptr::hash(v.as_ref(), state),
            Call(v) => std::ptr::hash(v.as_ref(), state),
            Get(v) => std::ptr::hash(v.as_ref(), state),
            Set(v) => std::ptr::hash(v.as_ref(), state),
        }
    }
}

pub trait ExprVisitor<T> {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> T;
    fn visit_variable_expr(&mut self, expr: &Rc<VariableExpr>) -> T;
    fn visit_assign_expr(&mut self, expr: &Rc<AssignExpr>) -> T;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> T;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> T;
    fn visit_condition_expr(&mut self, expr: &ConditionExpr) -> T;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> T;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> T;
    fn visit_get_expr(&mut self, expr: &GetExpr) -> T;
    fn visit_set_expr(&mut self, expr: &SetExpr) -> T;
}

impl Expr {
    pub fn accept<T, V: ExprVisitor<T>>(&self, visitor: &mut V) -> T {
        use Expr::*;
        match self {
            Literal(expr) => visitor.visit_literal_expr(expr),
            Variable(expr) => visitor.visit_variable_expr(expr),
            Assign(expr) => visitor.visit_assign_expr(expr),
            Unary(expr) => visitor.visit_unary_expr(expr),
            Binary(expr) => visitor.visit_binary_expr(expr),
            Condition(expr) => visitor.visit_condition_expr(expr),
            Grouping(expr) => visitor.visit_grouping_expr(expr),
            Call(expr) => visitor.visit_call_expr(expr),
            Get(expr) => visitor.visit_get_expr(expr),
            Set(expr) => visitor.visit_set_expr(expr),
        }
    }
}

pub struct LiteralExpr {
    pub value: LiteralValue,
}

pub struct VariableExpr {
    pub name: Token,
}

pub struct AssignExpr {
    pub name: Token,
    pub value: Expr,
}

pub struct UnaryExpr {
    pub operator: Token,
    pub expression: Expr,
}

pub struct BinaryExpr {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

pub struct ConditionExpr {
    pub left: Expr,
    pub operator: Token,
    pub right: Expr,
}

pub struct GroupingExpr {
    pub expression: Expr,
}

pub struct CallExpr {
    pub callee: Expr,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

pub struct GetExpr {
    pub object: Expr,
    pub name: Token,
}

pub struct SetExpr {
    pub object: Expr,
    pub name: Token,
    pub value: Expr,
}
