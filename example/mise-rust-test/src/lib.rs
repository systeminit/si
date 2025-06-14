pub fn hello_from_mise_rust() -> String {
    "Hello from Rust via mise!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let result = hello_from_mise_rust();
        assert_eq!(result, "Hello from Rust via mise!");
    }
}