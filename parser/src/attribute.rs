use anyhow::bail;

use crate::{
    annotation::{Annotation, ParameterAnnotations},
    flags::{ExportFlags, InnerClassAccessFlags, ModuleFlags, OpenFlags, RequiresFlags},
    instruction::Instruction,
    type_verification::VType,
};

pub(crate) mod attr_constants {
    pub(crate) const CONSTANT_VALUE: &[u8] = b"ConstantValue";
    pub(crate) const CODE: &[u8] = b"Code";
    pub(crate) const STACK_MAP_TABLE: &[u8] = b"StackMapTable";
    pub(crate) const EXCEPTIONS: &[u8] = b"Exceptions";
    pub(crate) const INNER_CLASSES: &[u8] = b"InnerClasses";
    pub(crate) const ENCLOSING_METHOD: &[u8] = b"EnclosingMethod";
    pub(crate) const NEST_HOST: &[u8] = b"NestHost";
    pub(crate) const NEST_MEMBERS: &[u8] = b"NestMembers";
    pub(crate) const SYNTHETIC: &[u8] = b"Synthetic";
    pub(crate) const SIGNATURE: &[u8] = b"Signature";
    pub(crate) const SOURCE_FILE: &[u8] = b"SourceFile";
    pub(crate) const SOURCE_DEBUG_EXTENSION: &[u8] = b"SourceDebugExtension";
    pub(crate) const LINE_NUMBER_TABLE: &[u8] = b"LineNumberTable";
    pub(crate) const LOCAL_VARIABLE_TABLE: &[u8] = b"LocalVariableTable";
    pub(crate) const LOCAL_VARIABLE_TYPE_TABLE: &[u8] = b"LocalVariableTypeTable";
    pub(crate) const DEPRECATED: &[u8] = b"Deprecated";
    pub(crate) const RUNTIME_VISIBLE_ANNOTATIONS: &[u8] = b"RuntimeVisibleAnnotations";
    pub(crate) const RUNTIME_INVISIBLE_ANNOTATIONS: &[u8] = b"RuntimeInvisibleAnnotations";
    pub(crate) const RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS: &[u8] =
        b"RuntimeVisibleParameterAnnotations";
    pub(crate) const RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS: &[u8] =
        b"RuntimeInvisibleParameterAnnotations";
    pub(crate) const RUNTIME_VISIBLE_TYPE_ANNOTATIONS: &[u8] = b"RuntimeVisibleTypeAnnotations";
    pub(crate) const RUNTIME_INVISIBLE_TYPE_ANNOTATIONS: &[u8] = b"RuntimeInvisibleTypeAnnotations";
    pub(crate) const ANNOTATION_DEFAULT: &[u8] = b"AnnotationDefault";
    pub(crate) const BOOTSTRAP_METHODS: &[u8] = b"BootstrapMethods";
    pub(crate) const METHOD_PARAMETERS: &[u8] = b"MethodParameters";
    pub(crate) const MODULE: &[u8] = b"Module";
    pub(crate) const MODULE_PACKAGES: &[u8] = b"ModulePackages";
    pub(crate) const MODULE_MAIN_CLASS: &[u8] = b"ModuleMainClass";
    pub(crate) const PERMITTED_SUBCLASSES: &[u8] = b"PermittedSubclasses";
    pub(crate) const RECORD: &[u8] = b"Record";
}

