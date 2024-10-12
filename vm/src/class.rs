use std::collections::HashMap;

use crate::data_types::array::ArrayComp;
use sumatra_parser::{
    attribute::ClassFileAttributes, class_file::ClassFile, constant::Constant,
    constant_pool::ConstantPool, desc_types::Primitive, field::Field, flags::ClassAccessFlags,
    method::Method,
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
    array_data: Option<ArrayComp>,
    primitive_class: Option<Primitive>,
}

impl Class {
    pub fn get_name(&self) -> String {
        if let Some(array_comp) = self.array_data.as_ref() {
            let name = match array_comp.root_comp() {
                ArrayComp::Byte => "B",
                ArrayComp::Char => "C",
                ArrayComp::Class(class) => class,
                ArrayComp::Double => "D",
                ArrayComp::Float => "F",
                ArrayComp::Int => "I",
                ArrayComp::Long => "J",
                ArrayComp::Short => "S",
                ArrayComp::Boolean => "Z",
                _ => panic!("Invalid component creating name of array class."),
            };
            return format!("{}{name}", "[".repeat(array_comp.dimension()));
        }

        if let Some(prim) = self.primitive_class.as_ref() {
            return match prim {
                Primitive::Byte => "byte",
                Primitive::Char => "char",
                Primitive::Double => "double",
                Primitive::Float => "float",
                Primitive::Int => "int",
                Primitive::Long => "long",
                Primitive::Short => "short",
                Primitive::Boolean => "boolean",
                Primitive::Invalid => panic!("Invalid primitive variant."),
            }
            .to_string();
        }

        let Constant::Class(index) = self.cp.get(self.this_class).unwrap() else {
            // Should not be possible if the class file is valid.
            panic!("Invalid class file format. this_class index did not point to a Class constant in the constant pool.");
        };

        self.cp.get_utf8(*index).unwrap().to_string()
    }

    pub fn array_class(comp: ArrayComp) -> Self {
        let mut class = Self::default();
        class.array_data = Some(comp);
        class
    }

    pub fn primitive_class(prim: Primitive) -> Self {
        let mut class = Self::default();
        class.primitive_class = Some(prim);
        class
    }

    /// Return true if the class is declared abstract.
    pub fn is_abstract(&self) -> bool { self.access_flags.contains(ClassAccessFlags::FINAL) }

    /// Returns true if the class is an annotation interface
    pub fn is_annotation(&self) -> bool { self.access_flags.contains(ClassAccessFlags::ANNOTATION) }

    /// Returns true if the class is an enum class.
    pub fn is_enum(&self) -> bool { self.access_flags.contains(ClassAccessFlags::ENUM) }

    /// Returns true if the class is declared final.
    pub fn is_final(&self) -> bool { self.access_flags.contains(ClassAccessFlags::FINAL) }

    /// Returns true if the class is an interface, not a class.
    pub fn is_interface(&self) -> bool { self.access_flags.contains(ClassAccessFlags::INTERFACE) }

    /// Returns true if the class is a module, not a class.
    pub fn is_module(&self) -> bool { self.access_flags.contains(ClassAccessFlags::MODULE) }

    /// Returns true if the class is declared public.
    pub fn is_public(&self) -> bool { self.access_flags.contains(ClassAccessFlags::PUBLIC) }

    /// Returns true if the `ACC_SUPER` flag is set. The `ACC_SUPER` flag is a
    /// flag to "treat superclass methods specially when invoked by the
    /// invokespecial instruction."
    pub fn is_super(&self) -> bool { self.access_flags.contains(ClassAccessFlags::SUPER) }

    /// Returns true if the class is synthetic (not declared in the source
    /// code).
    pub fn is_synthetic(&self) -> bool { self.access_flags.contains(ClassAccessFlags::SYNTHETIC) }

    /// Returns the name of this class' superclass
    pub fn superclass(&self) -> String {
        let super_index = self.super_class;
        if super_index == 0 {
            panic!("No superclass on java/lang/Object");
        }

        let Constant::Class(name_index) = self.cp.get(super_index).unwrap() else {
            panic!("Expected an Constant::Class at at superclass index.");
        };
        self.cp.get_utf8(*name_index).unwrap().to_string()
    }

    /// Returns the names of this class' IMMEDIATE interfaces. Does not return
    /// interface names of this class' superclasses.
    pub fn interfaces(&self) -> Vec<String> {
        self.interfaces
            .iter()
            .map(|interface| {
                let Constant::Class(name_index) = self.cp.get(*interface).unwrap() else {
                    panic!("Expected an Constant::Class at an interface index.");
                };
                self.cp.get_utf8(*name_index).unwrap().to_string()
            })
            .collect()
    }

    pub fn is_array_class(&self) -> bool { self.array_data.is_some() }

    pub fn array_class_dim(&self) -> usize {
        if self.array_data.is_none() {
            panic!("Class instance was not an array class.");
        }
        self.array_data.as_ref().unwrap().dimension()
    }

    pub fn is_primitive_class(&self) -> bool { self.primitive_class.is_some() }
}

impl From<&ClassFile> for Class {
    fn from(class_file: &ClassFile) -> Self {
        Self {
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
            array_data: None,
            primitive_class: None,
        }
    }
}

impl From<ClassFile> for Class {
    fn from(class_file: ClassFile) -> Self { Class::from(&class_file) }
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
