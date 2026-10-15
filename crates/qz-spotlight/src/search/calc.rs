//! # Pratt Parser Scientific Calculator Subsystem
//!
//! Evaluates complex mathematical and scientific expressions typed into Spotlight
//! with operator precedence, unary operators, scientific functions, and physical constants.
//!
//! ## Operator Precedence
//! - Exponentiation (`^`): Right-associative, highest binding power.
//! - Multiplication, Division, Modulo (`*`, `/`, `%`): Left-associative.
//! - Addition, Subtraction (`+`, `-`): Left-associative.
//! - Unary prefix (`-`, `+`).

use std::collections::HashMap;
use thiserror::Error;

/// Calculator evaluation errors.
#[derive(Debug, Error, PartialEq)]
pub enum CalcError {
    /// Unexpected end of expression stream.
    #[error("Unexpected end of expression")]
    UnexpectedEof,
    /// Unexpected token encountered during parsing.
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    /// Unbalanced parenthesis.
    #[error("Unbalanced parenthesis")]
    UnbalancedParenthesis,
    /// Unknown function name.
    #[error("Unknown function '{0}'")]
    UnknownFunction(String),
    /// Invalid argument count for function.
    #[error("Function '{0}' expected {1} arguments but received {2}")]
    InvalidArgCount(String, usize, usize),
    /// Division by zero error.
    #[error("Division by zero")]
    DivisionByZero,
    /// Domain error (e.g. square root of negative number or log of non-positive).
    #[error("Mathematical domain error: {0}")]
    DomainError(String),
    /// Invalid number literal.
    #[error("Invalid number literal: {0}")]
    InvalidNumber(String),
}

/// Token stream elements parsed from input text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Floating point literal value.
    Number(f64),
    /// Identifier for functions or constants.
    Ident(String),
    /// Addition operator (`+`).
    Plus,
    /// Subtraction operator (`-`).
    Minus,
    /// Multiplication operator (`*`).
    Star,
    /// Division operator (`/`).
    Slash,
    /// Modulo operator (`%`).
    Percent,
    /// Exponentiation operator (`^`).
    Caret,
    /// Left parenthesis (`(`).
    LParen,
    /// Right parenthesis (`)`).
    RParen,
    /// Argument separator (`,`).
    Comma,
}

/// Lexical tokenizer for mathematical expressions.
pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    /// Creates a new tokenizer over the input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    /// Tokenizes the entire input into a vector of tokens.
    pub fn tokenize(mut self) -> Result<Vec<Token>, CalcError> {
        let mut tokens = Vec::new();

        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
                continue;
            }

            match ch {
                '+' => {
                    self.chars.next();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.chars.next();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.chars.next();
                    tokens.push(Token::Star);
                }
                '/' => {
                    self.chars.next();
                    tokens.push(Token::Slash);
                }
                '%' => {
                    self.chars.next();
                    tokens.push(Token::Percent);
                }
                '^' => {
                    self.chars.next();
                    tokens.push(Token::Caret);
                }
                '(' => {
                    self.chars.next();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.chars.next();
                    tokens.push(Token::RParen);
                }
                ',' => {
                    self.chars.next();
                    tokens.push(Token::Comma);
                }
                '0'..='9' | '.' => {
                    tokens.push(self.read_number()?);
                }
                'a'..='z' | 'A'..='Z' | '_' | 'π' | 'τ' | 'φ' => {
                    tokens.push(self.read_ident());
                }
                _ => {
                    return Err(CalcError::UnexpectedToken(ch.to_string()));
                }
            }
        }

        Ok(tokens)
    }

    fn read_number(&mut self) -> Result<Token, CalcError> {
        let mut num_str = String::new();
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.chars.next();
            } else if ch == '.' && !has_dot && !has_exp {
                has_dot = true;
                num_str.push(ch);
                self.chars.next();
            } else if (ch == 'e' || ch == 'E') && !has_exp {
                has_exp = true;
                num_str.push(ch);
                self.chars.next();
                if let Some(&sign) = self.chars.peek() {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.chars.next();
                    }
                }
            } else {
                break;
            }
        }

        let val = num_str
            .parse::<f64>()
            .map_err(|_| CalcError::InvalidNumber(num_str))?;
        Ok(Token::Number(val))
    }

    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == 'π' || ch == 'τ' || ch == 'φ' {
                ident.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Ident(ident)
    }
}

