use crate::class::Class;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ClassLoader {
    pub(crate) classes: HashMap<String, Class>,
}

impl ClassLoader {
    pub(crate) fn push_class(&mut self, class: Class) {
        self.classes
            .insert(class.attributes.signature.clone(), class);
    }

    pub(crate) fn get_class(&self, name: &str) -> Option<&Class> { self.classes.get(name) }

    pub(crate) fn get_mut_class(&mut self, name: &str) -> Option<&mut Class> {
        self.classes.get_mut(name)
    }
}
