use uuid::Uuid;

pub(crate) fn uuid_string() -> String {
    Uuid::new_v4().to_string()
}
