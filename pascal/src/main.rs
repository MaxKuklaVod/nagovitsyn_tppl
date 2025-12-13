use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Integer(i32),
    Id(String),
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
    Assign,
    Semi,
    Dot,
    Begin,
    End,
    Eof,
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn integer(&mut self) -> i32 {
        let mut result = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                result.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        result.parse().unwrap()
    }

    fn id(&mut self) -> Token {
        let mut result = String::new();
        if let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '_' {
                result.push(c);
                self.chars.next();
            }
        }
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                result.push(c);
                self.chars.next();
            } else {
                break;
            }
        }

        match result.to_uppercase().as_str() {
            "BEGIN" => Token::Begin,
            "END" => Token::End,
            _ => Token::Id(result),
        }
    }

    fn get_next_token(&mut self) -> Token {
        self.skip_whitespace();

        let c = match self.chars.peek() {
            Some(&c) => c,
            None => return Token::Eof,
        };

        match c {
            '0'..='9' => Token::Integer(self.integer()),
            'a'..='z' | 'A'..='Z' | '_' => self.id(),
            '+' => { self.chars.next(); Token::Plus },
            '-' => { self.chars.next(); Token::Minus },
            '*' => { self.chars.next(); Token::Mul },
            '/' => { self.chars.next(); Token::Div },
            '(' => { self.chars.next(); Token::LParen },
            ')' => { self.chars.next(); Token::RParen },
            ';' => { self.chars.next(); Token::Semi },
            '.' => { self.chars.next(); Token::Dot },
            ':' => {
                self.chars.next(); 
                if let Some(&next_c) = self.chars.peek() {
                    if next_c == '=' {
                        self.chars.next();
                        return Token::Assign;
                    }
                }
                panic!("Lexer error: Unexpected token ':' expected ':='");
            },
            _ => panic!("Lexer error: Unexpected character: {}", c),
        }
    }
}

#[derive(Debug, Clone)]
enum AST {
    BinOp(Box<AST>, Token, Box<AST>),
    UnaryOp(Token, Box<AST>),
    Num(i32),
    Var(String),
    Assign(String, Box<AST>),
    Compound(Vec<AST>),
    NoOp,
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.get_next_token();
        Parser { lexer, current_token }
    }

    fn eat(&mut self, token_type: Token) {
        let match_ok = match (&self.current_token, &token_type) {
            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Id(_), Token::Id(_)) => true,
            (a, b) => a == b,
        };

        if match_ok {
            self.current_token = self.lexer.get_next_token();
        } else {
            panic!("Parser error: expected {:?}, found {:?}", token_type, self.current_token);
        }
    }

    fn factor(&mut self) -> AST {
        let token = self.current_token.clone();
        match token {
            Token::Plus => {
                self.eat(Token::Plus);
                AST::UnaryOp(Token::Plus, Box::new(self.factor()))
            },
            Token::Minus => {
                self.eat(Token::Minus);
                AST::UnaryOp(Token::Minus, Box::new(self.factor()))
            },
            Token::Integer(value) => {
                self.eat(Token::Integer(0));
                AST::Num(value)
            },
            Token::LParen => {
                self.eat(Token::LParen);
                let node = self.expr();
                self.eat(Token::RParen);
                node
            },
            Token::Id(_) => self.variable(),
            _ => panic!("Parser error in factor: unexpected token {:?}", token),
        }
    }

    fn term(&mut self) -> AST {
        let mut node = self.factor();

        while self.current_token == Token::Mul || self.current_token == Token::Div {
            let token = self.current_token.clone();
            self.eat(token.clone());
            node = AST::BinOp(Box::new(node), token, Box::new(self.factor()));
        }

        node
    }

    fn expr(&mut self) -> AST {
        let mut node = self.term();

        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let token = self.current_token.clone();
            self.eat(token.clone());
            node = AST::BinOp(Box::new(node), token, Box::new(self.term()));
        }

        node
    }

    fn variable(&mut self) -> AST {
        match self.current_token.clone() {
            Token::Id(name) => {
                self.eat(Token::Id(String::new()));
                AST::Var(name)
            },
            _ => panic!("Parser error: Expected identifier, found {:?}", self.current_token),
        }
    }

    fn assignment(&mut self) -> AST {
        let left = self.variable();
        
        if let AST::Var(name) = left {
            self.eat(Token::Assign);
            let right = self.expr();
            AST::Assign(name, Box::new(right))
        } else {
            unreachable!("Assignment always starts with a variable");
        }
    }

    fn statement(&mut self) -> AST {
        match self.current_token {
            Token::Begin => self.compound_statement(),
            Token::Id(_) => self.assignment(),
            _ => AST::NoOp, 
        }
    }

    fn statement_list(&mut self) -> Vec<AST> {
        let mut results = vec![self.statement()];

        while self.current_token == Token::Semi {
            self.eat(Token::Semi);
            results.push(self.statement());
        }

        if let Token::Id(_) = self.current_token {
             panic!("Parser error: Unexpected identifier after statement without semicolon");
        }
        
        results
    }

    fn compound_statement(&mut self) -> AST {
        self.eat(Token::Begin);
        let nodes = self.statement_list();
        self.eat(Token::End);
        AST::Compound(nodes)
    }

    fn program(&mut self) -> AST {
        let node = self.compound_statement();
        self.eat(Token::Dot);
        node
    }
}

