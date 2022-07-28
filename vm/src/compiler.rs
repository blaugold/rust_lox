use crate::{
    chunk::{Chunk, Op},
    debug::DEBUG_PRINT_CODE,
    scanner::{Scanner, Token, TokenType},
    value::Value,
};

#[derive(Clone, Copy)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Default for Precedence {
    fn default() -> Self {
        Precedence::None
    }
}

impl TryFrom<usize> for Precedence {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use Precedence::*;
        Ok(match value {
            x if x == None as usize => None,
            x if x == Assignment as usize => Assignment,
            x if x == Or as usize => Or,
            x if x == And as usize => And,
            x if x == Equality as usize => Equality,
            x if x == Comparison as usize => Comparison,
            x if x == Term as usize => Term,
            x if x == Factor as usize => Factor,
            x if x == Unary as usize => Unary,
            x if x == Call as usize => Call,
            x if x == Primary as usize => Primary,
            _ => return Err(()),
        })
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Compiler) -> ()>,
    infix: Option<fn(&mut Compiler) -> ()>,
    precedence: Precedence,
}

impl Default for ParseRule {
    fn default() -> Self {
        Self {
            prefix: None,
            infix: None,
            precedence: Default::default(),
        }
    }
}

fn make_parse_rule_table() -> Vec<ParseRule> {
    let mut vec = vec![
        (
            TokenType::LeftParen,
            ParseRule {
                prefix: Some(|c| c.grouping()),
                infix: None,
                precedence: Precedence::None,
            },
        ),
        (TokenType::RightParen, ParseRule::default()),
        (TokenType::LeftBrace, ParseRule::default()),
        (TokenType::RightBrace, ParseRule::default()),
        (TokenType::Comma, ParseRule::default()),
        (TokenType::Dot, ParseRule::default()),
        (
            TokenType::Minus,
            ParseRule {
                prefix: Some(|c| c.unary()),
                infix: Some(|x| x.binary()),
                precedence: Precedence::Term,
            },
        ),
        (
            TokenType::Plus,
            ParseRule {
                prefix: None,
                infix: Some(|c| c.binary()),
                precedence: Precedence::Term,
            },
        ),
        (TokenType::Semicolon, ParseRule::default()),
        (
            TokenType::Slash,
            ParseRule {
                prefix: None,
                infix: Some(|c| c.binary()),
                precedence: Precedence::Factor,
            },
        ),
        (
            TokenType::Star,
            ParseRule {
                prefix: None,
                infix: Some(|c| c.binary()),
                precedence: Precedence::Factor,
            },
        ),
        (TokenType::Bang, ParseRule::default()),
        (TokenType::BangEqual, ParseRule::default()),
        (TokenType::Equal, ParseRule::default()),
        (TokenType::EqualEqual, ParseRule::default()),
        (TokenType::Greater, ParseRule::default()),
        (TokenType::GreaterEqual, ParseRule::default()),
        (TokenType::Less, ParseRule::default()),
        (TokenType::LessEqual, ParseRule::default()),
        (TokenType::Identifier, ParseRule::default()),
        (TokenType::String, ParseRule::default()),
        (
            TokenType::Number,
            ParseRule {
                prefix: Some(|c| c.number()),
                infix: None,
                precedence: Precedence::None,
            },
        ),
        (TokenType::And, ParseRule::default()),
        (TokenType::Class, ParseRule::default()),
        (TokenType::Else, ParseRule::default()),
        (TokenType::False, ParseRule::default()),
        (TokenType::For, ParseRule::default()),
        (TokenType::Fun, ParseRule::default()),
        (TokenType::If, ParseRule::default()),
        (TokenType::Nil, ParseRule::default()),
        (TokenType::Or, ParseRule::default()),
        (TokenType::Print, ParseRule::default()),
        (TokenType::Return, ParseRule::default()),
        (TokenType::Super, ParseRule::default()),
        (TokenType::This, ParseRule::default()),
        (TokenType::True, ParseRule::default()),
        (TokenType::Var, ParseRule::default()),
        (TokenType::While, ParseRule::default()),
        (TokenType::Error, ParseRule::default()),
        (TokenType::Eof, ParseRule::default()),
    ];

    vec.sort_by(|a, b| (a.0 as usize).cmp(&(b.0 as usize)));

    vec.into_iter().map(|(_, rule)| rule).collect()
}

