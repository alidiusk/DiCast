use crate::dice::{Dice, StdDice};
use thiserror::Error;

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

pub fn parse_str(input: &str) -> Result<(i64, StdDice), ParseError> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Number(i64),
    Times,
    Dice,
    Drop,
    Mul,
    Div,
    Add,
    Sub,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match *self {
            Token::Number(n) => format!("Number({})", n),
            Token::Times => "Times".to_string(),
            Token::Dice => "Dice".to_string(),
            Token::Drop => "Drop".to_string(),
            Token::Mul => "Mul".to_string(),
            Token::Div => "Div".to_string(),
            Token::Add => "Add".to_string(),
            Token::Sub => "Sub".to_string(),
            Token::Eof => "Eof".to_string(),
        };

        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Lexer<'a> {
    pub(self) source: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Lexer {
            source: source.chars().peekable(),
        }
    }

    /// Returns a None if it encounters an invalid token
    /// or the end of the source.
    pub(crate) fn next(&mut self) -> Result<Token, ParseError> {
        let character = self.source.next();

        if character.is_none() {
            return Ok(Token::Eof);
        }

        let character = character.unwrap();

        if character.is_whitespace() {
            return self.next();
        }

        match character {
            '*' => Ok(Token::Mul),
            '/' => Ok(Token::Div),
            '+' => Ok(Token::Add),
            '-' => Ok(Token::Sub),
            'x' => Ok(Token::Times),
            'd' => Ok(Token::Dice),
            's' => Ok(Token::Drop),
            character if character.is_numeric() => {
                let mut number = character.to_string();
                while let Some(c) = self.source.peek() {
                    if c.is_numeric() {
                        number.push(self.source.next().unwrap());
                    } else {
                        break;
                    }
                }

                Ok(Token::Number(number.parse().unwrap()))
            }
            _ => Err(ParseError::InvalidToken(character.to_string())),
        }
    }
}

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        let current = lexer.next()?;

        Ok(Parser { lexer, current })
    }

    /// Returns the dice and the number of times to roll it.
    pub(crate) fn parse(&mut self) -> Result<(i64, StdDice), ParseError> {
        let (times, count) = {
            let n = self.number()?;

            if self.current_token_is(Token::Times) {
                self.next_token()?;
                let count = self.number()?;
                self.expect(Token::Dice)?;
                (n, count)
            } else {
                self.expect(Token::Dice)?;
                (1, n)
            }
        };

        let sides = self.number()?;

        let multiplier = self.parse_multiplier()?.unwrap_or(1);
        let modifier = self.parse_modifier()?.unwrap_or(0);
        let drop = self.parse_drop()?.unwrap_or(0);

        let range = 1..=sides;
        let dice = Dice::new(count, range, multiplier, modifier, drop);

        Ok((times, dice))
    }

    fn next_token(&mut self) -> Result<(), ParseError> {
        self.current = self.lexer.next()?;
        Ok(())
    }

    fn number(&mut self) -> Result<i64, ParseError> {
        if let Token::Number(n) = self.current {
            self.next_token()?;
            Ok(n)
        } else {
            Err(ParseError::UnexpectedToken(
                "Number(n)".to_string(),
                self.current.to_string(),
            ))
        }
    }

    fn parse_multiplier(&mut self) -> Result<Option<i64>, ParseError> {
        match self.current {
            Token::Mul => {
                self.next_token()?;
                let multiplier = self.number()?;
                Ok(Some(multiplier))
            }
            Token::Div => {
                self.next_token()?;
                let multiplier = 1 / self.number()?;
                Ok(Some(multiplier))
            }
            _ => Ok(None),
        }
    }

    fn parse_modifier(&mut self) -> Result<Option<i64>, ParseError> {
        match self.current {
            Token::Add => {
                self.next_token()?;
                let modifier = self.number()?;
                Ok(Some(modifier))
            }
            Token::Sub => {
                self.next_token()?;
                let modifier = -self.number()?;
                Ok(Some(modifier))
            }
            _ => Ok(None),
        }
    }

    fn parse_drop(&mut self) -> Result<Option<i64>, ParseError> {
        if let Token::Drop = self.current {
            self.next_token()?;
            Ok(Some(self.number()?))
        } else {
            Ok(None)
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if expected == self.current {
            self.next_token()?;
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(
                expected.to_string(),
                self.current.to_string(),
            ))
        }
    }

    fn current_token_is(&mut self, token: Token) -> bool {
        token == self.current
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("Encountered invalid token: `{0}`")]
    InvalidToken(String),
    #[error("Expected `{0}`, got `{1}`")]
    UnexpectedToken(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_new() {
        let lexer = Lexer::new("1d10+1");

        assert_eq!("1d10+1".to_string(), lexer.source.collect::<String>());
    }

    #[test]
    fn lexer_next_number() {
        use Token::*;

        let mut lexer = Lexer::new("2");
        assert_eq!(Ok(Number(2)), lexer.next());
        assert_eq!(Ok(Token::Eof), lexer.next());

        let mut lexer = Lexer::new("400");
        assert_eq!(Ok(Number(400)), lexer.next());
        assert_eq!(Ok(Token::Eof), lexer.next());
    }

    #[test]
    fn lexer_next_times() {
        let mut lexer = Lexer::new("x");
        assert_eq!(Ok(Token::Times), lexer.next());
    }

    #[test]
    fn lexer_next_dice() {
        let mut lexer = Lexer::new("d");
        assert_eq!(Ok(Token::Dice), lexer.next());
    }

    #[test]
    fn lexer_next_drop() {
        let mut lexer = Lexer::new("s");
        assert_eq!(Ok(Token::Drop), lexer.next());
    }

    #[test]
    fn lexer_next_mul() {
        let mut lexer = Lexer::new("*");
        assert_eq!(Ok(Token::Mul), lexer.next());
    }

    #[test]
    fn lexer_next_div() {
        let mut lexer = Lexer::new("/");
        assert_eq!(Ok(Token::Div), lexer.next());
    }

    #[test]
    fn lexer_next_add() {
        let mut lexer = Lexer::new("+");
        assert_eq!(Ok(Token::Add), lexer.next());
    }

    #[test]
    fn lexer_next_sub() {
        let mut lexer = Lexer::new("-");
        assert_eq!(Ok(Token::Sub), lexer.next());
    }

    #[test]
    fn lexer_next_all() {
        use Token::*;

        let mut lexer = Lexer::new("3x4d6*5+1s2");

        assert_eq!(Ok(Number(3)), lexer.next());
        assert_eq!(Ok(Times), lexer.next());
        assert_eq!(Ok(Number(4)), lexer.next());
        assert_eq!(Ok(Dice), lexer.next());
        assert_eq!(Ok(Number(6)), lexer.next());
        assert_eq!(Ok(Mul), lexer.next());
        assert_eq!(Ok(Number(5)), lexer.next());
        assert_eq!(Ok(Add), lexer.next());
        assert_eq!(Ok(Number(1)), lexer.next());
        assert_eq!(Ok(Drop), lexer.next());
        assert_eq!(Ok(Number(2)), lexer.next());
        assert_eq!(Ok(Token::Eof), lexer.next());
    }

    #[test]
    fn lexer_ignore_whitespace() {
        use Token::*;

        let mut lexer = Lexer::new(" ");
        assert_eq!(Ok(Token::Eof), lexer.next());

        let mut lexer = Lexer::new("    400 ");
        assert_eq!(Ok(Number(400)), lexer.next());
        assert_eq!(Ok(Token::Eof), lexer.next());
    }

    #[test]
    fn parse_parse_str() {
        let input = "3x4d6*5+1s2";

        let (times, dice) = parse_str(input).unwrap();

        assert_eq!(3, times);
        assert_eq!(4, dice.count);
        assert_eq!(1..=6, dice.range);
        assert_eq!(5, dice.multiplier);
        assert_eq!(1, dice.modifier);
        assert_eq!(2, dice.drop);
    }

    #[test]
    fn parser_parse() {
        let mut parser = Parser::new("3x4d6*5+1s2").unwrap();

        let (times, dice) = parser.parse().unwrap();

        assert_eq!(3, times);
        assert_eq!(4, dice.count);
        assert_eq!(1..=6, dice.range);
        assert_eq!(5, dice.multiplier);
        assert_eq!(1, dice.modifier);
        assert_eq!(2, dice.drop);
    }
}
