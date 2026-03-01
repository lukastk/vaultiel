//! Tokenizer and recursive descent parser for search query strings.
//!
//! Grammar:
//! ```text
//! query      = or_expr
//! or_expr    = and_expr ("OR" and_expr)*
//! and_expr   = unary_expr+
//! unary_expr = "-" atom | atom
//! atom       = field_expr | "(" query ")" | bare_text
//! field_expr = FIELD_PREFIX value_expr
//! value_expr = "(" query ")" | property_value | STRING_MATCHER
//! bare_text  = STRING_MATCHER  (→ Content predicate)
//! ```

use crate::error::VaultError;
use crate::search::types::*;

// ============================================================================
// Tokens
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    /// A plain word (unquoted, no special chars).
    Word(String),
    /// A double-quoted string literal.
    QuotedString(String),
    /// A regex literal: /pattern/
    RegexLiteral(String),
    /// A field prefix like "tag:", "path:", "content:", etc. (includes the colon).
    FieldPrefix(String),
    /// Opening parenthesis.
    OpenParen,
    /// Closing parenthesis.
    CloseParen,
    /// The literal keyword "OR".
    Or,
    /// Negation prefix (a leading `-`).
    Not,
    /// Comparison operator for property values.
    ComparisonOp(String),
}

// ============================================================================
// Tokenizer
// ============================================================================

const KNOWN_FIELDS: &[&str] = &[
    "path", "filename", "tag", "content", "section", "line", "property",
];

fn tokenize(input: &str) -> Result<Vec<Token>, VaultError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        // Skip whitespace
        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        // Parentheses
        if ch == '(' {
            tokens.push(Token::OpenParen);
            i += 1;
            continue;
        }
        if ch == ')' {
            tokens.push(Token::CloseParen);
            i += 1;
            continue;
        }

        // Negation: a leading `-` not followed by whitespace or end
        if ch == '-' && (i + 1 < len && !chars[i + 1].is_whitespace()) {
            tokens.push(Token::Not);
            i += 1;
            continue;
        }

        // Quoted string
        if ch == '"' {
            i += 1;
            let mut s = String::new();
            while i < len && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < len {
                    i += 1;
                    s.push(chars[i]);
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i < len {
                i += 1; // skip closing "
            }
            tokens.push(Token::QuotedString(s));
            continue;
        }

        // Regex literal: /pattern/
        if ch == '/' {
            i += 1;
            let mut pattern = String::new();
            while i < len && chars[i] != '/' {
                if chars[i] == '\\' && i + 1 < len {
                    pattern.push(chars[i]);
                    i += 1;
                    pattern.push(chars[i]);
                } else {
                    pattern.push(chars[i]);
                }
                i += 1;
            }
            if i < len {
                i += 1; // skip closing /
            }
            tokens.push(Token::RegexLiteral(pattern));
            continue;
        }

        // Comparison operators: !=, <=, >=, <, >, =
        if ch == '!' && i + 1 < len && chars[i + 1] == '=' {
            tokens.push(Token::ComparisonOp("!=".to_string()));
            i += 2;
            continue;
        }
        if ch == '<' && i + 1 < len && chars[i + 1] == '=' {
            tokens.push(Token::ComparisonOp("<=".to_string()));
            i += 2;
            continue;
        }
        if ch == '>' && i + 1 < len && chars[i + 1] == '=' {
            tokens.push(Token::ComparisonOp(">=".to_string()));
            i += 2;
            continue;
        }
        if ch == '<' {
            tokens.push(Token::ComparisonOp("<".to_string()));
            i += 1;
            continue;
        }
        if ch == '>' {
            tokens.push(Token::ComparisonOp(">".to_string()));
            i += 1;
            continue;
        }
        if ch == '=' {
            tokens.push(Token::ComparisonOp("=".to_string()));
            i += 1;
            continue;
        }

        // Word (may include field prefix)
        if is_word_char(ch) {
            let start = i;
            while i < len && is_word_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            // Check for "OR" keyword
            if word == "OR" {
                tokens.push(Token::Or);
                continue;
            }

            // Check for field prefix (word followed by colon)
            if i < len && chars[i] == ':' {
                let lower = word.to_lowercase();
                if KNOWN_FIELDS.contains(&lower.as_str()) {
                    i += 1; // consume the colon
                    tokens.push(Token::FieldPrefix(lower));
                    continue;
                }
            }

            tokens.push(Token::Word(word));
            continue;
        }

        // Skip unknown characters
        i += 1;
    }

    Ok(tokens)
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == '/'
}

