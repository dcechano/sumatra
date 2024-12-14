use std::fmt::Debug;

#[derive(Default, Debug)]
pub struct Static;

#[derive(Default, Debug)]
pub struct NonStatic;

#[allow(private_bounds)]
pub(crate) trait AllocType: Sealed {
    fn is_static() -> bool;
}

impl Sealed for Static {}
impl Sealed for NonStatic {}

impl AllocType for Static {
    fn is_static() -> bool { true }
}
impl AllocType for NonStatic {
    fn is_static() -> bool { false }
}

trait Sealed {}
