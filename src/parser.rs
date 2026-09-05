use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(error) => {
                    errors.push(error);
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(Program { statements })
        } else {
            Err(errors)
        }
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kind(&TokenKind::Fn) {
            return self.function_declaration();
        }
        if self.match_kind(&TokenKind::Let) {
            return self.let_declaration();
        }
        self.statement()
    }

    fn function_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.expect_identifier("expected function name")?;
        self.consume(&TokenKind::LeftParen, "expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                params.push(self.expect_identifier("expected parameter name")?);
                if !self.match_kind(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenKind::RightParen, "expected ')' after parameters")?;
        self.consume(&TokenKind::LeftBrace, "expected '{' before function body")?;
        let body = self.block_statements()?;

        Ok(Stmt::Function { name, params, body })
    }

    fn let_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.expect_identifier("expected variable name")?;
        self.consume(&TokenKind::Equal, "expected '=' after variable name")?;
        let initializer = self.expression()?;
        self.consume(&TokenKind::Semicolon, "expected ';' after variable declaration")?;
        Ok(Stmt::Let { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_kind(&TokenKind::If) {
            return self.if_statement();
        }
        if self.match_kind(&TokenKind::Print) {
            let value = self.expression()?;
            self.consume(&TokenKind::Semicolon, "expected ';' after print expression")?;
            return Ok(Stmt::Print(value));
        }
        if self.match_kind(&TokenKind::Return) {
            let value = if self.check(&TokenKind::Semicolon) {
                None
            } else {
                Some(self.expression()?)
            };
            self.consume(&TokenKind::Semicolon, "expected ';' after return")?;
            return Ok(Stmt::Return(value));
        }
        if self.match_kind(&TokenKind::LeftBrace) {
            return Ok(Stmt::Block(self.block_statements()?));
        }

        let expression = self.expression()?;
        self.consume(&TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::Expression(expression))
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        self.consume(&TokenKind::LeftBrace, "expected '{' after if condition")?;
        let then_branch = Stmt::Block(self.block_statements()?);
        let else_branch = if self.match_kind(&TokenKind::Else) {
            if self.match_kind(&TokenKind::If) {
                Some(Box::new(self.if_statement()?))
            } else {
                self.consume(&TokenKind::LeftBrace, "expected '{' after else")?;
                Some(Box::new(Stmt::Block(self.block_statements()?)))
            }
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&TokenKind::RightBrace, "expected '}' after block")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expression = self.or()?;

        if self.match_kind(&TokenKind::Equal) {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expression {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(self.error("invalid assignment target"));
        }

        Ok(expression)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        self.binary(|parser| parser.and(), &[TokenKind::OrOr], BinaryOp::Or)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        self.binary(|parser| parser.equality(), &[TokenKind::AndAnd], BinaryOp::And)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_kind(&TokenKind::EqualEqual) || self.match_kind(&TokenKind::BangEqual) {
            let operator = if self.previous_is(&TokenKind::EqualEqual) {
                BinaryOp::Equal
            } else {
                BinaryOp::NotEqual
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        loop {
            let operator = if self.match_kind(&TokenKind::Less) {
                Some(BinaryOp::Less)
            } else if self.match_kind(&TokenKind::LessEqual) {
                Some(BinaryOp::LessEqual)
            } else if self.match_kind(&TokenKind::Greater) {
                Some(BinaryOp::Greater)
            } else if self.match_kind(&TokenKind::GreaterEqual) {
                Some(BinaryOp::GreaterEqual)
            } else {
                None
            };

            let Some(operator) = operator else { break };
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        loop {
            let operator = if self.match_kind(&TokenKind::Plus) {
                Some(BinaryOp::Add)
            } else if self.match_kind(&TokenKind::Minus) {
                Some(BinaryOp::Subtract)
            } else {
                None
            };
            let Some(operator) = operator else { break };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        loop {
            let operator = if self.match_kind(&TokenKind::Star) {
                Some(BinaryOp::Multiply)
            } else if self.match_kind(&TokenKind::Slash) {
                Some(BinaryOp::Divide)
            } else if self.match_kind(&TokenKind::Percent) {
                Some(BinaryOp::Modulo)
            } else {
                None
            };
            let Some(operator) = operator else { break };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_kind(&TokenKind::Bang) {
            return Ok(Expr::Unary {
                operator: UnaryOp::Not,
                operand: Box::new(self.unary()?),
            });
        }
        if self.match_kind(&TokenKind::Minus) {
            return Ok(Expr::Unary {
                operator: UnaryOp::Negate,
                operand: Box::new(self.unary()?),
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_kind(&TokenKind::LeftParen) {
                let mut arguments = Vec::new();
                if !self.check(&TokenKind::RightParen) {
                    loop {
                        arguments.push(self.expression()?);
                        if !self.match_kind(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.consume(&TokenKind::RightParen, "expected ')' after arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    arguments,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(value) => Ok(Expr::Number(value)),
            TokenKind::String(value) => Ok(Expr::String(value)),
            TokenKind::True => Ok(Expr::Boolean(true)),
            TokenKind::False => Ok(Expr::Boolean(false)),
            TokenKind::Identifier(name) => Ok(Expr::Variable(name)),
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TokenKind::RightParen, "expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: "expected an expression".into(),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn binary<F>(&mut self, mut next: F, operators: &[TokenKind], operator: BinaryOp) -> Result<Expr, ParseError>
    where
        F: FnMut(&mut Self) -> Result<Expr, ParseError>,
    {
        let mut expr = next(self)?;
        while operators.iter().any(|kind| self.check(kind)) {
            self.advance();
            let right = next(self)?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if self.previous_is(&TokenKind::Semicolon) {
                return;
            }
            match self.peek().kind {
                TokenKind::Let | TokenKind::Fn | TokenKind::If | TokenKind::Return | TokenKind::Print => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn expect_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if let TokenKind::Identifier(name) = self.advance().kind.clone() {
            Ok(name)
        } else {
            Err(self.error(message))
        }
    }

    fn consume(&mut self, expected: &TokenKind, message: &str) -> Result<(), ParseError> {
        if self.check(expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn match_kind(&mut self, expected: &TokenKind) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(expected)
    }

    fn previous_is(&self, expected: &TokenKind) -> bool {
        if self.current == 0 {
            return false;
        }
        std::mem::discriminant(&self.tokens[self.current - 1].kind)
            == std::mem::discriminant(expected)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.into(),
            line: self.peek().line,
            column: self.peek().column,
        }
    }
}