/// Abstract Syntax Tree node for mathematical expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Constant or literal number value.
    Number(f64),
    /// Unary prefix negation or identity.
    UnaryOp {
        /// Operator character (`+` or `-`).
        op: char,
        /// Child operand.
        expr: Box<AstNode>,
    },
    /// Binary operator with left and right children.
    BinaryOp {
        /// Operator character (`+`, `-`, `*`, `/`, `%`, `^`).
        op: char,
        /// Left-hand child expression.
        left: Box<AstNode>,
        /// Right-hand child expression.
        right: Box<AstNode>,
    },
    /// Function invocation with argument list.
    FunctionCall {
        /// Function name.
        name: String,
        /// Evaluated argument expressions.
        args: Vec<AstNode>,
    },
}

/// Pratt Parser for recursive descent with binding power precedence.
pub struct PrattParser {
    tokens: Vec<Token>,
    cursor: usize,
    constants: HashMap<String, f64>,
}

impl PrattParser {
    /// Creates a parser instance initialized with standard mathematical constants.
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut constants = HashMap::new();
        constants.insert("pi".to_string(), std::f64::consts::PI);
        constants.insert("π".to_string(), std::f64::consts::PI);
        constants.insert("e".to_string(), std::f64::consts::E);
        constants.insert("tau".to_string(), std::f64::consts::TAU);
        constants.insert("τ".to_string(), std::f64::consts::TAU);
        constants.insert("phi".to_string(), 1.618033988749895);
        constants.insert("φ".to_string(), 1.618033988749895);

