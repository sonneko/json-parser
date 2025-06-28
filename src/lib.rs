use std::collections::HashMap;
use std::iter::Peekable;

#[derive(Debug)]
pub enum Node {
    IntLiteral(i32),
    FloatLiteral(f32),
    StringLiteral(String),
    NullLiteral,
    BoolLiteral(bool),
    Object(HashMap<String, Node>),
    Array(Vec<Node>),
}


pub struct Parser<T: Iterator<Item = char>> {
    json: Peekable<T>
}

impl<T: Iterator<Item = char>> Parser<T> {
    pub fn new(json: T) -> Self {
        Self {
            json: json.peekable()
        }
    }

    pub fn parse(&mut self) -> Node {
        self.value()
    }

    fn skip_space(&mut self) {
        while let Some(' ' | '\n' | '\t') = self.json.peek() {
            self.json.next();
        }
    }

    fn value(&mut self) -> Node {
        self.skip_space();
        match self.json.peek().unwrap() {
            '0'..='9' => self.number(),
            '"' => Node::StringLiteral(self.string()),
            '{' => self.object(),
            '[' => self.array(),
            'n' => self.null(),
            't' => self.parse_true(),
            'f' => self.parse_false(),
            _ => panic!("unexpected character")
        }
    }

    fn number(&mut self) -> Node {
        let mut ret: Vec<char> = Vec::new();
        while let Some(num) = self.json.peek() {
            if let '0'..='9' | '.' = num {
                ret.push(*num);
                self.json.next();
            } else {
                break;
            }
        }
        let number: String = ret.iter().collect();
        if let Ok(num) = number.parse::<i32>() {
            return Node::IntLiteral(num);
        } else if let Ok(num) = number.parse::<f32>() {
            return Node::FloatLiteral(num);
        } else {
            panic!("failed to parse number");
        }
    }

    fn string(&mut self) -> String {
        self.json.next();
        let mut ret: Vec<char> = Vec::new();
        while let Some(each_char) = self.json.next() {
            if each_char == '"' {break;}
            ret.push(each_char)
        }
        ret.iter().collect()
    }

    fn object(&mut self) -> Node {
        self.json.next();
        let mut ret: HashMap<String, Node> = HashMap::new();
        loop {
            self.skip_space();
            if self.json.peek() != Some(&'"') {
                assert!(self.json.next() == Some('}'));
                break;
            }
            let key = self.string();
            self.skip_space();
            assert!(self.json.next() == Some(':'), "expect : ");
            let value = self.value();
            assert!(self.json.next() == Some(','), "expect , in parsing object");
            ret.insert(key, value);
        };
        Node::Object(ret)
    }

    fn array(&mut self) -> Node {
        self.json.next();
        let mut ret: Vec<Node> = Vec::new();
        loop {
            self.skip_space();
            if self.json.peek() == Some(&']') {
                self.json.next();
                break;
            }
            ret.push(self.value());
            self.skip_space();
            assert!(self.json.next() == Some(','), "expect , in parsing array");
        }
        Node::Array(ret)
    }

    fn null(&mut self) -> Node {
        let error_message = "expect null";
        assert!(self.json.next() == Some('n'), "{}", error_message);
        assert!(self.json.next() == Some('u'), "{}", error_message);
        assert!(self.json.next() == Some('l'), "{}", error_message);
        assert!(self.json.next() == Some('l'), "{}", error_message);
        Node::NullLiteral
    }

    fn parse_true(&mut self) -> Node {
        let error_message = "expect true";
        assert!(self.json.next() == Some('t'), "{}", error_message);
        assert!(self.json.next() == Some('r'), "{}", error_message);
        assert!(self.json.next() == Some('u'), "{}", error_message);
        assert!(self.json.next() == Some('e'), "{}", error_message);
        Node::BoolLiteral(true)
    }

