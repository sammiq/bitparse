#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Unknown(char),
    OpenParen,
    CloseParen,
    UnaryOperator(String),
    Operator(String),
    Number(String),
    Identifier(String),
}

fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.next() {
        match c {
            '(' => {
                tokens.push(Token::OpenParen);
            }
            ')' => {
                tokens.push(Token::CloseParen);
            }
            '<' => {
                if iter.peek() == Some(&'<') {
                    iter.next();
                    tokens.push(Token::Operator("<<".to_owned()));
                }
            }
            '>' => {
                if iter.peek() == Some(&'>') {
                    iter.next();
                    tokens.push(Token::Operator(">>".to_owned()));
                }
            }
            '~' | '!' => {
                tokens.push(Token::UnaryOperator(c.to_string()));
            }
            '^' | '*' | '/' | '%' | '+' | '-' | '|' | '&' => {
                tokens.push(Token::Operator(c.to_string()));
            }
            'x' | 'o' | 'b' => {
                //prefix for data types, then loop
            }
            '0'..='9' => {
                let mut value = c.to_string();
                if c == '0' {
                    //we have a leading zero so check for prefix
                    if let Some(cc) = iter.peek() {
                        match cc {
                            'b' | 'o' | 'x' => {
                                value.push(*cc);
                                iter.next();
                            }
                            _ => {
                                //ignore and process below
                            }
                        }
                    }
                }
                while let Some(cc) = iter.peek() {
                    match cc {
                        '0'..='9' | 'A'..='F' | 'a'..='f' => {
                            //worry about validity during parse
                            value.push(*cc);
                            iter.next();
                        }
                        _ => {
                            //ignore anything and process below
                            break;
                        }
                    }
                }
                tokens.push(Token::Number(value));
            }
            'A'..='Z' | 'a'..='z' => {
                //loop until no longer alphabetic and add the whole thing as identifier
                let mut value = c.to_string();
                while let Some(cc) = iter.peek() {
                    match cc {
                        'A'..='Z' | 'a'..='z' => {
                            value.push(*cc);
                            iter.next();
                        }
                        _ => {
                            //ignore anything and process below
                            break;
                        }
                    }
                }
                tokens.push(Token::Identifier(value));
            }
            ' ' | '\t' | '\n' | '\r' => {
                //skip whitespace
            }
            _ => {
                tokens.push(Token::Unknown(c));
            }
        }
    }
    Ok(tokens)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Operator {
    token: Token,
    precedence: i32,
}

pub fn parse(input: &str) -> Result<u64, String> {
    let tokens = lex(input)?;

    let mut operators = Vec::new();
    let mut operands = Vec::new();
    let mut prev_token = None;
    for token in &tokens {
        match token {
            Token::OpenParen => queue_operator(token, 100, &mut operators, &mut operands)?,
            Token::CloseParen => {
                let mut openned = false;
                while let Some(op) = operators.pop() {
                    if let Token::OpenParen = op.token {
                        openned = true;
                    } else {
                        apply_operator(&op, &mut operands)?;
                    }
                }
                if !openned {
                    return Err("Missing open bracket".into());
                }
            }
            Token::UnaryOperator(_) => queue_operator(token, 1, &mut operators, &mut operands)?,
            Token::Operator(op) => {
                if is_prev_compatible(prev_token) {
                    queue_operator(token, operator_precedence(op), &mut operators, &mut operands)?;
                } else {
                    return Err(format!("Syntax error at token {}", op));
                }
            }
            Token::Number(num_str) => match parse_number(num_str) {
                Ok(num) => operands.push(num),
                Err(_) => return Err(format!("Unrecognised number {}", num_str)),
            },
            Token::Identifier(_) => {
                //push onto function stack
            }
            Token::Unknown(c) => return Err(format!("Unrecognised token {}", c)),
        }
        prev_token = Some(token);
    }

    while let Some(op) = operators.pop() {
        if let Token::OpenParen = op.token {
            return Err("Unexpected open bracket".into());
        }
        apply_operator(&op, &mut operands)?;
    }

    if let Some(result) = operands.pop() {
        Ok(result)
    } else {
        Err("No input".into())
    }
}

fn is_prev_compatible(prev_token: Option<&Token>) -> bool {
    !(prev_token.is_none() || matches!(prev_token, Some(Token::OpenParen)) || matches!(prev_token, Some(Token::Operator(_))))
}

fn parse_number(input: &str) -> Result<u64, std::num::ParseIntError> {
    let mut number = input;
    let radix = if input.starts_with("0x") {
        number = input.trim_start_matches("0x");
        16
    } else if input.starts_with("0o") {
        number = input.trim_start_matches("00");
        8
    } else if input.starts_with("0b") {
        number = input.trim_start_matches("0b");
        2
    } else {
        10
    };

    u64::from_str_radix(number, radix)
}

fn apply_operator(op: &Operator, operands: &mut Vec<u64>) -> Result<(), String> {
    let b = operands.pop().ok_or(format!("not enought operands for {:?}", op.token))?;
    if let Token::UnaryOperator(op_str) = &op.token {
        match op_str.as_str() {
            "~" => operands.push(!b),
            "!" => operands.push((b == 0).into()),
            _ => return Err(format!("Unsupported operator {:?}", op.token)),
        }
    } else if let Token::Operator(op_str) = &op.token {
        let a = operands.pop().ok_or(format!("not enought operands for {:?}", op.token))?;

        match op_str.as_str() {
            "*" => operands.push(a * b),
            "/" => {
                if b == 0 {
                    return Err(format!("Divide by zero '{} / {}'", a, b));
                }
                operands.push(a / b)
            }
            "%" => operands.push(a % b),
            "+" => operands.push(a + b),
            "-" => operands.push(a - b),
            ">>" => operands.push(a >> b),
            "<<" => operands.push(a << b),
            "|" => operands.push(a | b),
            "&" => operands.push(a & b),
            "^" => operands.push(a ^ b),
            _ => return Err(format!("Unsupported operator {:?}", op.token)),
        }
    } else {
        return Err(format!("Unsupported operator {:?}", op.token));
    }

    Ok(())
}

fn operator_precedence(op_str: &str) -> i32 {
    match op_str {
        "*" => 2,
        "/" => 2,
        "%" => 2,
        "+" => 3,
        "-" => 3,
        ">>" => 4,
        "<<" => 4,
        "&" => 5,
        "^" => 5,
        "|" => 6,
        _ => 7,
    }
}

fn queue_operator(token: &Token, precedence: i32, operators: &mut Vec<Operator>, operands: &mut Vec<u64>) -> Result<(), String> {
    while let Some(top_op) = operators.first() {
        if precedence <= top_op.precedence {
            break;
        }
        let stack_op = operators.pop().expect("should have item");
        apply_operator(&stack_op, operands)?;
    }
    operators.push(Operator {
        token: token.clone(),
        precedence,
    });
    Ok(())
}
