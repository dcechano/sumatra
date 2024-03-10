use crate::attribute::Attribute;

pub(crate) struct Exception {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
    attributes: Vec<Attribute>,
}
