use regex::Regex;

struct ConnectionAnnotation {
    tokens: Vec<String>,
}

impl TryFrom<String> for ConnectionAnnotation {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // A connection annotation is composed by a series of pairs, with the following recursive
        // structure:
        // PAIR ::= TOKEN<PAIR> | TOKEN
        // where TOKEN is any combination of word characters (\w regex matcher) and single spaces,
        // and < and > are those literal characters
        let mut tokens = vec![];

        let mut this_value = value;
        loop {
            // TODO Remove unwraps
            let re = Regex::new(r"^(?<token>[\w ]+)(?:<(?<tail>.+)>)?$").unwrap();
            let captures = re.captures(&this_value).unwrap();
            let token = captures.name("token").unwrap().as_str();
            tokens.push(token.to_string());

            let maybe_tail = captures.name("tail");
            if let Some(tail) = maybe_tail {
                this_value = tail.as_str().to_string();
            } else {
                break;
            }
        }

        Ok(ConnectionAnnotation { tokens })
    }

    // TODO Connection annotation fits another connection annotation
}

#[test]
fn deserialize_connection_annotation() {
    println!("hello");

    let cases = vec![
        ("arn", vec!["arn"]),
        ("arn<string>", vec!["arn", "string"]),
        ("userArn<arn<string>>", vec!["userArn", "arn", "string"]),
    ];

    for (raw, tokens) in cases {
        let ca =
            ConnectionAnnotation::try_from(raw.to_string()).expect("parse connection annotation");
        assert_eq!(ca.tokens, tokens)
    }
}
