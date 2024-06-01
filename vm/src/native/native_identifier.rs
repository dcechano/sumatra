#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct NativeIdentifier {
    class: String,
    method: String,
}

impl NativeIdentifier {
    pub fn new(class: String, method: String) -> Self { Self { class, method } }
}
