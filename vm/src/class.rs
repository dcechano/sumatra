use std::collections::HashMap;

use sumatra_parser::{
    attribute::ClassFileAttributes, class_file::ClassFile, constant_pool::ConstantPool,
    field::Field, flags::ClassAccessFlags, method::Method,
};

#[derive(Debug, Default, Clone)]
pub struct Class {
    pub minor_version: u16,
    pub major_version: u16,
    pub cp: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: usize,
    pub super_class: usize,
    pub interfaces: Vec<usize>,
    pub fields: HashMap<String, Field>,
    pub methods: HashMap<String, Method>,
    pub attributes: ClassFileAttributes,
}

impl From<&ClassFile> for Class {
    fn from(class_file: &ClassFile) -> Self {
        Class {
            minor_version: class_file.minor_version,
            major_version: class_file.major_version,
            cp: class_file.cp.clone(),
            access_flags: class_file.access_flags.clone(),
            this_class: class_file.this_class,
            super_class: class_file.super_class,
            interfaces: class_file.interfaces.clone(),
            fields: fields_map(&class_file.fields),
            methods: methods_map(&class_file.methods.clone()),
            attributes: class_file.attributes.clone(),
        }
    }
}

fn fields_map(fields: &[Field]) -> HashMap<String, Field> {
    let mut f_map = HashMap::with_capacity(fields.len());
    for field in fields {
        let name = field.name.clone();
        let field = field.clone();
        f_map.insert(name, field);
    }
    f_map
}

fn methods_map(methods: &[Method]) -> HashMap<String, Method> {
    let mut m_map = HashMap::with_capacity(methods.len());
    for method in methods {
        // Methods can have the same name as long as they have different descriptors
        // hence the concatenation with descriptor
        let name = format!("{}{}", method.name.clone(), method.descriptor.clone());
        let method = method.clone();
        m_map.insert(name, method);
    }
    m_map
}
