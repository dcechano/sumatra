use crate::{
    alloc::static_fields::StaticFields, class::Class, data_types::value::Value, result::Result,
};

#[derive(Debug)]
pub(crate) struct StaticData {
    pub class_id: usize,
    pub class: &'static Class,
    fields: &'static mut StaticFields,
}

impl StaticData {
    pub(crate) fn new(
        class_id: usize,
        class: &'static Class,
        fields: &'static mut StaticFields,
    ) -> Self {
        Self {
            class_id,
            class,
            fields,
        }
    }

    pub(crate) fn get_field(&self, name: &str) -> Result<&'static Value> {
        self.fields.get_field(name)
    }

    #[allow(dead_code)]
    pub(crate) fn get_field_mut(&mut self, name: &str) -> Result<&'static mut Value> {
        self.fields.get_field_mut(name)
    }

    pub(crate) fn set_field(&mut self, name: &str, value: Value) -> Result<()> {
        self.fields.set_field(name, value)
    }
}
