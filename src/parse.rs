use log::{debug, trace};

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
                trace!("push open parenthesis");
                tokens.push(Token::OpenParen);
            }
            ')' => {
                trace!("push close parenthesis");
                tokens.push(Token::CloseParen);
            }
            '<' if iter.peek() == Some(&'<') => {
                iter.next();
                trace!("push left shift operator");
                tokens.push(Token::Operator("<<".to_owned()));
            }
            '>' if iter.peek() == Some(&'>') => {
                iter.next();
                trace!("push right shift operator");
                tokens.push(Token::Operator(">>".to_owned()));
            }
            '~' | '!' => {
                trace!("push unary operator {}", c);
                tokens.push(Token::UnaryOperator(c.to_string()));
            }
            '^' | '*' | '/' | '%' | '+' | '-' | '|' | '&' => {
                trace!("push operator {}", c);
                tokens.push(Token::Operator(c.to_string()));
            }
            'x' | 'o' | 'b' => {
                //prefix for data types, then loop
            }
            '0'..='9' => {
                let value = lex_number(&mut iter, c);
                trace!("push number {}", value);
                tokens.push(Token::Number(value));
            }
            'A'..='Z' | 'a'..='z' => {
                let value = lex_identifier(&mut iter, c);
                trace!("push identifier {}", value);
                tokens.push(Token::Identifier(value));
            }
            ' ' | '\t' | '\n' | '\r' => {
                //skip whitespace
            }
            _ => {
                trace!("push unknown {}", c);
                tokens.push(Token::Unknown(c));
            }
        }
    }
    Ok(tokens)
}

