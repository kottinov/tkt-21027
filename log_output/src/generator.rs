use uuid::Uuid;

pub fn generate_string() -> String {
    Uuid::new_v4().to_string()
}
