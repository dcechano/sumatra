use anyhow::bail;

use crate::{
    annotation::{Annotation, ElementPairs, ParameterAnnotation, TypeAnnotation},
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
    pub(crate) const SOURCE_DEBUG_EXTENSION: &[u8] = b"SourceDebug, Clone, Eq, PartialEqExtension";
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

// TODO Delete Attribute when everything compiles without it.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Attribute;
/// Attributes are used in the [`crate::class_file::ClassFile`],
/// [`crate::field::Field`], [`crate::method::Method`], Code_attribute, and
/// record_component_info structures of the class file format

///The [`Attribute::ConstantValue`] attribute is a fixed-length attribute
/// in the attributes table of a field_info structure. A
/// [`Attribute::ConstantValue`] attribute represents the value of a
/// constant expression. https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.5
/// https://docs.oracle.com/javase/specs/jls/se21/html/jls-15.html#jls-15.29

/// The [`Attribute::StackMapTable`] attribute is a variable-length
/// attribute in the attributes table of a [`Attribute::Code`]
/// attribute. A [`Attribute::StackMapTable`] attribute is used during
/// the process of verification by type checking. https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.3
/// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.10.1

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct ClassFileAttributes {
    pub source_file: SourceFile,
    pub inner_classes: InnerClasses,
    pub enclosing_method: EnclosingMethod,
    pub source_debug_extension: SourceDebugExtension,
    pub synthetic: bool,
    pub signature: String,
    pub bootstrap_methods: BootstrapMethods,
    pub module: Module,
    pub module_packages: ModulePackages,
    pub module_main_class: ModuleMainClass,
    pub nest_host: NestHost,
    pub nest_members: NestMembers,
    pub record: Record,
    pub permitted_subclasses: PermittedSubclasses,
}
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct StackMapTable(pub Vec<StackMapFrame>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Exceptions(pub Vec<u16>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct InnerClasses(pub Vec<InnerClassInfo>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct EnclosingMethod {
    pub class_index: usize,
    pub method_index: usize,
}
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Synthetic;
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Signature(pub u16);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct SourceFile(pub usize);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct SourceDebugExtension(pub Vec<u8>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct LineNumberTable(pub Vec<LineNumberTableEntry>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct LocalVariableTable(pub Vec<LocalVarTableEntry>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct LocalVariableTypeTable(pub Vec<LocalVarTypeEntry>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Deprecated;
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct AnnotationDefault {
    pub attribute_name_index: usize,
    pub attributes: Vec<u8>,
}
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct BootstrapMethods(pub Vec<BootstrapMethod>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct MethodParameters {
    pub attribute_name_index: usize,
    pub attributes: Vec<Attribute>,
}
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Module {
    pub module_name_index: usize,
    pub module_flags: ModuleFlags,
    pub module_ver_index: usize,
    pub requires: Vec<Requires>,
    pub exports: Vec<Exports>,
    pub opens: Vec<Opens>,
    pub uses: Vec<usize>,
    pub provides: Vec<Provides>,
}
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct ModulePackages(pub Vec<usize>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct ModuleMainClass(pub usize);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct NestHost(pub usize);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct NestMembers(pub Vec<usize>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Record(pub Vec<RecordComponent>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct PermittedSubclasses(pub Vec<usize>);
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Custom(pub Vec<u8>);

/// The [`Attribute::Code`] attribute is a variable-length attribute in the
/// attributes table of a [`crate::method::Method`] structure. A
/// [`Attribute::Code`] attribute contains the Java Virtual Machine
/// instructions and auxiliary information for a method, including an
/// instance initialization method and a class or interface initialization
/// method. https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.9.1
/// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html#jvms-2.9.2
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Code {
    pub max_stack: u16,
    pub max_locals: u16,
    pub op_code: Vec<Instruction>,
    pub exception_table: Vec<Exception>,
    pub line_number_table: LineNumberTable,
    pub local_var_table: LocalVariableTable,
    pub stack_map_table: StackMapTable,
    pub local_var_type_table: LocalVariableTypeTable,
    pub attributes: Vec<u8>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Exception {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct LocalVarTableEntry {
    pub start_pc: u16,
    pub len: u16,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub index: usize,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct LocalVarTypeEntry {
    pub start_pc: u16,
    pub len: u16,
    pub name_index: usize,
    pub signature_index: usize,
    pub index: usize,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
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
    #[default]
    Invalid,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct InnerClassInfo {
    pub inner_class_info_index: usize,
    pub outer_class_info_index: usize,
    pub inner_name_index: usize,
    pub inner_class_access_flags: InnerClassAccessFlags,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct BootstrapMethod {
    pub btstr_mthd_ref: u16,
    pub btstr_args: Vec<u16>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Requires {
    pub requires_index: usize,
    pub requires_flags: RequiresFlags,
    pub requires_ver_index: usize,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Exports {
    pub exports_index: usize,
    pub exports_flags: ExportFlags,
    pub exports_to_index: Vec<usize>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Opens {
    pub opens_index: usize,
    pub opens_flags: OpenFlags,
    pub opens_to_index: Vec<u16>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Provides {
    pub provides_index: usize,
    pub provides_with_index: Vec<usize>,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct RecordComponent {
    pub name_index: usize,
    pub descriptor_index: usize,
    pub runtime_annotations: Vec<RuntimeAnnotation>,
}

/// Used to tell parsers if the attributes being parsed are for a
/// Class | Interface | Record.
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub enum ClassType {
    Class,
    Interface,
    Record,
    #[default]
    Invalid,
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub enum RuntimeAnnotation {
    #[default]
    Invalid,
    RuntimeVisibleAnnotations(Vec<Annotation>),
    RuntimeInvisibleAnnotations(Vec<Annotation>),
    RuntimeVisibleParameterAnnotations(Vec<ParameterAnnotation>),
    RuntimeInvisibleParameterAnnotations(Vec<ParameterAnnotation>),
    RuntimeVisibleTypeAnnotations(Vec<TypeAnnotation>),
    RuntimeInvisibleTypeAnnotations(Vec<TypeAnnotation>),
    AnnotationDefault(ElementPairs),
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