fn lex_number(iter: &mut std::iter::Peekable<std::str::Chars<'_>>, c: char) -> String {
    let mut value = c.to_string();
    let mut is_prefixed = false;

    if c == '0' {
        //we have a leading zero so check for prefix
        if let Some(cc) = iter.peek() {
            match cc {
                'b' | 'o' | 'x' => {
                    is_prefixed = true;
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
            '.' => {
                if is_prefixed {
                    break;
                }
                //worry about validity during parse
                value.push(*cc);
                iter.next();
            }
            _ => break,
        }
    }
    value
}

fn lex_identifier(iter: &mut std::iter::Peekable<std::str::Chars<'_>>, c: char) -> String {
    //loop until no longer alphabetic and add the whole thing as identifier
    let mut value = c.to_string();
    while let Some(cc) = iter.peek() {
        match cc {
            'A'..='Z' | 'a'..='z' => {
                value.push(*cc);
                iter.next();
            }
            _ => break,
        }
    }
    value
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
        debug!("processing token {:?}", token);
        match token {
            Token::OpenParen => queue_operator(token, 0, &mut operators, &mut operands)?,
            Token::CloseParen => {
                let mut openned = false;
                while let Some(op) = operators.pop() {
                    debug!("popped operator {:?} from stack", op.token);
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
                Some(num) => {
                    debug!("push operand {} onto stack", num);
                    operands.push(num);
                }
                None => return Err(format!("Unrecognised number {}", num_str)),
            },
            Token::Identifier(_) => {
                //push onto function stack
            }
            Token::Unknown(c) => return Err(format!("Unrecognised token {}", c)),
        }
        prev_token = Some(token);
    }

    while let Some(op) = operators.pop() {
        debug!("popped operator {:?} from stack", op.token);
        if let Token::OpenParen = op.token {
            return Err("Unexpected open bracket".into());
        }
        apply_operator(&op, &mut operands)?;
    }

    if let Some(result) = operands.pop() {
        debug!("popped final operand {} from stack", result);
        Ok(result)
    } else {
        Err("No input".into())
    }
}

fn is_prev_compatible(prev_token: Option<&Token>) -> bool {
    !(prev_token.is_none() || matches!(prev_token, Some(Token::OpenParen)) || matches!(prev_token, Some(Token::Operator(_))))
}

fn parse_number(input: &str) -> Option<u64> {
    if input.contains('.') {
        if input.ends_with("f") {
            let number = input.strip_suffix("f").unwrap_or(input);
            number.parse::<f32>().map(|f| f.to_bits() as u64).ok()
        } else {
            input.parse::<f64>().map(f64::to_bits).ok()
        }
    } else {
        let mut number = input;
        let radix = if input.starts_with("0x") {
            number = input.strip_prefix("0x").unwrap_or(input);
            16
        } else if input.starts_with("0o") {
            number = input.strip_prefix("00").unwrap_or(input);
            8
        } else if input.starts_with("0b") {
            number = input.strip_prefix("0b").unwrap_or(input);
            2
        } else {
            10
        };

        u64::from_str_radix(number, radix).ok()
    }
}

fn apply_operator(op: &Operator, operands: &mut Vec<u64>) -> Result<(), String> {
    let b = operands.pop().ok_or(format!("not enough operands for {:?}", op.token))?;
    debug!("popped operand {} from stack", b);
    if let Token::UnaryOperator(op_str) = &op.token {
        match op_str.as_str() {
            "~" => operands.push(!b),
            "!" => operands.push((b == 0).into()),
            _ => return Err(format!("Unsupported operator {:?}", op.token)),
        }
    } else if let Token::Operator(op_str) = &op.token {
        let a = operands.pop().ok_or(format!("not enough operands for {:?}", op.token))?;
        debug!("popped operand {} from stack", a);
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
    debug!("new operator is {:?} with precedence {}", token, precedence);

    let mut status = Ok(());
    while let Some(top_op) = operators.first()
        && status.is_ok()
    {
        debug!("top operator on stack is {:?} with precedence {}", top_op.token, top_op.precedence);
        if precedence <= top_op.precedence {
            break;
        }
        let stack_op = operators.pop().expect("should have item");
        debug!("popped operator {:?} from stack", stack_op.token);
        status = apply_operator(&stack_op, operands);
    }
    debug!("push operator {:?} onto stack", token);
    operators.push(Operator {
        token: token.clone(),
        precedence,
    });
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_number_from(input: &str) -> (String, String) {
        let mut chars = input.chars().peekable();
        let first = chars.next().expect("test input should not be empty");

        let value = lex_number(&mut chars, first);

        let rest = chars.collect();
        (value, rest)
    }

    #[test]
    fn lex_number_collects_decimal_digits() {
        let (value, rest) = lex_number_from("12345+6");

        assert_eq!(value, "12345");
        assert_eq!(rest, "+6");
    }

    #[test]
    fn lex_number_keeps_base_prefixes_with_leading_zero() {
        for (input, expected) in [("0b1010 | 3", "0b1010"), ("0o755 & 0xff", "0o755"), ("0xDEad + 1", "0xDEad")] {
            let (value, _) = lex_number_from(input);

            assert_eq!(value, expected);
        }
    }

    #[test]
    fn lex_number_only_accepts_prefix_after_leading_zero() {
        let (value, rest) = lex_number_from("10xFF");

        assert_eq!(value, "10");
        assert_eq!(rest, "xFF");
    }

    #[test]
    fn lex_number_stops_before_non_digit_delimiters() {
        for (input, expected_number, expected_rest) in [
            ("42)", "42", ")"),
            ("7<<2", "7", "<<2"),
            ("0b1010_0101", "0b1010", "_0101"),
            ("0xfaceg", "0xface", "g"),
        ] {
            let (value, rest) = lex_number_from(input);

            assert_eq!(value, expected_number);
            assert_eq!(rest, expected_rest);
        }
    }

    #[test]
    fn lex_number_accepts_an_interior_decimal_point() {
        for (input, expected_number, expected_rest) in [("12.34+5", "12.34", "+5"), ("1.0f | 3", "1.0f", " | 3")] {
            let (value, rest) = lex_number_from(input);

            assert_eq!(value, expected_number);
            assert_eq!(rest, expected_rest);
        }
    }

    #[test]
    fn lex_numbers_with_operators_in_expression() {
        let tokens = lex("12.5 + 0xff & 0b1010").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::Number("12.5".to_owned()),
                Token::Operator("+".to_owned()),
                Token::Number("0xff".to_owned()),
                Token::Operator("&".to_owned()),
                Token::Number("0b1010".to_owned()),
            ]
        );
    }
}