struct Interpreter {
    variables: HashMap<String, i32>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            variables: HashMap::new(),
        }
    }

    fn visit(&mut self, node: &AST) -> i32 {
        match node {
            AST::Num(val) => *val,
            AST::BinOp(left, op, right) => {
                let l_val = self.visit(left);
                let r_val = self.visit(right);
                match op {
                    Token::Plus => l_val + r_val,
                    Token::Minus => l_val - r_val,
                    Token::Mul => l_val * r_val,
                    Token::Div => l_val / r_val,
                    _ => panic!("Interpreter error: Invalid binary operator"),
                }
            },
            AST::UnaryOp(op, expr) => {
                let val = self.visit(expr);
                match op {
                    Token::Plus => val,
                    Token::Minus => -val,
                    _ => panic!("Interpreter error: Invalid unary operator"),
                }
            },
            AST::Compound(statements) => {
                for stmt in statements {
                    self.visit(stmt);
                }
                0 
            },
            AST::Assign(name, expr) => {
                let val = self.visit(expr);
                self.variables.insert(name.clone(), val);
                0
            },
            AST::Var(name) => {
                *self.variables.get(name).expect(&format!("Interpreter error: Variable '{}' not found", name))
            },
            AST::NoOp => 0,
        }
    }

    fn interpret(&mut self, source: &str) -> HashMap<String, i32> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let tree = parser.program();
        self.visit(&tree);
        self.variables.clone()
    }
}

