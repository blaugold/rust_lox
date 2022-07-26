use std::rc::Rc;

use crate::{
    token::{LiteralValue, Token},
    utils::Late,
};

pub enum Stmt {
    Expression(ExpressionStmt),
    Block(BlockStmt),
    Var(VarStmt),
    Function(FunctionStmt),
    Class(ClassStmt),
    Print(PrintStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(ReturnStmt),
}

impl Stmt {
    pub fn as_function(&self) -> &FunctionStmt {
        match self {
            Stmt::Function(stmt) => stmt,
            _ => panic!(),
        }
    }
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_block_stmt(&mut self, stmt: &BlockStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_var_stmt(&mut self, stmt: &VarStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_function_stmt(&mut self, stmt: &FunctionStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_class_stmt(&mut self, stmt: &ClassStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_print_stmt(&mut self, stmt: &PrintStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_if_stmt(&mut self, stmt: &IfStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_while_stmt(&mut self, stmt: &WhileStmt, ptr: &Rc<Stmt>) -> T;
    fn visit_return_stmt(&mut self, stmt: &ReturnStmt, ptr: &Rc<Stmt>) -> T;
}

pub trait VisitStmt {
    fn accept<T, V: StmtVisitor<T>>(&self, visitor: &mut V) -> T;
}

impl VisitStmt for Rc<Stmt> {
    fn accept<T, V: StmtVisitor<T>>(&self, visitor: &mut V) -> T {
        use Stmt::*;
        match self.as_ref() {
            Expression(expr) => visitor.visit_expression_stmt(expr, self),
            Block(expr) => visitor.visit_block_stmt(expr, self),
            Var(expr) => visitor.visit_var_stmt(expr, self),
            Function(expr) => visitor.visit_function_stmt(expr, self),
            Class(expr) => visitor.visit_class_stmt(expr, self),
            Print(expr) => visitor.visit_print_stmt(expr, self),
            If(expr) => visitor.visit_if_stmt(expr, self),
            While(expr) => visitor.visit_while_stmt(expr, self),
            Return(expr) => visitor.visit_return_stmt(expr, self),
        }
    }
}

pub struct ExpressionStmt {
    pub expression: Rc<Expr>,
}

pub struct BlockStmt {
    pub statements: Vec<Rc<Stmt>>,
}

pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Rc<Expr>>,
}

pub struct FunctionStmt {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Rc<Stmt>>,
}

pub struct ClassStmt {
    pub name: Token,
    pub super_class: Option<Rc<Expr>>,
    pub methods: Vec<Rc<Stmt>>,
}

pub struct PrintStmt {
    pub expression: Rc<Expr>,
}

pub struct IfStmt {
    pub condition: Rc<Expr>,
    pub then_statement: Rc<Stmt>,
    pub else_statement: Option<Rc<Stmt>>,
}

pub struct WhileStmt {
    pub condition: Rc<Expr>,
    pub body: Rc<Stmt>,
}

pub struct ReturnStmt {
    pub token: Token,
    pub value: Option<Rc<Expr>>,
}

pub enum Expr {
    Literal(LiteralExpr),
    Variable(VariableExpr),
    Assign(AssignExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Condition(ConditionExpr),
    Grouping(GroupingExpr),
    Call(CallExpr),
    Get(GetExpr),
    Set(SetExpr),
    This(ThisExpr),
    Super(SuperExpr),
}

impl Expr {
    pub fn as_variable(&self) -> &VariableExpr {
        match self {
            Expr::Variable(expr) => expr,
            _ => panic!(),
        }
    }
}

pub trait VisitExpr {
    fn accept<T, V: ExprVisitor<T>>(&self, visitor: &mut V) -> T;
}

pub trait ExprVisitor<T> {
    fn visit_literal_expr(&mut self, expr: &LiteralExpr, ptr: &Rc<Expr>) -> T;
    fn visit_variable_expr(&mut self, expr: &VariableExpr, ptr: &Rc<Expr>) -> T;
    fn visit_assign_expr(&mut self, expr: &AssignExpr, ptr: &Rc<Expr>) -> T;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr, ptr: &Rc<Expr>) -> T;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr, ptr: &Rc<Expr>) -> T;
    fn visit_condition_expr(&mut self, expr: &ConditionExpr, ptr: &Rc<Expr>) -> T;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr, ptr: &Rc<Expr>) -> T;
    fn visit_call_expr(&mut self, expr: &CallExpr, ptr: &Rc<Expr>) -> T;
    fn visit_get_expr(&mut self, expr: &GetExpr, ptr: &Rc<Expr>) -> T;
    fn visit_set_expr(&mut self, expr: &SetExpr, ptr: &Rc<Expr>) -> T;
    fn visit_this_expr(&mut self, expr: &ThisExpr, ptr: &Rc<Expr>) -> T;
    fn visit_super_expr(&mut self, expr: &SuperExpr, ptr: &Rc<Expr>) -> T;
}

impl VisitExpr for Rc<Expr> {
    fn accept<T, V: ExprVisitor<T>>(&self, visitor: &mut V) -> T {
        use Expr::*;
        match self.as_ref() {
            Literal(expr) => visitor.visit_literal_expr(expr, self),
            Variable(expr) => visitor.visit_variable_expr(expr, self),
            Assign(expr) => visitor.visit_assign_expr(expr, self),
            Unary(expr) => visitor.visit_unary_expr(expr, self),
            Binary(expr) => visitor.visit_binary_expr(expr, self),
            Condition(expr) => visitor.visit_condition_expr(expr, self),
            Grouping(expr) => visitor.visit_grouping_expr(expr, self),
            Call(expr) => visitor.visit_call_expr(expr, self),
            Get(expr) => visitor.visit_get_expr(expr, self),
            Set(expr) => visitor.visit_set_expr(expr, self),
            This(expr) => visitor.visit_this_expr(expr, self),
            Super(expr) => visitor.visit_super_expr(expr, self),
        }
    }
}

pub struct LiteralExpr {
    pub value: LiteralValue,
}

pub struct VariableExpr {
    pub name: Token,
    pub scope_index: Late<Option<usize>>,
}

pub struct AssignExpr {
    pub name: Token,
    pub value: Rc<Expr>,
    pub scope_index: Late<Option<usize>>,
}

pub struct UnaryExpr {
    pub operator: Token,
    pub expression: Rc<Expr>,
}

pub struct BinaryExpr {
    pub left: Rc<Expr>,
    pub operator: Token,
    pub right: Rc<Expr>,
}

pub struct ConditionExpr {
    pub left: Rc<Expr>,
    pub operator: Token,
    pub right: Rc<Expr>,
}

pub struct GroupingExpr {
    pub expression: Rc<Expr>,
}

pub struct CallExpr {
    pub callee: Rc<Expr>,
    pub paren: Token,
    pub arguments: Vec<Rc<Expr>>,
}

pub struct GetExpr {
    pub object: Rc<Expr>,
    pub name: Token,
}

pub struct SetExpr {
    pub object: Rc<Expr>,
    pub name: Token,
    pub value: Rc<Expr>,
}

pub struct ThisExpr {
    pub token: Token,
    pub scope_index: Late<Option<usize>>,
}

pub struct SuperExpr {
    pub keyword: Token,
    pub method: Token,
    pub scope_index: Late<Option<usize>>,
}