/// Attributes are used in the [`crate::class_file::ClassFile`],
/// [`crate::field::Field`], [`crate::method::Method`], Code_attribute, and
/// record_component_info structures of the class file format
#[derive(Debug)]
pub enum Attribute {
    ///The [`Attribute::ConstantValue`] attribute is a fixed-length attribute
    /// in the attributes table of a field_info structure. A
    /// [`Attribute::ConstantValue`] attribute represents the value of a
    /// constant expression. https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.5
    /// https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.29
    ConstantValue {
        attribute_name_index: u16,
        attribute_info: Vec<Attribute>,
    },
    /// The [`Attribute::Code`] attribute is a variable-length attribute in the
    /// attributes table of a [`crate::method::Method`] structure. A
    /// [`Attribute::Code`] attribute contains the Java Virtual Machine
    /// instructions and auxiliary information for a method, including an
    /// instance initialization method and a class or interface initialization
    /// method. https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.9.1
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.9.2
    Code {
        max_stack: u16,
        max_locals: u16,
        code: Vec<Instruction>,
        exception_table: Vec<Exception>,
        attributes: Vec<Attribute>,
    },
    /// The [`Attribute::StackMapTable`] attribute is a variable-length
    /// attribute in the attributes table of a [`Attribute::Code`]
    /// attribute. A [`Attribute::StackMapTable`] attribute is used during
    /// the process of verification by type checking. https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.3
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.10.1
    StackMapTable(Vec<StackMapFrame>),
    Exceptions(Vec<u16>),
    InnerClasses(Vec<InnerClassInfo>),
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },
    Synthetic,
    Signature,
    SourceFile(u16),
    SourceDebugExtension(Vec<u8>),
    LineNumberTable(Vec<LineNumberTableEntry>),
    LocalVariableTable(Vec<LocalVarTableEntry>),
    LocalVariableTypeTable(Vec<LocalVarTypeEntry>),
    Deprecated,
    RuntimeVisibleAnnotations(Vec<Annotation>),
    RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleParameterAnnotations(Vec<ParameterAnnotations>),
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
    BootstrapMethods(Vec<BootstrapMethod>),
    MethodParameters {
        attribute_name_index: u16,
        attributes: Vec<Attribute>,
    },
    Module {
        module_name_index: u16,
        module_flags: ModuleFlags,
        module_ver_index: u16,
        requires: Vec<Requires>,
        exports: Vec<Exports>,
        opens: Vec<Opens>,
        uses: Vec<u16>,
        provides: Vec<Provides>,
    },
    ModulePackages(Vec<u16>),
    ModuleMainClass(u16),
    NestHost(u16),
    NestMembers(Vec<u16>),
    Record(Vec<RecordComponent>),
    PermittedSubclasses(Vec<u16>),
    Custom(Vec<u8>),
}

#[derive(Debug)]
pub struct Exception {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug)]
pub struct LocalVarTableEntry {
    pub start_pc: u16,
    pub len: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

#[derive(Debug)]
pub struct LocalVarTypeEntry {
    pub start_pc: u16,
    pub len: u16,
    pub name_index: u16,
    pub signature_index: u16,
    pub index: u16,
}

#[derive(Debug)]
pub enum StackMapFrame {
    SameFrame,
    SameLocals,
    SameLocalsExt(u16, VType),
    Chop(u16),
    SameFrameExt(u16),
    Append(u16, Vec<VType>),
    Full {
        offset: u16,
        locals: Vec<VType>,
        stack: Vec<VType>,
    },
}

#[derive(Debug)]
pub struct InnerClassInfo {
    pub inner_class_info_index: u16,
    pub outer_class_info_index: u16,
    pub inner_name_index: u16,
    pub inner_class_access_flags: InnerClassAccessFlags,
}

#[derive(Debug)]
pub struct BootstrapMethod {
    pub btstr_mthd_ref: u16,
    pub btstr_args: Vec<u16>,
}

#[derive(Debug)]
pub struct Requires {
    pub requires_index: u16,
    pub requires_flags: RequiresFlags,
    pub requires_ver_index: u16,
}

#[derive(Debug)]
pub struct Exports {
    pub exports_index: u16,
    pub exports_flags: ExportFlags,
    pub exports_to_index: Vec<u16>,
}

#[derive(Debug)]
pub struct Opens {
    pub opens_index: u16,
    pub opens_flags: OpenFlags,
    pub opens_to_index: Vec<u16>,
}

#[derive(Debug)]
pub struct Provides {
    pub provides_index: u16,
    pub provides_with_index: Vec<u16>,
}

#[derive(Debug)]
pub struct RecordComponent {
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

/// Used to tell parsers if the attributes being parsed are for a
/// Class | Interface | Record.
#[derive(Debug)]
pub enum ClassType {
    Class,
    Interface,
    Record,
}

impl TryFrom<u8> for StackMapFrame {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let smf = match value {
            0..=63 => StackMapFrame::SameFrame,
            64..=127 => StackMapFrame::SameLocals,
            247 => StackMapFrame::SameLocalsExt(0, VType::Dummy),
            248..=250 => StackMapFrame::Chop(0),
            251 => StackMapFrame::SameFrameExt(0),
            252..=254 => StackMapFrame::Append(0, vec![]),

            invalid => {
                bail!("Invalid byte {{{invalid}}} for StackMapFrame.");
            }
        };

        Ok(smf)
    }
}
