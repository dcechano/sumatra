use crate::class_file::ClassFile;
use crate::class_reader::ClassReader;
use crate::constant_pool::ConstantPool;
use crate::exception::Exception;
use crate::instruction::Instruction;
use crate::stack_map::StackMapFrame;

/// Attributes are used in the [`crate::class_file::ClassFile`], [`crate::field::Field`],
/// [`crate::method::Method`], Code_attribute, and record_component_info structures
/// of the class file format
#[derive(Debug)]
pub(crate) enum Attribute {
    ///The [`Attribute::ConstantValue`] attribute is a fixed-length attribute in the
    /// attributes table of a field_info structure. A [`Attribute::ConstantValue`]
    /// attribute represents the value of a constant expression.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.5
    /// https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.29
    ConstantValue {
        attribute_name_index: u16,
        attribute_info: Vec<Attribute>,
    },
    /// The [`Attribute::Code`] attribute is a variable-length attribute in the attributes table
    /// of a [`crate::method::Method`] structure. A [`Attribute::Code`] attribute contains the
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
        attributes: Vec<Attribute>,
    },
    ///The [`Attribute::StackMapTable`] attribute is a variable-length attribute in the
    /// attributes table of a [`Attribute::Code`] attribute. A [`Attribute::StackMapTable`]
    /// attribute is used during the process of verification by type checking.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.3
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.10.1
    StackMapTable {
        attribute_name_index: u16,
        entries: Vec<StackMapFrame>,
    },
    Exceptions {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    InnerClasses {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    EnclosingMethod {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    Synthetic {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    Signature {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    SourceFile {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    SourceDebugExtension {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    LineNumberTable {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    LocalVariableTable {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    LocalVariableTypeTable {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    Deprecated {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    RuntimeVisibleAnnotations {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    RuntimeInvisibleAnnotations {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    RuntimeVisibleParameterAnnotations {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    RuntimeInvisibleParameterAnnotations {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    RuntimeVisibleTypeAnnotations {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    RuntimeInvisibleTypeAnnotations {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    AnnotationDefault {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    BootstrapMethods {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    MethodParameters {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    Module {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    ModulePackages {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    ModuleMainClass {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    NestHost {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    NestMembers {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    Record {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    PermittedSubclasses {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
}

pub(crate) trait AttributeParser {
    fn parse_attr(
        constant_pool: &ConstantPool,
        class_reader: &mut ClassReader,
        count: u16,
    ) -> Result<Vec<Attribute>, String>;
}