// ============================================================================
// Parser
// ============================================================================

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), VaultError> {
        match self.advance() {
            Some(ref tok) if tok == expected => Ok(()),
            Some(tok) => Err(VaultError::SearchError(format!(
                "Expected {:?}, got {:?} at position {}",
                expected, tok, self.pos
            ))),
            None => Err(VaultError::SearchError(format!(
                "Expected {:?}, got end of input",
                expected
            ))),
        }
    }

    /// Parse the full query.
    fn parse_query(&mut self) -> Result<SearchQuery, VaultError> {
        let result = self.parse_or_expr()?;
        Ok(result)
    }

    /// or_expr = and_expr ("OR" and_expr)*
    fn parse_or_expr(&mut self) -> Result<SearchQuery, VaultError> {
        let mut children = vec![self.parse_and_expr()?];

        while self.peek() == Some(&Token::Or) {
            self.advance(); // consume OR
            children.push(self.parse_and_expr()?);
        }

        if children.len() == 1 {
            Ok(children.into_iter().next().unwrap())
        } else {
            Ok(SearchQuery::Or { children })
        }
    }

    /// and_expr = unary_expr+
    fn parse_and_expr(&mut self) -> Result<SearchQuery, VaultError> {
        let mut children = vec![self.parse_unary_expr()?];

        // Continue while the next token can start a unary_expr
        // (i.e., not OR, not CloseParen, not end of input)
        while let Some(tok) = self.peek() {
            match tok {
                Token::Or | Token::CloseParen => break,
                _ => children.push(self.parse_unary_expr()?),
            }
        }

        if children.len() == 1 {
            Ok(children.into_iter().next().unwrap())
        } else {
            Ok(SearchQuery::And { children })
        }
    }

    /// unary_expr = "-" atom | atom
    fn parse_unary_expr(&mut self) -> Result<SearchQuery, VaultError> {
        if self.peek() == Some(&Token::Not) {
            self.advance(); // consume -
            let child = self.parse_atom()?;
            Ok(SearchQuery::Not {
                child: Box::new(child),
            })
        } else {
            self.parse_atom()
        }
    }

    /// atom = field_expr | "(" query ")" | bare_text
    fn parse_atom(&mut self) -> Result<SearchQuery, VaultError> {
        match self.peek() {
            Some(Token::FieldPrefix(_)) => self.parse_field_expr(),
            Some(Token::OpenParen) => {
                self.advance(); // consume (
                let query = self.parse_query()?;
                self.expect(&Token::CloseParen)?;
                Ok(query)
            }
            Some(Token::Word(_)) | Some(Token::QuotedString(_)) | Some(Token::RegexLiteral(_)) => {
                let matcher = self.parse_string_matcher()?;
                Ok(SearchQuery::Field(FieldPredicate::Content { matcher }))
            }
            Some(tok) => Err(VaultError::SearchError(format!(
                "Unexpected token {:?} at position {}",
                tok, self.pos
            ))),
            None => Err(VaultError::SearchError(
                "Unexpected end of input".to_string(),
            )),
        }
    }

    /// field_expr = FIELD_PREFIX value_expr
    fn parse_field_expr(&mut self) -> Result<SearchQuery, VaultError> {
        let field = match self.advance() {
            Some(Token::FieldPrefix(f)) => f,
            _ => unreachable!(),
        };

        match field.as_str() {
            "property" => self.parse_property_expr(),
            "tag" => self.parse_tag_expr(),
            "section" => self.parse_scoping_expr("section"),
            "line" => self.parse_scoping_expr("line"),
            "path" => {
                let matcher = self.parse_field_value()?;
                Ok(SearchQuery::Field(FieldPredicate::Path { matcher }))
            }
            "filename" => {
                let matcher = self.parse_field_value()?;
                Ok(SearchQuery::Field(FieldPredicate::Filename { matcher }))
            }
            "content" => {
                let matcher = self.parse_field_value()?;
                Ok(SearchQuery::Field(FieldPredicate::Content { matcher }))
            }
            _ => Err(VaultError::SearchError(format!(
                "Unknown field: {}",
                field
            ))),
        }
    }

    /// Parse property expression: property:key or property:key=value or property:key<value etc.
    fn parse_property_expr(&mut self) -> Result<SearchQuery, VaultError> {
        // Next token should be a word (the property key)
        let key = match self.advance() {
            Some(Token::Word(w)) => w,
            Some(Token::QuotedString(s)) => s,
            Some(tok) => {
                return Err(VaultError::SearchError(format!(
                    "Expected property key, got {:?}",
                    tok
                )))
            }
            None => {
                return Err(VaultError::SearchError(
                    "Expected property key, got end of input".to_string(),
                ))
            }
        };

        // Check for comparison operator
        if let Some(Token::ComparisonOp(_)) = self.peek() {
            let op_str = match self.advance() {
                Some(Token::ComparisonOp(op)) => op,
                _ => unreachable!(),
            };
            let op = match op_str.as_str() {
                "=" => PropertyOp::Eq,
                "!=" => PropertyOp::NotEq,
                "<" => PropertyOp::Lt,
                ">" => PropertyOp::Gt,
                "<=" => PropertyOp::Lte,
                ">=" => PropertyOp::Gte,
                _ => {
                    return Err(VaultError::SearchError(format!(
                        "Unknown operator: {}",
                        op_str
                    )))
                }
            };

            // Parse the value
            let value = match self.advance() {
                Some(Token::Word(w)) => w,
                Some(Token::QuotedString(s)) => s,
                Some(tok) => {
                    return Err(VaultError::SearchError(format!(
                        "Expected property value, got {:?}",
                        tok
                    )))
                }
                None => {
                    return Err(VaultError::SearchError(
                        "Expected property value, got end of input".to_string(),
                    ))
                }
            };

            Ok(SearchQuery::Field(FieldPredicate::Property {
                key,
                op,
                value: Some(value),
            }))
        } else {
            // No operator — just "property:key" means Exists
            Ok(SearchQuery::Field(FieldPredicate::Property {
                key,
                op: PropertyOp::Exists,
                value: None,
            }))
        }
    }

    /// Parse tag expression: tag:value or tag:(value1 OR value2)
    fn parse_tag_expr(&mut self) -> Result<SearchQuery, VaultError> {
        // Check for grouped expression: tag:(x OR y)
        if self.peek() == Some(&Token::OpenParen) {
            self.advance(); // consume (
            let inner = self.parse_tag_group()?;
            self.expect(&Token::CloseParen)?;
            Ok(inner)
        } else {
            let value = match self.advance() {
                Some(Token::Word(w)) => w,
                Some(Token::QuotedString(s)) => s,
                Some(tok) => {
                    return Err(VaultError::SearchError(format!(
                        "Expected tag value, got {:?}",
                        tok
                    )))
                }
                None => {
                    return Err(VaultError::SearchError(
                        "Expected tag value, got end of input".to_string(),
                    ))
                }
            };
            Ok(SearchQuery::Field(FieldPredicate::Tag { value }))
        }
    }

    /// Parse tag group: word ("OR" word)* — each word becomes a Tag predicate
    fn parse_tag_group(&mut self) -> Result<SearchQuery, VaultError> {
        let mut children = Vec::new();
        let first = match self.advance() {
            Some(Token::Word(w)) => w,
            Some(Token::QuotedString(s)) => s,
            Some(tok) => {
                return Err(VaultError::SearchError(format!(
                    "Expected tag value, got {:?}",
                    tok
                )))
            }
            None => {
                return Err(VaultError::SearchError(
                    "Expected tag value, got end of input".to_string(),
                ))
            }
        };
        children.push(SearchQuery::Field(FieldPredicate::Tag { value: first }));

        while self.peek() == Some(&Token::Or) {
            self.advance(); // consume OR
            let next = match self.advance() {
                Some(Token::Word(w)) => w,
                Some(Token::QuotedString(s)) => s,
                Some(tok) => {
                    return Err(VaultError::SearchError(format!(
                        "Expected tag value after OR, got {:?}",
                        tok
                    )))
                }
                None => {
                    return Err(VaultError::SearchError(
                        "Expected tag value after OR, got end of input".to_string(),
                    ))
                }
            };
            children.push(SearchQuery::Field(FieldPredicate::Tag { value: next }));
        }

        if children.len() == 1 {
            Ok(children.into_iter().next().unwrap())
        } else {
            Ok(SearchQuery::Or { children })
        }
    }

    /// Parse scoping expressions: section:(...) or line:(...)
    fn parse_scoping_expr(&mut self, scope: &str) -> Result<SearchQuery, VaultError> {
        // Must be followed by "(" sub-query ")" or a simple string matcher
        if self.peek() == Some(&Token::OpenParen) {
            self.advance(); // consume (
            let sub_query = self.parse_query()?;
            self.expect(&Token::CloseParen)?;
            match scope {
                "section" => Ok(SearchQuery::Field(FieldPredicate::Section {
                    query: Box::new(sub_query),
                })),
                "line" => Ok(SearchQuery::Field(FieldPredicate::Line {
                    query: Box::new(sub_query),
                })),
                _ => unreachable!(),
            }
        } else {
            // Single value: section:"error handling" or section:error
            let matcher = self.parse_string_matcher()?;
            let content_query = SearchQuery::Field(FieldPredicate::Content { matcher });
            match scope {
                "section" => Ok(SearchQuery::Field(FieldPredicate::Section {
                    query: Box::new(content_query),
                })),
                "line" => Ok(SearchQuery::Field(FieldPredicate::Line {
                    query: Box::new(content_query),
                })),
                _ => unreachable!(),
            }
        }
    }

    /// Parse a field value (for path, filename, content fields).
    /// Supports "(" query ")" for grouped expressions or a simple string matcher.
    fn parse_field_value(&mut self) -> Result<StringMatcher, VaultError> {
        self.parse_string_matcher()
    }

    /// Parse a string matcher: quoted string, regex, or word.
    fn parse_string_matcher(&mut self) -> Result<StringMatcher, VaultError> {
        match self.advance() {
            Some(Token::QuotedString(s)) => Ok(StringMatcher::Exact { value: s }),
            Some(Token::RegexLiteral(p)) => Ok(StringMatcher::Regex { pattern: p }),
            Some(Token::Word(w)) => Ok(StringMatcher::Contains { value: w }),
            Some(tok) => Err(VaultError::SearchError(format!(
                "Expected string, quoted string, or regex; got {:?}",
                tok
            ))),
            None => Err(VaultError::SearchError(
                "Expected string value, got end of input".to_string(),
            )),
        }
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Parse a search query string into a `SearchQuery` AST.
pub fn parse_query(input: &str) -> Result<SearchQuery, VaultError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(VaultError::SearchError("Empty search query".to_string()));
    }

    let tokens = tokenize(trimmed)?;
    if tokens.is_empty() {
        return Err(VaultError::SearchError("Empty search query".to_string()));
    }

    let mut parser = Parser::new(tokens);
    let query = parser.parse_query()?;

    // Check for unconsumed tokens
    if parser.pos < parser.tokens.len() {
        return Err(VaultError::SearchError(format!(
            "Unexpected token {:?} at position {}",
            parser.tokens[parser.pos], parser.pos
        )));
    }

    Ok(query)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Tokenizer tests --

    #[test]
    fn test_tokenize_simple_word() {
        let tokens = tokenize("hello").unwrap();
        assert_eq!(tokens, vec![Token::Word("hello".to_string())]);
    }

    #[test]
    fn test_tokenize_quoted_string() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens, vec![Token::QuotedString("hello world".to_string())]);
    }

    #[test]
    fn test_tokenize_regex() {
        let tokens = tokenize("/error\\d+/").unwrap();
        assert_eq!(tokens, vec![Token::RegexLiteral("error\\d+".to_string())]);
    }

    #[test]
    fn test_tokenize_field_prefix() {
        let tokens = tokenize("tag:project").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::FieldPrefix("tag".to_string()),
                Token::Word("project".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_or_keyword() {
        let tokens = tokenize("a OR b").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Word("a".to_string()),
                Token::Or,
                Token::Word("b".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_negation() {
        let tokens = tokenize("-tag:archived").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Not,
                Token::FieldPrefix("tag".to_string()),
                Token::Word("archived".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_parens() {
        let tokens = tokenize("(a OR b)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::OpenParen,
                Token::Word("a".to_string()),
                Token::Or,
                Token::Word("b".to_string()),
                Token::CloseParen,
            ]
        );
    }

    #[test]
    fn test_tokenize_comparison_ops() {
        let tokens = tokenize("property:due<2024-03-01").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::FieldPrefix("property".to_string()),
                Token::Word("due".to_string()),
                Token::ComparisonOp("<".to_string()),
                Token::Word("2024-03-01".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_comparison_lte() {
        let tokens = tokenize("property:x<=5").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::FieldPrefix("property".to_string()),
                Token::Word("x".to_string()),
                Token::ComparisonOp("<=".to_string()),
                Token::Word("5".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_not_eq() {
        let tokens = tokenize("property:status!=done").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::FieldPrefix("property".to_string()),
                Token::Word("status".to_string()),
                Token::ComparisonOp("!=".to_string()),
                Token::Word("done".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_unknown_word_with_colon_not_field() {
        // "unknown" is not a known field, so "unknown:" should be treated as word
        let tokens = tokenize("unknown:value").unwrap();
        assert_eq!(
            tokens,
            vec![
                // "unknown" is not in KNOWN_FIELDS, so it stays a word
                // and then : is skipped as unknown char, then "value" is a word
                Token::Word("unknown".to_string()),
                Token::Word("value".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_escaped_quote() {
        let tokens = tokenize("\"hello \\\"world\\\"\"").unwrap();
        assert_eq!(
            tokens,
            vec![Token::QuotedString("hello \"world\"".to_string())]
        );
    }

    // -- Parser tests --

    #[test]
    fn test_parse_bare_word() {
        let q = parse_query("hello").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Content {
                matcher: StringMatcher::Contains { value },
            }) => assert_eq!(value, "hello"),
            _ => panic!("Expected Content Contains, got {:?}", q),
        }
    }

    #[test]
    fn test_parse_quoted_content() {
        let q = parse_query("\"meeting notes\"").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Content {
                matcher: StringMatcher::Exact { value },
            }) => assert_eq!(value, "meeting notes"),
            _ => panic!("Expected Content Exact, got {:?}", q),
        }
    }

    #[test]
    fn test_parse_regex_content() {
        let q = parse_query("/error\\d+/").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Content {
                matcher: StringMatcher::Regex { pattern },
            }) => assert_eq!(pattern, "error\\d+"),
            _ => panic!("Expected Content Regex, got {:?}", q),
        }
    }

    #[test]
    fn test_parse_implicit_and() {
        let q = parse_query("tag:project path:daily/").unwrap();
        match q {
            SearchQuery::And { children } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    SearchQuery::Field(FieldPredicate::Tag { value }) => {
                        assert_eq!(value, "project")
                    }
                    _ => panic!("Expected Tag"),
                }
                match &children[1] {
                    SearchQuery::Field(FieldPredicate::Path {
                        matcher: StringMatcher::Contains { value },
                    }) => assert_eq!(value, "daily/"),
                    _ => panic!("Expected Path Contains"),
                }
            }
            _ => panic!("Expected And, got {:?}", q),
        }
    }

    #[test]
    fn test_parse_or() {
        let q = parse_query("tag:project OR tag:log").unwrap();
        match q {
            SearchQuery::Or { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Or"),
        }
    }

    #[test]
    fn test_parse_not() {
        let q = parse_query("-tag:archived").unwrap();
        match q {
            SearchQuery::Not { child } => match *child {
                SearchQuery::Field(FieldPredicate::Tag { value }) => {
                    assert_eq!(value, "archived")
                }
                _ => panic!("Expected Tag"),
            },
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_parse_grouped_or_with_and() {
        let q = parse_query("(tag:project OR tag:log) content:\"meeting\"").unwrap();
        match q {
            SearchQuery::And { children } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    SearchQuery::Or { children: or_children } => {
                        assert_eq!(or_children.len(), 2);
                    }
                    _ => panic!("Expected Or as first child"),
                }
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_tag_group() {
        let q = parse_query("tag:(project OR log)").unwrap();
        match q {
            SearchQuery::Or { children } => {
                assert_eq!(children.len(), 2);
                match &children[0] {
                    SearchQuery::Field(FieldPredicate::Tag { value }) => {
                        assert_eq!(value, "project")
                    }
                    _ => panic!("Expected Tag"),
                }
                match &children[1] {
                    SearchQuery::Field(FieldPredicate::Tag { value }) => {
                        assert_eq!(value, "log")
                    }
                    _ => panic!("Expected Tag"),
                }
            }
            _ => panic!("Expected Or"),
        }
    }

    #[test]
    fn test_parse_property_exists() {
        let q = parse_query("property:status").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Property { key, op, value }) => {
                assert_eq!(key, "status");
                assert!(matches!(op, PropertyOp::Exists));
                assert!(value.is_none());
            }
            _ => panic!("Expected Property Exists"),
        }
    }

    #[test]
    fn test_parse_property_eq() {
        let q = parse_query("property:status=active").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Property { key, op, value }) => {
                assert_eq!(key, "status");
                assert!(matches!(op, PropertyOp::Eq));
                assert_eq!(value.as_deref(), Some("active"));
            }
            _ => panic!("Expected Property Eq"),
        }
    }

    #[test]
    fn test_parse_property_lt() {
        let q = parse_query("property:due<2024-03-01").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Property { key, op, value }) => {
                assert_eq!(key, "due");
                assert!(matches!(op, PropertyOp::Lt));
                assert_eq!(value.as_deref(), Some("2024-03-01"));
            }
            _ => panic!("Expected Property Lt"),
        }
    }

    #[test]
    fn test_parse_property_gte() {
        let q = parse_query("property:priority>=5").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Property { key, op, value }) => {
                assert_eq!(key, "priority");
                assert!(matches!(op, PropertyOp::Gte));
                assert_eq!(value.as_deref(), Some("5"));
            }
            _ => panic!("Expected Property Gte"),
        }
    }

    #[test]
    fn test_parse_line_scope() {
        let q = parse_query("line:(TODO deadline)").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Line { query }) => match *query {
                SearchQuery::And { children } => assert_eq!(children.len(), 2),
                _ => panic!("Expected And inside line scope"),
            },
            _ => panic!("Expected Line"),
        }
    }

    #[test]
    fn test_parse_section_scope() {
        let q = parse_query("section:(error handling)").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Section { query }) => match *query {
                SearchQuery::And { children } => assert_eq!(children.len(), 2),
                _ => panic!("Expected And inside section scope"),
            },
            _ => panic!("Expected Section"),
        }
    }

    #[test]
    fn test_parse_complex_query() {
        let q =
            parse_query("(tag:project OR tag:log) content:\"meeting\" -tag:archived").unwrap();
        match q {
            SearchQuery::And { children } => {
                assert_eq!(children.len(), 3);
                assert!(matches!(&children[0], SearchQuery::Or { .. }));
                assert!(matches!(
                    &children[1],
                    SearchQuery::Field(FieldPredicate::Content { .. })
                ));
                assert!(matches!(&children[2], SearchQuery::Not { .. }));
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_empty_query_error() {
        let err = parse_query("").unwrap_err();
        match err {
            VaultError::SearchError(msg) => assert!(msg.contains("Empty")),
            _ => panic!("Expected SearchError"),
        }
    }

    #[test]
    fn test_parse_unbalanced_paren() {
        let err = parse_query("(tag:project").unwrap_err();
        assert!(matches!(err, VaultError::SearchError(_)));
    }

    #[test]
    fn test_parse_filename_field() {
        let q = parse_query("filename:meeting").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Filename {
                matcher: StringMatcher::Contains { value },
            }) => assert_eq!(value, "meeting"),
            _ => panic!("Expected Filename Contains"),
        }
    }

    #[test]
    fn test_parse_path_with_quoted() {
        let q = parse_query("path:\"daily/2024\"").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Path {
                matcher: StringMatcher::Exact { value },
            }) => assert_eq!(value, "daily/2024"),
            _ => panic!("Expected Path Exact"),
        }
    }

    #[test]
    fn test_parse_section_with_simple_value() {
        let q = parse_query("section:\"error handling\"").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Section { query }) => match *query {
                SearchQuery::Field(FieldPredicate::Content {
                    matcher: StringMatcher::Exact { value },
                }) => assert_eq!(value, "error handling"),
                _ => panic!("Expected Content Exact inside section"),
            },
            _ => panic!("Expected Section"),
        }
    }

    #[test]
    fn test_parse_multiple_negations() {
        let q = parse_query("-tag:draft -tag:archived").unwrap();
        match q {
            SearchQuery::And { children } => {
                assert_eq!(children.len(), 2);
                assert!(matches!(&children[0], SearchQuery::Not { .. }));
                assert!(matches!(&children[1], SearchQuery::Not { .. }));
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_property_noteq() {
        let q = parse_query("property:status!=done").unwrap();
        match q {
            SearchQuery::Field(FieldPredicate::Property { key, op, value }) => {
                assert_eq!(key, "status");
                assert!(matches!(op, PropertyOp::NotEq));
                assert_eq!(value.as_deref(), Some("done"));
            }
            _ => panic!("Expected Property NotEq"),
        }
    }
}