        Self {
            tokens,
            cursor: 0,
            constants,
        }
    }

    /// Evaluates an expression string directly end-to-end.
    pub fn evaluate_expression(input: &str) -> Result<f64, CalcError> {
        let tokens = Lexer::new(input).tokenize()?;
        if tokens.is_empty() {
            return Err(CalcError::UnexpectedEof);
        }
        let mut parser = Self::new(tokens);
        let ast = parser.parse_expression(0)?;
        if parser.cursor < parser.tokens.len() {
            return Err(CalcError::UnexpectedToken(format!(
                "{:?}",
                parser.tokens[parser.cursor]
            )));
        }
        ast.evaluate()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            Some(tok)
        } else {
            None
        }
    }

    /// Parses an expression using Pratt binding power.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<AstNode, CalcError> {
        let mut left = self.parse_prefix()?;

        while let Some(tok) = self.peek() {
            if let Some((l_bp, r_bp)) = Self::infix_binding_power(tok) {
                if l_bp < min_bp {
                    break;
                }

                let op_tok = self.advance().ok_or(CalcError::UnexpectedEof)?;
                let op = match op_tok {
                    Token::Plus => '+',
                    Token::Minus => '-',
                    Token::Star => '*',
                    Token::Slash => '/',
                    Token::Percent => '%',
                    Token::Caret => '^',
                    _ => unreachable!(),
                };

                let right = self.parse_expression(r_bp)?;
                left = AstNode::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<AstNode, CalcError> {
        let token = self.advance().ok_or(CalcError::UnexpectedEof)?;
        match token {
            Token::Number(val) => Ok(AstNode::Number(val)),
            Token::Plus => {
                let expr = self.parse_expression(7)?;
                Ok(AstNode::UnaryOp {
                    op: '+',
                    expr: Box::new(expr),
                })
            }
            Token::Minus => {
                let expr = self.parse_expression(7)?;
                Ok(AstNode::UnaryOp {
                    op: '-',
                    expr: Box::new(expr),
                })
            }
            Token::LParen => {
                let expr = self.parse_expression(0)?;
                match self.advance() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err(CalcError::UnbalancedParenthesis),
                }
            }
            Token::Ident(ident) => {
                let lower = ident.to_lowercase();
                if let Some(&const_val) = self.constants.get(&lower) {
                    Ok(AstNode::Number(const_val))
                } else if self.peek() == Some(&Token::LParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::RParen) {
                        loop {
                            args.push(self.parse_expression(0)?);
                            if self.peek() == Some(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    match self.advance() {
                        Some(Token::RParen) => Ok(AstNode::FunctionCall {
                            name: lower,
                            args,
                        }),
                        _ => Err(CalcError::UnbalancedParenthesis),
                    }
                } else {
                    Err(CalcError::UnknownFunction(ident))
                }
            }
            other => Err(CalcError::UnexpectedToken(format!("{other:?}"))),
        }
    }

    fn infix_binding_power(tok: &Token) -> Option<(u8, u8)> {
        match tok {
            Token::Plus | Token::Minus => Some((1, 2)),
            Token::Star | Token::Slash | Token::Percent => Some((3, 4)),
            Token::Caret => Some((6, 5)), // Right-associative exponentiation
            _ => None,
        }
    }
}

impl AstNode {
    /// Recursively computes the mathematical value of this AST node.
    pub fn evaluate(&self) -> Result<f64, CalcError> {
        match self {
            AstNode::Number(val) => Ok(*val),
            AstNode::UnaryOp { op, expr } => {
                let v = expr.evaluate()?;
                match op {
                    '+' => Ok(v),
                    '-' => Ok(-v),
                    _ => Err(CalcError::UnexpectedToken(op.to_string())),
                }
            }
            AstNode::BinaryOp { op, left, right } => {
                let l = left.evaluate()?;
                let r = right.evaluate()?;
                match op {
                    '+' => Ok(l + r),
                    '-' => Ok(l - r),
                    '*' => Ok(l * r),
                    '/' => {
                        if r.abs() < f64::EPSILON {
                            Err(CalcError::DivisionByZero)
                        } else {
                            Ok(l / r)
                        }
                    }
                    '%' => {
                        if r.abs() < f64::EPSILON {
                            Err(CalcError::DivisionByZero)
                        } else {
                            Ok(l % r)
                        }
                    }
                    '^' => Ok(l.powf(r)),
                    _ => Err(CalcError::UnexpectedToken(op.to_string())),
                }
            }
            AstNode::FunctionCall { name, args } => {
                Self::evaluate_function(name, args)
            }
        }
    }

    fn evaluate_function(name: &str, args: &[AstNode]) -> Result<f64, CalcError> {
        let eval_arg = |idx: usize| -> Result<f64, CalcError> {
            args.get(idx)
                .ok_or_else(|| CalcError::InvalidArgCount(name.to_string(), 1, args.len()))?
                .evaluate()
        };

        match name {
            "sin" => Ok(eval_arg(0)?.sin()),
            "cos" => Ok(eval_arg(0)?.cos()),
            "tan" => Ok(eval_arg(0)?.tan()),
            "asin" => {
                let v = eval_arg(0)?;
                if !(-1.0..=1.0).contains(&v) {
                    Err(CalcError::DomainError("asin requires value in [-1, 1]".into()))
                } else {
                    Ok(v.asin())
                }
            }
            "acos" => {
                let v = eval_arg(0)?;
                if !(-1.0..=1.0).contains(&v) {
                    Err(CalcError::DomainError("acos requires value in [-1, 1]".into()))
                } else {
                    Ok(v.acos())
                }
            }
            "atan" => Ok(eval_arg(0)?.atan()),
            "sinh" => Ok(eval_arg(0)?.sinh()),
            "cosh" => Ok(eval_arg(0)?.cosh()),
            "tanh" => Ok(eval_arg(0)?.tanh()),
            "sqrt" => {
                let v = eval_arg(0)?;
                if v < 0.0 {
                    Err(CalcError::DomainError("sqrt requires non-negative value".into()))
                } else {
                    Ok(v.sqrt())
                }
            }
            "cbrt" => Ok(eval_arg(0)?.cbrt()),
            "ln" => {
                let v = eval_arg(0)?;
                if v <= 0.0 {
                    Err(CalcError::DomainError("ln requires strictly positive value".into()))
                } else {
                    Ok(v.ln())
                }
            }
            "log" | "log10" => {
                let v = eval_arg(0)?;
                if v <= 0.0 {
                    Err(CalcError::DomainError("log requires strictly positive value".into()))
                } else {
                    Ok(v.log10())
                }
            }
            "log2" => {
                let v = eval_arg(0)?;
                if v <= 0.0 {
                    Err(CalcError::DomainError("log2 requires strictly positive value".into()))
                } else {
                    Ok(v.log2())
                }
            }
            "abs" => Ok(eval_arg(0)?.abs()),
            "ceil" => Ok(eval_arg(0)?.ceil()),
            "floor" => Ok(eval_arg(0)?.floor()),
            "round" => Ok(eval_arg(0)?.round()),
            "exp" => Ok(eval_arg(0)?.exp()),
            "deg_to_rad" | "rad" => Ok(eval_arg(0)?.to_radians()),
            "rad_to_deg" | "deg" => Ok(eval_arg(0)?.to_degrees()),
            "pow" => {
                if args.len() != 2 {
                    return Err(CalcError::InvalidArgCount(name.to_string(), 2, args.len()));
                }
                let base = args[0].evaluate()?;
                let exp = args[1].evaluate()?;
                Ok(base.powf(exp))
            }
            _ => Err(CalcError::UnknownFunction(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic_and_precedence() {
        assert_eq!(PrattParser::evaluate_expression("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(PrattParser::evaluate_expression("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(PrattParser::evaluate_expression("10 - 4 - 2").unwrap(), 4.0);
        assert_eq!(PrattParser::evaluate_expression("24 / 6 / 2").unwrap(), 2.0);
        assert_eq!(PrattParser::evaluate_expression("17 % 5").unwrap(), 2.0);
    }

    #[test]
    fn test_exponentiation_right_associativity() {
        // 2 ^ 3 ^ 2 should be 2 ^ (3 ^ 2) = 2 ^ 9 = 512
        assert_eq!(PrattParser::evaluate_expression("2 ^ 3 ^ 2").unwrap(), 512.0);
    }

    #[test]
    fn test_unary_signs() {
        assert_eq!(PrattParser::evaluate_expression("-5 + 8").unwrap(), 3.0);
        assert_eq!(PrattParser::evaluate_expression("-(3 * 4)").unwrap(), -12.0);
        assert_eq!(PrattParser::evaluate_expression("+10 - -5").unwrap(), 15.0);
    }

    #[test]
    fn test_constants_and_functions() {
        let res = PrattParser::evaluate_expression("sin(pi / 2)").unwrap();
        assert!((res - 1.0).abs() < 1e-10);

        let sqrt_res = PrattParser::evaluate_expression("sqrt(16) + abs(-4)").unwrap();
        assert_eq!(sqrt_res, 8.0);

        let log_res = PrattParser::evaluate_expression("log(100)").unwrap();
        assert!((log_res - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_scientific_notation() {
        assert_eq!(PrattParser::evaluate_expression("1e3 + 2.5e2").unwrap(), 1250.0);
        assert_eq!(PrattParser::evaluate_expression("1e-2 * 100").unwrap(), 1.0);
    }

    #[test]
    fn test_division_by_zero_and_domain_error() {
        assert_eq!(
            PrattParser::evaluate_expression("10 / 0"),
            Err(CalcError::DivisionByZero)
        );
        assert_eq!(
            PrattParser::evaluate_expression("sqrt(-4)"),
            Err(CalcError::DomainError("sqrt requires non-negative value".into()))
        );
    }
}
