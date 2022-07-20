use crate::token::{LiteralValue, Token};

pub enum Stmt<'a> {
    Expression(Box<ExpressionStmt<'a>>),
    Block(Box<BlockStmt<'a>>),
    Var(Box<VarStmt<'a>>),
    Print(Box<PrintStmt<'a>>),
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> T;
    fn visit_block_stmt(&self, stmt: &BlockStmt) -> T;
    fn visit_var_stmt(&self, stmt: &VarStmt) -> T;
    fn visit_print_stmt(&self, stmt: &PrintStmt) -> T;
}

impl<'a> Stmt<'a> {
    pub fn accept<T, V: StmtVisitor<T>>(&self, visitor: V) -> T {
        use Stmt::*;
        match self {
            Expression(expr) => visitor.visit_expression_stmt(expr),
            Var(expr) => visitor.visit_var_stmt(expr),
            Block(expr) => visitor.visit_block_stmt(expr),
            Print(expr) => visitor.visit_print_stmt(expr),
        }
    }
}

pub struct ExpressionStmt<'a> {
    pub expression: Expr<'a>,
}

pub struct BlockStmt<'a> {
    pub expressions: Vec<Stmt<'a>>,
}

pub struct VarStmt<'a> {
    pub name: Token<'a>,
    pub expression: Expr<'a>,
}

pub struct PrintStmt<'a> {
    pub expression: Expr<'a>,
}

pub enum Expr<'a> {
    Literal(Box<LiteralExpr<'a>>),
    Variable(Box<VariableExpr<'a>>),
    Assign(Box<AssignExpr<'a>>),
    Unary(Box<UnaryExpr<'a>>),
    Binary(Box<BinaryExpr<'a>>),
    Grouping(Box<GroupingExpr<'a>>),
}

pub trait ExprVisitor<T> {
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> T;
    fn visit_variable_expr(&self, expr: &VariableExpr) -> T;
    fn visit_assign_expr(&self, expr: &AssignExpr) -> T;
    fn visit_unary_expr(&self, expr: &UnaryExpr) -> T;
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> T;
    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> T;
}

impl<'a> Expr<'a> {
    pub fn accept<T, V: ExprVisitor<T>>(&self, visitor: V) -> T {
        use Expr::*;
        match self {
            Literal(expr) => visitor.visit_literal_expr(expr),
            Variable(expr) => visitor.visit_variable_expr(expr),
            Assign(expr) => visitor.visit_assign_expr(expr),
            Unary(expr) => visitor.visit_unary_expr(expr),
            Binary(expr) => visitor.visit_binary_expr(expr),
            Grouping(expr) => visitor.visit_grouping_expr(expr),
        }
    }
}

pub struct LiteralExpr<'a> {
    pub value: LiteralValue<'a>,
}

pub struct VariableExpr<'a> {
    pub identifier: Token<'a>,
}

pub struct AssignExpr<'a> {
    pub left: Expr<'a>,
    pub right: Expr<'a>,
}

pub struct UnaryExpr<'a> {
    pub operator: Token<'a>,
    pub expression: Expr<'a>,
}

pub struct BinaryExpr<'a> {
    pub left: Expr<'a>,
    pub operator: Token<'a>,
    pub right: Expr<'a>,
}

pub struct GroupingExpr<'a> {
    pub expression: Expr<'a>,
}
