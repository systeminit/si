pub struct Secret;

// TODO: actually implement proper secret fetching
impl Secret {
    pub fn all() -> Vec<String> {
        vec!["My Biggest Secret".to_owned(), "Pls No Tell".to_owned()]
    }
}
