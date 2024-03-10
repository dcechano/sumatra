use crate::class_file::ClassFile;
use crate::exception::Exception;
use crate::instruction::Instruction;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use crate::stack_map::StackMapFrame;

#[derive(Debug)]
pub struct Attribute {
    pub name_index: usize,
    pub info: Vec<u8>,
}

impl Attribute {
    pub(crate) fn parse_attributes(
        attributes_count: usize,
        class_reader: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<Attribute>, String> {
        let mut attributes = Vec::with_capacity(attributes_count);
        for _ in 0..attributes_count {
            let name_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
            let attribute_length = class_reader.read_u32::<BigEndian>().unwrap();
            let info = ClassFile::parse_bytes(attribute_length as usize, class_reader).unwrap();
            attributes.push(Attribute { name_index, info });
        }
        Ok(attributes)
    }
}

/// Attributes are used in the [`crate::class_file::ClassFile`], [`crate::field::Field`],
/// [`crate::method::Method`], Code_attribute, and record_component_info structures
/// of the class file format
pub(crate) enum AttributeInfo {
    ///The [`Attribute::ConstantValue`] attribute is a fixed-length attribute in the
    /// attributes table of a field_info structure. A [`Attribute::ConstantValue`]
    /// attribute represents the value of a constant expression.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.5
    /// https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.29
    ConstantValue {
        attribute_name_index: u16,
        attribute_info: Vec<AttributeInfo>,
    },
    /// The [`AttributeInfo::Code`] attribute is a variable-length attribute in the attributes table
    /// of a [`crate::method::Method`] structure. A [`AttributeInfo::Code`] attribute contains the
    /// Java Virtual Machine instructions and auxiliary information for a method,
    /// including an instance initialization method and a class or interface initialization method.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.9.1
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.9.2
    Code {
        attribute_name_index: u16,
        attribute_length: u32,
        max_stack: u16,
        max_locals: u16,
        code: Vec<Instruction>,
        exception_table: Vec<Exception>,
        attributes: Vec<AttributeInfo>,
    },
    ///The [`AttributeInfo::StackMapTable`] attribute is a variable-length attribute in the
    /// attributes table of a [`AttributeInfo::Code`] attribute. A [`AttributeInfo::StackMapTable`]
    /// attribute is used during the process of verification by type checking.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.3
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.10.1
    StackMapTable {
        attribute_name_index: u16,
        entries: Vec<StackMapFrame>,
    },
    Exceptions {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    InnerClasses {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    EnclosingMethod {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    Synthetic {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    Signature {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    SourceFile {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    SourceDebugExtension {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    LineNumberTable {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    LocalVariableTable {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    LocalVariableTypeTable {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    Deprecated {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    RuntimeVisibleAnnotations {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    RuntimeInvisibleAnnotations {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    RuntimeVisibleParameterAnnotations {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    RuntimeInvisibleParameterAnnotations {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    RuntimeVisibleTypeAnnotations {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    RuntimeInvisibleTypeAnnotations {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    AnnotationDefault {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    BootstrapMethods {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    MethodParameters {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    Module {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    ModulePackages {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    ModuleMainClass {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    NestHost {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    NestMembers {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    Record {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    },
    PermittedSubclasses {
        attribute_name_index: u16,
        attributes: Vec<AttributeInfo>,
    }
}
