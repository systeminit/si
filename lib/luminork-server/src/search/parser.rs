use super::{
    Error,
    Result,
    SearchQuery,
    SearchTerm,
};

/// Parses a search query.
pub fn parse(query_string: &str) -> Result<SearchQuery> {
    let mut parser = SearchQueryParser {
        remaining_query_string: query_string,
    };

    // Parse the top level query; if there are stray end parentheses, ignore them and continue parsing.
    let mut queries = vec![];
    loop {
        // Parse the next subquery (up to the ) or end of string)
        if let Some(query) = parser.parse_subquery() {
            queries.push(query);
        } else if parser.consume(')') {
            // If there is a stray end paren, ignore it and continue parsing more exprssions.
        } else if parser.finished() {
            break;
        } else {
            // This should be impossible, but this avoids an infinite loop if we can't make
            // progress.
            return Err(Error::ParserFailed {
                query_string: query_string.to_string(),
                position: query_string.len() - parser.remaining_query_string.len(),
            });
        }
    }

    // If there are a bunch of conditions with stray parens between, AND them all together.
    if queries.len() > 1 {
        Ok(SearchQuery::And(queries))
    } else {
        Ok(queries.into_iter().next().unwrap_or(SearchQuery::All))
    }
}

/// Holds state for parsing a search query string.
struct SearchQueryParser<'a> {
    remaining_query_string: &'a str,
}

impl<'a> SearchQueryParser<'a> {
    /// Parse a sub-query
    ///
    /// Returns None if we're at ) or end of string
    fn parse_subquery(&mut self) -> Option<SearchQuery> {
        self.parse_or_condition()
    }

    /// Parse an sub-queries separated by "|"
    ///
    /// Consumes |, &/space, !, (), "", value, attr:value
    ///
    /// Returns None if we're at ) or end of string
    fn parse_or_condition(&mut self) -> Option<SearchQuery> {
        let mut queries = vec![];
        loop {
            if let Some(query) = self.parse_and_condition() {
                queries.push(query);
            }

            if !self.consume('|') {
                break;
            }
        }

        // If there are multiple queries, OR them together
        if queries.len() > 1 {
            Some(SearchQuery::Or(queries))
        } else {
            queries.into_iter().next()
        }
    }

    /// Parse sub-queries between & (or spaces between terms)
    ///
    /// Consumes &/space, !, (), "", value, attr:value
    ///
    /// Returns None if we're at |, ) or end of string
    fn parse_and_condition(&mut self) -> Option<SearchQuery> {
        let mut queries = vec![];
        loop {
            if let Some(query) = self.parse_atom() {
                queries.push(query);
            }

            if !(self.consume(' ') || self.consume('&')) {
                break;
            }
        }

        // If there are multiple queries, AND them together
        if queries.len() > 1 {
            Some(SearchQuery::And(queries))
        } else {
            queries.into_iter().next()
        }
    }

    /// Parse a single thing
    ///
    /// Consumes !, (), "", value, attr:value
    ///
    /// Returns None if we're at &/space, |, ), or end of string
    fn parse_atom(&mut self) -> Option<SearchQuery> {
        // Parse parens as a single term (...)
        if self.consume('(') {
            let condition = self.parse_subquery();
            self.consume(')'); // Don't care if it's actually there; end of string closes all parens.
            return condition;
        }

        // Parse !expression
        if self.consume('!') {
            // Skip whitespace (support ! <term>)
            while self.consume(' ') {}
            return self
                .parse_atom()
                .map(|term| SearchQuery::Not(Box::new(term)));
        }

        //
        // Parse value or attr:value
        //
        self.parse_term()
    }

    /// Parse a value or attr:value term
    ///
    /// Consumes "", value, attr:value
    ///
    /// Returns None if we're at &/space, |, ), or end of string
    fn parse_term(&mut self) -> Option<SearchQuery> {
        // Parse quoted term "..."
        if self.consume('"') {
            return Some(SearchQuery::MatchValue(self.parse_quoted_term()));
        }

        // Read everything up to the next special character or : (first word)
        // If this is attr:value:str, this will only read "attr" and we'll read "value:str" next.
        let name_or_value = self
            .consume_until([' ', '(', ')', '&', '|', '!', '"', ':'])
            .to_string();

        // Handle attr:value
        if self.consume(':') {
            // Special case "::"
            // If the user types AWS::EC2::Instance, treat the whole thing as a single term
            // instead of treating it like attr:value (attr="AWS", value=":EC2::Instance"
            // doesn't seem right generally).
            if self.consume(':') {
                let remaining_value = self.consume_until([' ', '(', ')', '&', '|', '!', '"']);
                return Some(SearchQuery::MatchValue(SearchTerm::Match(format!(
                    "{name_or_value}::{remaining_value}"
                ))));
            }

            let name = name_or_value;
            let terms = self.parse_attr_terms();
            return Some(SearchQuery::MatchAttr { name, terms });
        }

        let value = name_or_value;
        if value.is_empty() {
            None
        } else {
            Some(SearchQuery::MatchValue(SearchTerm::Match(value)))
        }
    }

