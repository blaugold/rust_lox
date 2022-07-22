use std::rc::Rc;

use crate::token::{LiteralValue, Token};

pub enum Stmt {
    Expression(Box<ExpressionStmt>),
    Block(Box<BlockStmt>),
    Var(Box<VarStmt>),
    Function(Rc<FunctionStmt>),
    Print(Box<PrintStmt>),
    If(Box<IfStmt>),
    While(Box<WhileStmt>),
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> T;
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> T;
    fn visit_var_stmt(&mut self, stmt: &VarStmt) -> T;
    fn visit_function_stmt(&mut self, stmt: &Rc<FunctionStmt>) -> T;
    fn visit_print_stmt(&mut self, stmt: &PrintStmt) -> T;
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> T;
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> T;
}

impl Stmt {
    pub fn accept<T, V: StmtVisitor<T>>(&self, visitor: &mut V) -> T {
        use Stmt::*;
        match self {
            Expression(expr) => visitor.visit_expression_stmt(expr),
            Block(expr) => visitor.visit_block_stmt(expr),
            Var(expr) => visitor.visit_var_stmt(expr),
            Function(expr) => visitor.visit_function_stmt(expr),
            Print(expr) => visitor.visit_print_stmt(expr),
            If(expr) => visitor.visit_if_stmt(expr),
            While(expr) => visitor.visit_while_stmt(expr),
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

pub enum Expr {
    Literal(Box<LiteralExpr>),
    Variable(Box<VariableExpr>),
    Assign(Box<AssignExpr>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Grouping(Box<GroupingExpr>),
    Call(Box<CallExpr>),
}

pub trait ExprVisitor<T> {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> T;
    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> T;
    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> T;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> T;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> T;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> T;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> T;
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
            Grouping(expr) => visitor.visit_grouping_expr(expr),
            Call(expr) => visitor.visit_call_expr(expr),
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

pub struct GroupingExpr {
    pub expression: Expr,
}

pub struct CallExpr {
    pub callee: Expr,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}
