use std::{error::Error, fmt};

use crate::{
    ast::{
        AssignExpr, BinaryExpr, BlockStmt, Expr, ExpressionStmt, GroupingExpr, IfStmt, LiteralExpr,
        PrintStmt, Stmt, UnaryExpr, VarStmt, VariableExpr, WhileStmt,
    },
    lox::Lox,
    token::{LiteralValue, Token, TokenType},
};

pub struct Parser<'a> {
    lox: &'a mut Lox,
    tokens: &'a Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lox: &'a mut Lox, tokens: &'a Vec<Token<'a>>) -> Parser<'a> {
        Parser {
            lox,
            tokens,
            current: 0,
        }
    }

    pub fn parse(mut self) -> (Vec<Stmt<'a>>, &'a mut Lox) {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration_with_sync() {
                Some(statement) => statements.push(statement),
                None => {}
            }
        }

        (statements, self.lox)
    }

    fn declaration_with_sync(&mut self) -> Option<Stmt<'a>> {
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
            match self.peek().token_type {
                Var | Fun | Class | This | Super | If | For | While | Return => {
                    break;
                }
                _ => {}
            }

            self.advance();
        }
    }

    fn declaration(&mut self) -> Result<Stmt<'a>, ParserError> {
        if self.match_token(TokenType::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt<'a>, ParserError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = match self.match_token(TokenType::Equal) {
            true => Some(self.expression()?),
            false => None,
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var(Box::new(VarStmt { name, initializer })))
    }

    fn statement(&mut self) -> Result<Stmt<'a>, ParserError> {
        if self.match_token(TokenType::LeftBrace) {
            Ok(Stmt::Block(Box::new(BlockStmt {
                statements: self.block()?,
            })))
        } else if self.match_token(TokenType::Print) {
            self.print_stmt()
        } else if self.match_token(TokenType::If) {
            self.if_stmt()
        } else if self.match_token(TokenType::While) {
            self.while_stmt()
        } else {
            self.expression_stmt()
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt<'a>>, ParserError> {
        let mut statements = Vec::new();

        while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after statement block.")?;

        Ok(statements)
    }

    fn print_stmt(&mut self) -> Result<Stmt<'a>, ParserError> {
        let expression = self.expression()?;

        self.consume(TokenType::Semicolon, "Expect ';' after print statement.")?;

        Ok(Stmt::Print(Box::new(PrintStmt { expression })))
    }

    fn if_stmt(&mut self) -> Result<Stmt<'a>, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' before if condition.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' before if condition.")?;

        let then_statement = self.statement()?;

        let else_statement = match self.match_token(TokenType::Else) {
            true => Some(self.statement()?),
            false => None,
        };

        Ok(Stmt::If(Box::new(IfStmt {
            condition,
            then_statement,
            else_statement,
        })))
    }

    fn while_stmt(&mut self) -> Result<Stmt<'a>, ParserError> {
        self.consume(TokenType::LeftParen, "Expect '(' before while condition.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' before while condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While(Box::new(WhileStmt { condition, body })))
    }

    fn expression_stmt(&mut self) -> Result<Stmt<'a>, ParserError> {
        let expression = self.expression()?;

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after expression statement.",
        )?;

        Ok(Stmt::Expression(Box::new(ExpressionStmt { expression })))
    }

    fn expression(&mut self) -> Result<Expr<'a>, ParserError> {
        self.assign_expr()
    }

    fn assign_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let expr = self.equality_expr()?;

        if self.match_token(TokenType::Equal) {
            let name = match expr {
                Expr::Variable(expr) => expr.name,
                _ => {
                    return self.error(self.peek(), "Expect assignment to variable.");
                }
            };
            let value = self.assign_expr()?;
            Ok(Expr::Assign(Box::new(AssignExpr { name, value })))
        } else {
            Ok(expr)
        }
    }

    fn equality_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut expr = self.comparison_expr()?;

        while self.match_token(TokenType::EqualEqual) || self.match_token(TokenType::BangEqual) {
            let operator = self.previous();
            let right = self.comparison_expr()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn comparison_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut expr = self.sum_expr()?;

        while self.match_token(TokenType::Less)
            || self.match_token(TokenType::LessEqual)
            || self.match_token(TokenType::Greater)
            || self.match_token(TokenType::GreaterEqual)
        {
            let operator = self.previous();
            let right = self.sum_expr()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn sum_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut expr = self.factor_expr()?;

        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let operator = self.previous();
            let right = self.factor_expr()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn factor_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        let mut expr = self.unary_expr()?;

        while self.match_token(TokenType::Slash) || self.match_token(TokenType::Star) {
            let operator = self.previous();
            let right = self.unary_expr()?;
            expr = Expr::Binary(Box::new(BinaryExpr {
                left: expr,
                operator,
                right,
            }))
        }

        Ok(expr)
    }

    fn unary_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        if self.match_token(TokenType::Bang) || self.match_token(TokenType::Minus) {
            let operator = self.previous();
            let expression = self.unary_expr()?;
            Ok(Expr::Unary(Box::new(UnaryExpr {
                operator,
                expression,
            })))
        } else {
            self.grouping_expr()
        }
    }

    fn grouping_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        if self.match_token(TokenType::LeftParen) {
            let expression = self.expression()?;
            self.consume(
                TokenType::RightParen,
                "Expect ')' after grouping expression",
            )?;
            Ok(Expr::Grouping(Box::new(GroupingExpr { expression })))
        } else {
            self.primary_expr()
        }
    }

    fn primary_expr(&mut self) -> Result<Expr<'a>, ParserError> {
        if self.match_token(TokenType::Nil) {
            Ok(Expr::Literal(Box::new(LiteralExpr {
                value: LiteralValue::Nil,
            })))
        } else if self.match_token(TokenType::True) {
            Ok(Expr::Literal(Box::new(LiteralExpr {
                value: LiteralValue::Bool(true),
            })))
        } else if self.match_token(TokenType::False) {
            Ok(Expr::Literal(Box::new(LiteralExpr {
                value: LiteralValue::Bool(false),
            })))
        } else if self.match_token(TokenType::Number) || self.match_token(TokenType::String) {
            Ok(Expr::Literal(Box::new(LiteralExpr {
                value: self.previous().literal.unwrap(),
            })))
        } else if self.match_token(TokenType::Identifier) {
            Ok(Expr::Variable(Box::new(VariableExpr {
                name: self.previous(),
            })))
        } else {
            self.error(self.peek(), "Expected expression.")
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &'a Token<'a> {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &'a Token<'a> {
        &self.tokens[self.current - 1]
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

    fn consume(
        &mut self,
        token_type: TokenType,
        message: &str,
    ) -> Result<&'a Token<'a>, ParserError> {
        let token = self.peek();
        if token.token_type == token_type {
            self.advance();
            return Ok(token);
        }

        self.error(token, message)
    }

    fn error<T>(&mut self, token: &'a Token<'a>, message: &str) -> Result<T, ParserError> {
        self.lox.parser_error(token, message);
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