    /// Parse a quoted value
    ///
    /// Consumes until the next quote or end of string
    ///
    /// Returns Exact if there is a closing quote, StartsWith if not.
    fn parse_quoted_term(&mut self) -> SearchTerm {
        // Read until the next quote or end of string
        let value = self.consume_until(['"']).to_string();
        // Parse the final quote.
        if self.consume('"') {
            SearchTerm::Exact(value)
        } else {
            // unclosed quotes should still show the partial match!
            SearchTerm::StartsWith(value)
        }
    }

    /// Parse the attribute values with | or , immediately between them, as part of
    /// attr:value1|value2|value3|...
    ///
    /// Consumes until &/space, (, ), !, or end of string
    fn parse_attr_terms(&mut self) -> Vec<SearchTerm> {
        let mut values = vec![];
        loop {
            if let Some(value) = self.parse_attr_value() {
                values.push(value);
            }

            // If there's a | or ,, continue parsing more values
            if !(self.consume('|') || self.consume(',')) {
                break;
            }
        }
        values
    }

    /// Parse a single attribute value alternative, like value1 in attr:value1|value2|value3|...
    ///
    /// Consumes until &/space, |/,, (, ), !, or end of string
    ///
    /// Returns None if we're at &/space, |/,, ), or end of string
    fn parse_attr_value(&mut self) -> Option<SearchTerm> {
        // If the string starts with ", we treat it as an exact match value (unless it's not closed, read below)
        if self.consume('"') {
            let value = self.consume_until(['"']).to_string();
            // Consume the closing quote so we can move on
            if self.consume('"') {
                Some(SearchTerm::Exact(value))
            } else {
                // If there's no close quote, treat it as a startsWith match to improve UX while typing
                Some(SearchTerm::StartsWith(value))
            }
        } else {
            // No quotes, it's a normal match for the given value
            let value = self.consume_until([' ', '(', ')', '&', '!', '"', '|', ',']);
            if value.is_empty() {
                None
            } else {
                Some(SearchTerm::Match(value.to_string()))
            }
        }
    }

    /// Consume the given char if it's next in the string, returning true if it was.
    fn consume(&mut self, ch: char) -> bool {
        match self.remaining_query_string.strip_prefix(ch) {
            Some(remaining) => {
                self.remaining_query_string = remaining;
                true
            }
            None => false,
        }
    }

    /// Consume all characters until you reach one of the given chars, or end of string.
    /// Does not consume the found char.
    fn consume_until<const N: usize>(&mut self, chars: [char; N]) -> &str {
        let index = self
            .remaining_query_string
            .find(chars)
            .unwrap_or(self.remaining_query_string.len());
        let (consumed, remaining) = self.remaining_query_string.split_at(index);
        self.remaining_query_string = remaining;
        consumed
    }

