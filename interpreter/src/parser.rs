use std::{error::Error, fmt, rc::Rc};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, CallExpr, ClassStmt, ConditionExpr, Expr,
        ExpressionStmt, FunctionStmt, GetExpr, GroupingExpr, IfStmt, LiteralExpr, PrintStmt,
        ReturnStmt, SetExpr, Stmt, SuperExpr, ThisExpr, UnaryExpr, VarStmt, VariableExpr,
        WhileStmt,
    },
    lox::ErrorCollector,
    token::{LiteralValue, Token, TokenType},
    utils::Late,
};

pub struct Parser<'a> {
    error_collector: &'a mut ErrorCollector,
    tokens: Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(error_collector: &'a mut ErrorCollector, tokens: Vec<Token>) -> Parser {
        Parser {
            error_collector,
            tokens,
            current: 0,
        }
    }

    pub fn parse(mut self) -> Vec<Rc<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(statement) = self.try_declaration() {
                statements.push(statement);
            }
        }

        statements
    }

    fn try_declaration(&mut self) -> Option<Rc<Stmt>> {
        match self.declaration() {
            Ok(statement) => Some(statement),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        loop {
            if self.is_at_end() {
                break;
            }

            if self.match_token(TokenType::Semicolon) {
                break;
            }

            use TokenType::*;
            if let Var | Fun | Class | This | Super | If | For | While | Return =
                self.peek().token_type
            {
                break;
            }

            self.advance();
        }
    }

    fn declaration(&mut self) -> Result<Rc<Stmt>, ParserError> {
        if self.match_token(TokenType::Fun) {
            self.function_declaration("function")
        } else if self.match_token(TokenType::Class) {
            self.class_declaration()
        } else if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn function_declaration(&mut self, kind: &str) -> Result<Rc<Stmt>, ParserError> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;

        self.consume(TokenType::LeftParen, "Expect '(' before parameters.")?;

        let mut parameters = vec![];

        while self.peek().token_type != TokenType::RightParen {
            if parameters.len() >= 255 {
                let _ = self.error::<()>(
                    &self.peek().clone(),
                    "Cannot have more than 255 parameters.",
                );
            }

            parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

            if !self.match_token(TokenType::Comma) {
                break;
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(TokenType::LeftBrace, "Expect '{' after parameters.")?;

        let body = self.block()?;

        Ok(Rc::new(Stmt::Function(FunctionStmt {
            name,
            parameters,
            body,
        })))
    }

    fn class_declaration(&mut self) -> Result<Rc<Stmt>, ParserError> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;

        let mut super_class = None;
        if self.match_token(TokenType::Less) {
            let name = self.consume(TokenType::Identifier, "Expect super class name.")?;
            super_class = Some(Rc::new(Expr::Variable(VariableExpr {
                name,
                scope_index: Late::new(),
            })));
        }

        self.consume(TokenType::LeftBrace, "Expect '{' after class name.")?;

        let mut methods: Vec<Rc<Stmt>> = vec![];
        while !self.is_at_end() && self.peek().token_type != TokenType::RightBrace {
            methods.push(self.function_declaration("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Rc::new(Stmt::Class(ClassStmt {
            name,
            super_class,
            methods,
        })))
    }

    fn var_declaration(&mut self) -> Result<Rc<Stmt>, ParserError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = match self.match_token(TokenType::Equal) {
            true => Some(self.expression()?),
            false => None,
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Rc::new(Stmt::Var(VarStmt { name, initializer })))
    }

    fn statement(&mut self) -> Result<Rc<Stmt>, ParserError> {
        if self.match_token(TokenType::LeftBrace) {
            Ok(Rc::new(Stmt::Block(BlockStmt {
                statements: self.block()?,
            })))
        } else if self.match_token(TokenType::Print) {
            self.print_stmt()
        } else if self.match_token(TokenType::If) {
            self.if_stmt()
        } else if self.match_token(TokenType::While) {
            self.while_stmt()
        } else if self.match_token(TokenType::For) {
            self.for_stmt()
        } else if self.match_token(TokenType::Return) {
            self.return_stmt()
        } else {
            self.expression_stmt()
        }
    }

    fn block(&mut self) -> Result<Vec<Rc<Stmt>>, ParserError> {
        let mut statements = Vec::new();

        while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after statement block.")?;

        Ok(statements)
    }

    fn print_stmt(&mut self) -> Result<Rc<Stmt>, ParserError> {
        let expression = self.expression()?;

        self.consume(TokenType::Semicolon, "Expect ';' after print statement.")?;

        Ok(Rc::new(Stmt::Print(PrintStmt { expression })))
    }

    fn if_stmt(&mut self) -> Result<Rc<Stmt>, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' before if condition.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' before if condition.")?;

        let then_statement = self.statement()?;

        let else_statement = match self.match_token(TokenType::Else) {
            true => Some(self.statement()?),
            false => None,
        };

        Ok(Rc::new(Stmt::If(IfStmt {
            condition,
            then_statement,
            else_statement,
        })))
    }

    fn while_stmt(&mut self) -> Result<Rc<Stmt>, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' before while condition.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' before while condition.")?;

        let body = self.statement()?;

        Ok(Rc::new(Stmt::While(WhileStmt { condition, body })))
    }

    fn for_stmt(&mut self) -> Result<Rc<Stmt>, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' before for initializer.")?;

        let initializer = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.match_token(TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_stmt()?)
        };

        let condition = if self.match_token(TokenType::Semicolon) {
            Rc::new(Expr::Literal(LiteralExpr {
                value: LiteralValue::Bool(true),
            }))
        } else {
            let expr = self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after for condition.")?;
            expr
        };

        let increment = if self.match_token(TokenType::RightParen) {
            None
        } else {
            let expr = Some(self.expression()?);
            self.consume(TokenType::RightParen, "Expect ')' after for increment.")?;
            expr
        };

        let mut body = self.statement()?;

        if let Some(expression) = increment {
            body = Rc::new(Stmt::Block(BlockStmt {
                statements: vec![
                    body,
                    Rc::new(Stmt::Expression(ExpressionStmt { expression })),
                ],
            }))
        };

        body = Rc::new(Stmt::While(WhileStmt { condition, body }));

        if let Some(statement) = initializer {
            body = Rc::new(Stmt::Block(BlockStmt {
                statements: vec![statement, body],
            }))
        }

        Ok(body)
    }

    fn return_stmt(&mut self) -> Result<Rc<Stmt>, ParserError> {
        let token = self.previous();

        let value = if self.match_token(TokenType::Semicolon) {
            None
        } else {
            let expression = self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
            Some(expression)
        };

        Ok(Rc::new(Stmt::Return(ReturnStmt { token, value })))
    }

    fn expression_stmt(&mut self) -> Result<Rc<Stmt>, ParserError> {
        let expression = self.expression()?;

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after expression statement.",
        )?;

        Ok(Rc::new(Stmt::Expression(ExpressionStmt { expression })))
    }

    fn expression(&mut self) -> Result<Rc<Expr>, ParserError> {
        self.assign_expr()
    }

    fn assign_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let expr = self.or_expr()?;

        if self.match_token(TokenType::Equal) {
            let value = self.assign_expr()?;

            match expr.as_ref() {
                Expr::Variable(expr) => Ok(Rc::new(Expr::Assign(AssignExpr {
                    name: expr.name.clone(),
                    value,
                    scope_index: Late::new(),
                }))),
                Expr::Get(expr) => Ok(Rc::new(Expr::Set(SetExpr {
                    object: expr.object.clone(),
                    name: expr.name.clone(),
                    value,
                }))),
                _ => {
                    return self.error(
                        &self.peek().clone(),
                        "Expect assignment to variable or property.",
                    );
                }
            }
        } else {
            Ok(expr)
        }
    }

    fn or_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expr = self.and_expr()?;

        while self.match_token(TokenType::Or) {
            let operator = self.previous();
            let right = self.and_expr()?;
            expr = Rc::new(Expr::Condition(ConditionExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn and_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expr = self.equality_expr()?;

        while self.match_token(TokenType::And) {
            let operator = self.previous();
            let right = self.equality_expr()?;
            expr = Rc::new(Expr::Condition(ConditionExpr {
                left: expr,
                operator,
                right,
            }));
        }

        Ok(expr)
    }

    fn equality_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expr = self.comparison_expr()?;

        while self.match_token(TokenType::EqualEqual) || self.match_token(TokenType::BangEqual) {
            let operator = self.previous();
            let right = self.comparison_expr()?;
            expr = Rc::new(Expr::Binary(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn comparison_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expr = self.sum_expr()?;

        while self.match_token(TokenType::Less)
            || self.match_token(TokenType::LessEqual)
            || self.match_token(TokenType::Greater)
            || self.match_token(TokenType::GreaterEqual)
        {
            let operator = self.previous();
            let right = self.sum_expr()?;
            expr = Rc::new(Expr::Binary(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn sum_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expr = self.factor_expr()?;

        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let operator = self.previous();
            let right = self.factor_expr()?;
            expr = Rc::new(Expr::Binary(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn factor_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expr = self.unary_expr()?;

        while self.match_token(TokenType::Slash) || self.match_token(TokenType::Star) {
            let operator = self.previous();
            let right = self.unary_expr()?;
            expr = Rc::new(Expr::Binary(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn unary_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        if self.match_token(TokenType::Bang) || self.match_token(TokenType::Minus) {
            let operator = self.previous();
            let expression = self.unary_expr()?;
            Ok(Rc::new(Expr::Unary(UnaryExpr {
                operator,
                expression,
            })))
        } else {
            self.grouping_expr()
        }
    }

    fn grouping_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        if self.match_token(TokenType::LeftParen) {
            let expression = self.expression()?;
            self.consume(
                TokenType::RightParen,
                "Expect ')' after grouping expression",
            )?;
            Ok(Rc::new(Expr::Grouping(GroupingExpr { expression })))
        } else {
            self.call_expr()
        }
    }

    fn call_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        let mut expression = self.primary_expr()?;

        loop {
            if self.match_token(TokenType::LeftParen) {
                expression = self.finish_call_expr(expression)?;
            } else if self.match_token(TokenType::Dot) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                expression = Rc::new(Expr::Get(GetExpr {
                    object: expression,
                    name,
                }))
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn finish_call_expr(&mut self, callee: Rc<Expr>) -> Result<Rc<Expr>, ParserError> {
        let mut arguments = vec![];

        loop {
            if self.peek().token_type == TokenType::RightParen {
                break;
            }

            if arguments.len() >= 255 {
                let _ =
                    self.error::<()>(&self.peek().clone(), "Cannot have more than 255 arguments.");
            }

            arguments.push(self.expression()?);

            if self.match_token(TokenType::Comma) {
                break;
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Rc::new(Expr::Call(CallExpr {
            callee,
            paren,
            arguments,
        })))
    }

    fn primary_expr(&mut self) -> Result<Rc<Expr>, ParserError> {
        if self.match_token(TokenType::Nil) {
            Ok(Rc::new(Expr::Literal(LiteralExpr {
                value: LiteralValue::Nil,
            })))
        } else if self.match_token(TokenType::True) {
            Ok(Rc::new(Expr::Literal(LiteralExpr {
                value: LiteralValue::Bool(true),
            })))
        } else if self.match_token(TokenType::False) {
            Ok(Rc::new(Expr::Literal(LiteralExpr {
                value: LiteralValue::Bool(false),
            })))
        } else if self.match_token(TokenType::Number) || self.match_token(TokenType::String) {
            Ok(Rc::new(Expr::Literal(LiteralExpr {
                value: self.previous().literal.unwrap(),
            })))
        } else if self.match_token(TokenType::Identifier) {
            Ok(Rc::new(Expr::Variable(VariableExpr {
                name: self.previous(),
                scope_index: Late::new(),
            })))
        } else if self.match_token(TokenType::This) {
            Ok(Rc::new(Expr::This(ThisExpr {
                token: self.previous(),
                scope_index: Late::new(),
            })))
        } else if self.match_token(TokenType::Super) {
            let keyword = self.previous();

            self.consume(TokenType::Dot, "Expect '.' after super.")?;

            let method = self.consume(TokenType::Identifier, "Expect super method.")?;

            Ok(Rc::new(Expr::Super(SuperExpr {
                keyword,
                method,
                scope_index: Late::new(),
            })))
        } else {
            self.error(&self.peek().clone(), "Expected expression.")
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek<'b>(&'b self) -> &'b Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.peek().token_type == token_type {
            self.advance();
            return true;
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParserError> {
        let token = self.peek().clone();
        if token.token_type == token_type {
            self.advance();
            return Ok(token);
        }

        self.error(&token, message)
    }

    fn error<T>(&mut self, token: &Token, message: &str) -> Result<T, ParserError> {
        self.error_collector.parser_error(token, message);
        Err(ParserError {})
    }
}

#[derive(Debug)]
struct ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParserError")
    }
}

impl Error for ParserError {}
