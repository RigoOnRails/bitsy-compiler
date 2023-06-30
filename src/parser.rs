use anyhow::{Result, bail};

use crate::lexer::{Lexer, Token};

/// Represents an operator token.
#[derive(Debug, PartialEq)]
pub enum OperatorToken {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

/// Represents the AST.
#[derive(Debug, PartialEq)]
pub enum ASTNode {
    /// The entrypoint of the program.
    Program(Vec<ASTNode>),

    /// An assignment.
    Assignment(String, Box<ASTNode>),

    /// A binary operation.
    BinaryOperation {
        left: Box<ASTNode>,
        operator: OperatorToken,
        right: Box<ASTNode>,
    },

    /// A variable.
    Variable(String),

    /// A number.
    Number(i32),

    /// Placeholder for unimplemented nodes.
    Todo,
}

/// Parses the tokens into an AST.
pub struct Parser {
    /// The lexer.
    lexer: Lexer,
    /// The current token.
    current_token: Token,
    /// The next token.
    next_token: Option<Token>,
    /// The current block depth.
    block_depth: usize,
}

impl Parser {
    /// Create a new parser from a lexer.
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        // Get the first token.
        let Some(current_token) = lexer.next().transpose()? else {
            bail!("The program is empty.");
        };

        if current_token != Token::Begin {
            bail!("The program must start with `BEGIN`.");
        }

        // Get the second token.
        let next_token = lexer.next().transpose()?;

        Ok(Self {
            lexer,
            current_token,
            next_token,
            block_depth: 1,
        })
    }

    /// Parse the program. Returns the AST.
    pub fn parse(&mut self) -> Result<ASTNode> {
        let program = self.parse_program()?;
        Ok(ASTNode::Program(program))
    }

    /// Parses a `BEGIN ... END` block.
    fn parse_program(&mut self) -> Result<Vec<ASTNode>> {
        let mut program = vec![];
        while !self.completed() {
            self.set_next_token()?;

            // Don't consider `END` as a node.
            if self.current_token != Token::End {
                program.push(ASTNode::Todo);
            }
        }

        // Don't allow instructions after `BEGIN ... END`.
        if self.next_token.is_some() {
            bail!("Can't parse instructions after `BEGIN ... END`.");
        }

        Ok(program)
    }

    /// Sets the next token. Returns an error if an EOF is reached.
    /// Also updates the block depth.
    fn set_next_token(&mut self) -> Result<()> {
        if let Some(next_token) = self.next_token.take() {
            self.current_token = next_token;
            self.next_token = self.lexer.next().transpose()?;
        } else {
            bail!("Unexpected end of file.");
        }

        match self.current_token {
            Token::Begin | Token::IfPositive | Token::IfZero | Token::IfNegative | Token::Loop => {
                self.block_depth += 1
            },
            Token::End => self.block_depth -= 1,
            _ => {},
        }

        Ok(())
    }

    /// Returns true if the parsing is complete.
    fn completed(&self) -> bool {
        self.current_token == Token::End && self.block_depth == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<ASTNode> {
        Parser::new(Lexer::new(input.to_string()))?.parse()
    }

    #[test]
    fn generates_ast_correctly() -> Result<()> {
        assert_eq!(
            parse("
                BEGIN
                    a = 5
                    b = (2 * a) + 5

                    LOOP
                    END

                    LOOP
                    END
                END
            ")?,
            ASTNode::Program(vec![
                ASTNode::Assignment(String::from("a"), Box::new(ASTNode::Number(5))),
                ASTNode::Assignment(
                    String::from("b"),
                    Box::new(ASTNode::BinaryOperation {
                        left: Box::new(ASTNode::BinaryOperation {
                            left: Box::new(ASTNode::Number(2)),
                            operator: OperatorToken::Multiply,
                            right: Box::new(ASTNode::Variable(String::from("a"))),
                        }),
                        operator: OperatorToken::Add,
                        right: Box::new(ASTNode::Number(5)),
                    }),
                ),
                ASTNode::Todo,
                ASTNode::Todo,
            ]),
        );

        Ok(())
    }

    #[test]
    fn handles_empty_program() {
        assert_eq!(parse("").unwrap_err().to_string(), "The program is empty.");
    }

    #[test]
    fn handles_missing_begin() {
        assert_eq!(parse("END").unwrap_err().to_string(), "The program must start with `BEGIN`.");
    }

    #[test]
    fn handles_unexpected_eof() {
        assert_eq!(
            parse("
                BEGIN
                    LOOP
                    END
            ").unwrap_err().to_string(),
            "Unexpected end of file.",
        );
    }

    #[test]
    fn handles_instructions_after_end() {
        assert_eq!(
            parse("
                BEGIN
                    LOOP
                    END
                END

                BEGIN
                END
            ").unwrap_err().to_string(),
            "Can't parse instructions after `BEGIN ... END`.",
        );
    }
}