#[cfg(not(test))]
fn main() {
    let code = "
    BEGIN
        x := 2 + 3 * (2 + 3);
        y := 2 / 2 - 2 + 3 * ((1 + 1) + (1 + 1));
    END.";

    let mut interpreter = Interpreter::new();
    let result = interpreter.interpret(code);
    println!("Execution result:\n{:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let code = "BEGIN END.";
        let mut interpreter = Interpreter::new();
        let vars = interpreter.interpret(code);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_math_expression() {
        let code = "
        BEGIN
            x := 2 + 3 * (2 + 3);
            y := 2 / 2 - 2 + 3 * ((1 + 1) + (1 + 1));
        END.";
        let mut interpreter = Interpreter::new();
        let vars = interpreter.interpret(code);
        
        assert_eq!(*vars.get("x").unwrap(), 17);
        assert_eq!(*vars.get("y").unwrap(), 11);
    }

    #[test]
    fn test_nested_scopes() {
        let code = "
        BEGIN
            y := 2;
            BEGIN
                a := 3;
                a := a;
                b := 10 + a + 10 * y / 4;
                c := a - b
            END;
            x := 11;
        END.";
        let mut interpreter = Interpreter::new();
        let vars = interpreter.interpret(code);

        assert_eq!(*vars.get("y").unwrap(), 2);
        assert_eq!(*vars.get("a").unwrap(), 3);
        assert_eq!(*vars.get("b").unwrap(), 18);
        assert_eq!(*vars.get("c").unwrap(), -15);
        assert_eq!(*vars.get("x").unwrap(), 11);
    }
    
    #[test]
    fn test_unary_operators() {
        let code = "BEGIN x := -5; y := +3; END.";
        let mut interpreter = Interpreter::new();
        let vars = interpreter.interpret(code);
        assert_eq!(*vars.get("x").unwrap(), -5);
        assert_eq!(*vars.get("y").unwrap(), 3);
    }

    #[test]
    fn test_case_insensitivity() {
        let code = "begin X := 10; end.";
        let mut interpreter = Interpreter::new();
        let vars = interpreter.interpret(code);
        assert_eq!(*vars.get("X").unwrap(), 10);
    }

    #[test]
    fn test_noop_semi() {
        let code = "BEGIN x := 1; ; ; END.";
        let mut interpreter = Interpreter::new();
        let vars = interpreter.interpret(code);
        assert_eq!(*vars.get("x").unwrap(), 1);
    }

    #[test]
    #[should_panic(expected = "Lexer error: Unexpected token ':' expected ':='")]
    fn test_lexer_incomplete_assign() {
        let code = "BEGIN x : 10 END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Lexer error: Unexpected character: @")]
    fn test_lexer_invalid_char() {
        let code = "BEGIN x := @; END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Parser error: expected Dot, found Semi")]
    fn test_parser_missing_dot() {
        let code = "BEGIN x := 1 END;";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Parser error: expected End, found Integer(5)")]
    fn test_parser_invalid_variable_assign() {
        let code = "BEGIN 5 := 10 END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }
    
    #[test]
    #[should_panic(expected = "Parser error: expected End, found Integer(1)")]
    fn test_parser_assign_to_expression() {
        let code = "BEGIN 1 := 1 END."; 
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Parser error: expected RParen")]
    fn test_parser_unclosed_paren() {
        let code = "BEGIN x := (2 + 2 END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Parser error in factor: unexpected token Mul")]
    fn test_parser_unexpected_factor() {
        let code = "BEGIN x := 2 + * 3 END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Parser error: Unexpected identifier after statement without semicolon")]
    fn test_parser_missing_semi() {
        let code = "BEGIN x := 1 y := 2 END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Interpreter error: Variable 'z' not found")]
    fn test_interpreter_undefined_var() {
        let code = "BEGIN x := z + 1 END.";
        let mut interpreter = Interpreter::new();
        interpreter.interpret(code);
    }

    #[test]
    #[should_panic(expected = "Interpreter error: Invalid binary operator")]
    fn test_interpreter_invalid_binop() {
        let mut interpreter = Interpreter::new();
        let ast = AST::BinOp(
            Box::new(AST::Num(1)), 
            Token::Begin, 
            Box::new(AST::Num(1))
        );
        interpreter.visit(&ast);
    }

    #[test]
    #[should_panic(expected = "Interpreter error: Invalid unary operator")]
    fn test_interpreter_invalid_unaryop() {
        let mut interpreter = Interpreter::new();
        let ast = AST::UnaryOp(
            Token::Mul, 
            Box::new(AST::Num(1))
        );
        interpreter.visit(&ast);
    }

    #[test]
    #[should_panic(expected = "Parser error: Expected identifier, found Integer(1)")]
    fn test_parser_internal_variable_panic() {
        let lexer = Lexer::new("1");
        let mut parser = Parser::new(lexer);
        parser.variable();
    }
}