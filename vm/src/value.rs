#[derive(Debug, Default, Clone)]
pub(crate) enum Value {
    #[default]
    Null,
    Double(f64),
    Float(f32),
    Integer(i32),
    Long(i64),
    Object(String),
}