pub struct Compiler<'a> {
    parser: Parser<'a>,
    current_chunk: &'a mut Chunk,
    table: Vec<ParseRule>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str, chunk: &'a mut Chunk) -> Compiler<'a> {
        Compiler {
            parser: Parser::new(Scanner::new(source)),
            current_chunk: chunk,
            table: make_parse_rule_table(),
        }
    }

    pub fn compile(&mut self) -> bool {
        self.expression();
        self.parser
            .consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();
        !self.parser.had_error
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let value = self.parser.previous.unwrap().lexeme.parse::<f64>().unwrap();
        self.emit_constant(Value(value));
    }

    fn unary(&mut self) {
        let operator = self.parser.previous.unwrap().token_type;

        // Compile the operand.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match operator {
            TokenType::Minus => self.emit_op(Op::Negate),
            _ => {}
        };
    }

    fn binary(&mut self) {
        let operator = self.parser.previous.unwrap().token_type;
        let rule = self.get_rule(operator);
        self.parse_precedence((rule.precedence as usize + 1).try_into().unwrap());

        match operator {
            TokenType::Plus => self.emit_op(Op::Add),
            TokenType::Minus => self.emit_op(Op::Subtract),
            TokenType::Star => self.emit_op(Op::Multiply),
            TokenType::Slash => self.emit_op(Op::Divide),
            _ => {}
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.parser
            .consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let prefix_rule = self
            .get_rule(self.parser.previous.unwrap().token_type)
            .prefix;
        match prefix_rule {
            None => {
                self.parser.error("Expect expression.");
                return;
            }
            Some(prefix_rule) => prefix_rule(self),
        }

        loop {
            let rule = self.get_rule(self.parser.current.token_type);

            let rule_has_precedence = precedence as usize <= rule.precedence as usize;
            if !rule_has_precedence {
                return;
            }

            let infix_rule = rule.infix.unwrap();
            self.parser.advance();
            infix_rule(self);
        }
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if DEBUG_PRINT_CODE {
            if !self.parser.had_error {
                self.current_chunk.disassemble("code");
                println!();
            }
        }
    }

    fn emit_return(&mut self) {
        self.emit_op(Op::Return)
    }

    fn emit_op(&mut self, op: Op) {
        self.emit_byte(op.into());
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(Op::Constant.into(), constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk.add_constant(value);
        if constant > std::u8::MAX as usize {
            self.parser.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk
            .write(byte, self.parser.previous.as_ref().unwrap().line)
    }

    fn emit_bytes(&mut self, byte0: u8, byte1: u8) {
        self.emit_byte(byte0);
        self.emit_byte(byte1);
    }

    fn get_rule(&self, token_type: TokenType) -> &ParseRule {
        &self.table[token_type as usize]
    }
}

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token<'a>,
    previous: Option<Token<'a>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    fn new(scanner: Scanner<'a>) -> Parser {
        let mut scanner = scanner;
        let current = scanner.scan_token();
        Parser {
            scanner,
            current,
            previous: None,
            had_error: false,
            panic_mode: false,
        }
    }

    fn advance(&mut self) {
        self.previous = Some(self.current);

        loop {
            self.current = self.scanner.scan_token();
            if TokenType::Error != self.current.token_type {
                break;
            }

            self.error_at_current(self.current.lexeme);
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at_current(&mut self, message: &str) {
        let token = self.current;
        self.error_at(&token, message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.previous.unwrap(), message);
    }

    fn error_at(&mut self, token: &Token<'a>, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.token_type {
            TokenType::Eof => {
                eprint!(" at end");
            }
            TokenType::Error => {
                // Nothing.
            }
            _ => {
                eprint!(" at '{}'", token.lexeme);
            }
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }
}
