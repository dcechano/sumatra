use crate::class::Class;

pub struct InstanceData {
    pub(crate) primary: &'static Class,
    pub(crate) class_id: usize,
    pub(crate) super_classes: Vec<&'static Class>,
}

impl InstanceData {
    pub(crate) fn new(
        primary: &'static Class,
        class_id: usize,
        super_classes: Vec<&'static Class>,
    ) -> Self {
        Self {
            primary,
            class_id,
            super_classes,
        }
    }
}