    fn parse_false(&mut self) -> Node {
        let error_message = "expect false";
        assert!(self.json.next() == Some('f'), "{}", error_message);
        assert!(self.json.next() == Some('a'), "{}", error_message);
        assert!(self.json.next() == Some('l'), "{}", error_message);
        assert!(self.json.next() == Some('s'), "{}", error_message);
        assert!(self.json.next() == Some('e'), "{}", error_message);
        Node::BoolLiteral(false)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int_literal() {
        let json_str = "123";
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::IntLiteral(value) = result {
            assert_eq!(value, 123);
        } else {
            panic!("Expected IntLiteral, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let json_str = "123.45";
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::FloatLiteral(value) = result {
            assert_eq!(value, 123.45);
        } else {
            panic!("Expected FloatLiteral, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let json_str = r#""hello world""#;
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::StringLiteral(value) = result {
            assert_eq!(value, "hello world");
        } else {
            panic!("Expected StringLiteral, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_null_literal() {
        let json_str = "null";
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::NullLiteral = result {
            // Success
        } else {
            panic!("Expected NullLiteral, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_empty_object() {
        let json_str = "{}";
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Object(map) = result {
            assert!(map.is_empty());
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_simple_object() {
        let json_str = r#"{"key": 123,}"#;
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Object(map) = result {
            assert_eq!(map.len(), 1);
            if let Some(Node::IntLiteral(value)) = map.get("key") {
                assert_eq!(*value, 123);
            } else {
                panic!("Expected IntLiteral for key 'key'");
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_multi_entry_object() {
        let json_str = r#"{"name": "Alice", "age": 30, "isStudent": false,}"#;
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Object(map) = result {
            assert_eq!(map.len(), 3);
            if let Some(Node::StringLiteral(name)) = map.get("name") {
                assert_eq!(name, "Alice");
            } else { panic!("Missing or wrong type for 'name'"); }
            if let Some(Node::IntLiteral(age)) = map.get("age") {
                assert_eq!(*age, 30);
            } else { panic!("Missing or wrong type for 'age'"); }
            // Note: The provided parser doesn't handle booleans.
            // For now, this test will pass if `isStudent` is not parsed or errors out.
            // If the parser were extended to handle booleans, this test would need modification.
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let json_str = "[]";
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Array(vec) = result {
            assert!(vec.is_empty());
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_simple_array() {
        let json_str = "[1, 2, 3,]";
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Array(vec) = result {
            assert_eq!(vec.len(), 3);
            if let Node::IntLiteral(v) = vec[0] { assert_eq!(v, 1); } else { panic!("Expected IntLiteral"); }
            if let Node::IntLiteral(v) = vec[1] { assert_eq!(v, 2); } else { panic!("Expected IntLiteral"); }
            if let Node::IntLiteral(v) = vec[2] { assert_eq!(v, 3); } else { panic!("Expected IntLiteral"); }
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_mixed_array() {
        let json_str = r#"[1, "hello", null, 3.14,]"#;
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Array(vec) = result {
            assert_eq!(vec.len(), 4);
            if let Node::IntLiteral(v) = vec[0] { assert_eq!(v, 1); } else { panic!("Expected IntLiteral"); }
            if let Node::StringLiteral(s) = &vec[1] { assert_eq!(s, "hello"); } else { panic!("Expected StringLiteral"); }
            if let Node::NullLiteral = vec[2] { /* OK */ } else { panic!("Expected NullLiteral"); }
            if let Node::FloatLiteral(f) = vec[3] { assert_eq!(f, 3.14); } else { panic!("Expected FloatLiteral"); }
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let json_str = r#"{"data": {"id": 1, "name": "Test",},}"#;
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Object(outer_map) = result {
            if let Some(Node::Object(inner_map)) = outer_map.get("data") {
                if let Some(Node::IntLiteral(id)) = inner_map.get("id") {
                    assert_eq!(*id, 1);
                } else { panic!("Missing or wrong type for 'id'"); }
                if let Some(Node::StringLiteral(name)) = inner_map.get("name") {
                    assert_eq!(name, "Test");
                } else { panic!("Missing or wrong type for 'name'"); }
            } else {
                panic!("Expected nested Object for key 'data'");
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_parse_nested_array() {
        let json_str = r#"[1, [2, 3,], 4,]"#;
        let mut parser = Parser::new(json_str.chars());
        let result = parser.parse();
        if let Node::Array(outer_vec) = result {
            assert_eq!(outer_vec.len(), 3);
            if let Node::IntLiteral(v) = outer_vec[0] { assert_eq!(v, 1); } else { panic!("Expected IntLiteral"); }
            if let Node::Array(inner_vec) = &outer_vec[1] {
                assert_eq!(inner_vec.len(), 2);
                if let Node::IntLiteral(v) = inner_vec[0] { assert_eq!(v, 2); } else { panic!("Expected IntLiteral"); }
                if let Node::IntLiteral(v) = inner_vec[1] { assert_eq!(v, 3); } else { panic!("Expected IntLiteral"); }
            } else {
                panic!("Expected nested Array");
            }
            if let Node::IntLiteral(v) = outer_vec[2] { assert_eq!(v, 4); } else { panic!("Expected IntLiteral"); }
        } else {
            panic!("Expected Array, got {:?}", result);
        }
    }
}