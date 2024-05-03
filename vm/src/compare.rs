#[derive(Debug, Copy, Clone)]
pub(crate) enum Compare {
    Equal,
    NotEqual,
    LessThan,
    GreaterOrEqual,
    GreaterThan,
    LessOrEqual,
}