    /// Whether we're at the end of the query string.
    fn finished(&self) -> bool {
        self.remaining_query_string.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic_in_result_fn)]

    // Test that we can parse the query string "Instance"
    use color_eyre::Result;
    use pretty_assertions_sorted::assert_eq;

    use super::*;

    #[test]
    fn parse_simple() -> Result<()> {
        assert_eq!(
            parse("Instance")?,
            SearchQuery::MatchValue(SearchTerm::Match("Instance".to_string()))
        );
        Ok(())
    }

    #[test]
    fn parse_exact() -> Result<()> {
        assert_eq!(
            parse("\"Instance\"")?,
            SearchQuery::MatchValue(SearchTerm::Exact("Instance".to_string()))
        );
        Ok(())
    }

    #[test]
    fn parse_starts_with() -> Result<()> {
        assert_eq!(
            parse("\"Instance")?,
            SearchQuery::MatchValue(SearchTerm::StartsWith("Instance".to_string()))
        );
        Ok(())
    }

    #[test]
    fn parse_exact_without_spaces() -> Result<()> {
        assert_eq!(
            parse("ab\"cd\"ef\"gh")?,
            SearchQuery::And(vec![
                SearchQuery::MatchValue(SearchTerm::Match("ab".to_string())),
                SearchQuery::MatchValue(SearchTerm::Exact("cd".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("ef".to_string())),
                SearchQuery::MatchValue(SearchTerm::StartsWith("gh".to_string())),
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_attr() -> Result<()> {
        assert_eq!(
            parse("Name:me")?,
            SearchQuery::MatchAttr {
                name: "Name".to_string(),
                terms: vec![SearchTerm::Match("me".to_string())]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_empty_attr() -> Result<()> {
        assert_eq!(
            parse("Name:")?,
            SearchQuery::MatchAttr {
                name: "Name".to_string(),
                terms: vec![]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_attr_multi() -> Result<()> {
        assert_eq!(
            parse("Name:a|b")?,
            SearchQuery::MatchAttr {
                name: "Name".to_string(),
                terms: vec![
                    SearchTerm::Match("a".to_string()),
                    SearchTerm::Match("b".to_string())
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_attr_multi_varied() -> Result<()> {
        assert_eq!(
            parse("Name:a|\"b|c\"|d|e|\"f|g")?,
            SearchQuery::MatchAttr {
                name: "Name".to_string(),
                terms: vec![
                    SearchTerm::Match("a".to_string()),
                    SearchTerm::Exact("b|c".to_string()),
                    SearchTerm::Match("d".to_string()),
                    SearchTerm::Match("e".to_string()),
                    SearchTerm::StartsWith("f|g".to_string()),
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_attr_exact() -> Result<()> {
        assert_eq!(
            parse("Name:\"a|b\"")?,
            SearchQuery::MatchAttr {
                name: "Name".to_string(),
                terms: vec![SearchTerm::Exact("a|b".to_string())]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_attr_starts_with() -> Result<()> {
        assert_eq!(
            parse("Name:\"a|b")?,
            SearchQuery::MatchAttr {
                name: "Name".to_string(),
                terms: vec![SearchTerm::StartsWith("a|b".to_string())]
            }
        );
        Ok(())
    }

    #[test]
    fn parse_double_colon() -> Result<()> {
        assert_eq!(
            parse("AWS::EC2::Instance")?,
            SearchQuery::MatchValue(SearchTerm::Match("AWS::EC2::Instance".to_string()))
        );
        Ok(())
    }

    #[test]
    fn parse_multiple() -> Result<()> {
        assert_eq!(
            parse("ABC D  EF")?,
            SearchQuery::And(vec![
                SearchQuery::MatchValue(SearchTerm::Match("ABC".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("D".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("EF".to_string())),
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_and() -> Result<()> {
        assert_eq!(
            parse("A&B&&C & D& E F &G")?,
            SearchQuery::And(vec![
                SearchQuery::MatchValue(SearchTerm::Match("A".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("B".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("C".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("D".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("E".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("F".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("G".to_string())),
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_or() -> Result<()> {
        assert_eq!(
            parse("A | B|C |D| E")?,
            SearchQuery::Or(vec![
                SearchQuery::MatchValue(SearchTerm::Match("A".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("B".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("C".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("D".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("E".to_string())),
            ]),
        );
        Ok(())
    }

    #[test]
    fn parse_not() -> Result<()> {
        assert_eq!(
            parse("!abc")?,
            SearchQuery::Not(Box::new(SearchQuery::MatchValue(SearchTerm::Match(
                "abc".to_string()
            )))),
        );
        Ok(())
    }

    #[test]
    fn parse_not_with_space() -> Result<()> {
        assert_eq!(
            parse("! abc")?,
            SearchQuery::Not(Box::new(SearchQuery::MatchValue(SearchTerm::Match(
                "abc".to_string()
            )))),
        );
        Ok(())
    }

    #[test]
    fn parse_and_or_precedence() -> Result<()> {
        assert_eq!(
            parse("A&B&&C & D& E F &G | H & I | J")?,
            SearchQuery::Or(vec![
                SearchQuery::And(vec![
                    SearchQuery::MatchValue(SearchTerm::Match("A".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("B".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("C".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("D".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("E".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("F".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("G".to_string())),
                ]),
                SearchQuery::And(vec![
                    SearchQuery::MatchValue(SearchTerm::Match("H".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("I".to_string())),
                ]),
                SearchQuery::MatchValue(SearchTerm::Match("J".to_string())),
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_attr_multi_or_precedence() -> Result<()> {
        assert_eq!(
            parse("a | b | Name:c|d | e | f")?,
            SearchQuery::Or(vec![
                SearchQuery::MatchValue(SearchTerm::Match("a".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("b".to_string())),
                SearchQuery::MatchAttr {
                    name: "Name".to_string(),
                    terms: vec![
                        SearchTerm::Match("c".to_string()),
                        SearchTerm::Match("d".to_string())
                    ]
                },
                SearchQuery::MatchValue(SearchTerm::Match("e".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("f".to_string())),
            ]),
        );
        Ok(())
    }

    #[test]
    fn parse_groupings() -> Result<()> {
        assert_eq!(
            parse("a b (c|d) & (e|f | !(g|h))")?,
            SearchQuery::And(vec![
                SearchQuery::MatchValue(SearchTerm::Match("a".to_string())),
                SearchQuery::MatchValue(SearchTerm::Match("b".to_string())),
                SearchQuery::Or(vec![
                    SearchQuery::MatchValue(SearchTerm::Match("c".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("d".to_string())),
                ]),
                SearchQuery::Or(vec![
                    SearchQuery::MatchValue(SearchTerm::Match("e".to_string())),
                    SearchQuery::MatchValue(SearchTerm::Match("f".to_string())),
                    SearchQuery::Not(Box::new(SearchQuery::Or(vec![
                        SearchQuery::MatchValue(SearchTerm::Match("g".to_string())),
                        SearchQuery::MatchValue(SearchTerm::Match("h".to_string())),
                    ]))),
                ]),
            ])
        );
        Ok(())
    }

    #[test]
    fn parse_empty() -> Result<()> {
        assert_eq!(parse("")?, SearchQuery::All);
        Ok(())
    }

    #[test]
    fn parse_spaces() -> Result<()> {
        assert_eq!(parse("   ")?, SearchQuery::All);
        Ok(())
    }
}
