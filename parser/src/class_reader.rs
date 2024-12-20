use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::{Cursor, Read, Write},
    ops::{Deref, DerefMut},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
    time::SystemTime,
};

use anyhow::{bail, Context, Result};
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    annotation::{
        Annotation, ElementPairs,
        ElementValue::{self, AnnotationValue, Array, ClassIndex, ConstIndex},
        LocalVarTargetTable, LocalVarTargetTableEntry, ParameterAnnotation, TypeAnnotation,
        TypePath, TypePathEntry,
        TypeTarget::{
            self, Catch, Empty, FormalParameter, LocalVar, Offset, SuperType, Throws, TypeArgument,
            TypeParameter, TypeParameterBound,
        },
    },
    attribute::{
        self,
        attr_constants::{
            ANNOTATION_DEFAULT, BOOTSTRAP_METHODS, CODE, CONSTANT_VALUE, DEPRECATED,
            ENCLOSING_METHOD, EXCEPTIONS, INNER_CLASSES, LINE_NUMBER_TABLE, LOCAL_VARIABLE_TABLE,
            LOCAL_VARIABLE_TYPE_TABLE, METHOD_PARAMETERS, MODULE, MODULE_MAIN_CLASS,
            MODULE_PACKAGES, NEST_HOST, NEST_MEMBERS, PERMITTED_SUBCLASSES, RECORD,
            RUNTIME_INVISIBLE_ANNOTATIONS, RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS,
            RUNTIME_INVISIBLE_TYPE_ANNOTATIONS, RUNTIME_VISIBLE_ANNOTATIONS,
            RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS, RUNTIME_VISIBLE_TYPE_ANNOTATIONS, SIGNATURE,
            SOURCE_DEBUG_EXTENSION, SOURCE_FILE, STACK_MAP_TABLE, SYNTHETIC,
        },
        BootstrapMethod, BootstrapMethods, ClassFileAttributes, Code, EnclosingMethod, Exception,
        Exceptions, Exports, InnerClassInfo, InnerClasses, LineNumberTable, LineNumberTableEntry,
        LocalVarTableEntry, LocalVarTypeEntry, LocalVariableTable, LocalVariableTypeTable,
        MethodParameters, ModuleMainClass, ModulePackages, NestHost, NestMembers, Opens,
        PermittedSubclasses, Provides, Record, RecordComponent, Requires,
        RuntimeAnnotation::{
            self, AnnotationDefault, RuntimeInvisibleAnnotations,
            RuntimeInvisibleParameterAnnotations, RuntimeInvisibleTypeAnnotations,
            RuntimeVisibleAnnotations, RuntimeVisibleParameterAnnotations,
            RuntimeVisibleTypeAnnotations,
        },
        SourceDebugExtension, SourceFile, StackMapFrame, StackMapTable,
    },
    constant::Constant::{
        self, Class, Double, Dummy, Dynamic, FieldRef, Float, Integer, InterfaceMethodRef,
        InvokeDynamic, Long, MethodHandle, MethodRef, MethodType, Module, NameAndType, Package,
        UnRecCharSet, UTF8,
    },
    constant_pool::ConstantPool,
    desc_types::{FieldDescriptor, MethodDescriptor},
    field::Field,
    flags::{
        ExportFlags, FieldAccessFlags, InnerClassAccessFlags, MethodAccessFlags,
        MethodParamAccessFlags, ModuleFlags, OpenFlags, RequiresFlags,
    },
    instruction::Instruction,
    method::Method,
    type_verification::VType,
};

const CLASS_EXT: &[u8] = b"class";
pub(crate) struct ClassReader(Cursor<Vec<u8>>);

impl ClassReader {
    /// Create a new `ClassReader`.
    #[inline]
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let ext = path
            .as_ref()
            .extension()
            .context("Unable to confirm .class file.")?;
        if ext != OsStr::from_bytes(CLASS_EXT) {
            bail!("ClassReader was not provided a .class file.");
        }

        let mut file = File::open(path.as_ref())?;
        let mut buffer = Vec::with_capacity(file.metadata()?.size() as usize);
        file.read_to_end(&mut buffer)?;
        Ok(Self(Cursor::new(buffer)))
    }

    /// Create a `ClassReader` from an already obtained byte buffer.
    #[inline]
    pub(crate) fn with_buffer(buffer: &[u8]) -> Self { Self(Cursor::new(buffer.into())) }

    /// Read `u8`.
    #[inline]
    pub(crate) fn read_u8(&mut self) -> Result<u8> { self.0.read_u8().context("Failed to read u8") }

    /// Read `i8`.
    #[inline]
    pub(crate) fn read_i8(&mut self) -> Result<i8> { self.0.read_i8().context("Failed to read i8") }

    /// Read `u16`.
    #[inline]
    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        self.0.read_u16::<BigEndian>().context("Failed to read u16")
    }

    /// Read `i16`.
    #[inline]
    pub(crate) fn read_i16(&mut self) -> Result<i16> {
        self.0.read_i16::<BigEndian>().context("Failed to read i16")
    }

    /// Read `u32`.
    #[inline]
    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        self.0.read_u32::<BigEndian>().context("Failed to read u32")
    }

    /// Read `i32`.
    #[inline]
    pub(crate) fn read_i32(&mut self) -> Result<i32> {
        self.0.read_i32::<BigEndian>().context("Failed to read i32")
    }

    /// Read `i64`.
    #[inline]
    pub(crate) fn read_i64(&mut self) -> Result<i64> {
        self.0.read_i64::<BigEndian>().context("Failed to read i64")
    }

    /// Read `f32`.
    #[inline]
    pub(crate) fn read_f32(&mut self) -> Result<f32> {
        self.0.read_f32::<BigEndian>().context("Failed to read f32")
    }

    /// Read `f64`.
    #[inline]
    pub(crate) fn read_f64(&mut self) -> Result<f64> {
        self.0.read_f64::<BigEndian>().context("Failed to read f64")
    }
}

impl ClassReader {
    /// Read and interpret the next bytes as the constant pool.
    pub(crate) fn parse_cp(&mut self) -> Result<ConstantPool> {
        let cp_count = self.read_u16()? as usize - 1;
        let mut cp = ConstantPool::new(cp_count);
        cp.push(Dummy);
        let mut i = 0;
        while i < cp_count {
            let constant = match self.read_u8()? {
                1 => {
                    let length = self.read_u16()? as usize;
                    let bytes = self.read_bytes(length)?;
                    match Self::parse_jvm_utf8(&bytes) {
                        Ok(string) => UTF8(string),
                        Err(_) => UnRecCharSet(bytes),
                    }
                }
                3 => Integer(self.read_u32()? as i32),
                4 => Float(self.read_f32()?),
                5 => Long(self.read_i64()?),
                6 => Double(self.read_f64()?),
                7 => Class(self.read_u16()? as usize),
                8 => Constant::String(self.read_u16()? as usize),
                9 => FieldRef {
                    class_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                10 => MethodRef {
                    class_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                11 => InterfaceMethodRef {
                    class_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                12 => NameAndType {
                    name_index: self.read_u16()? as usize,
                    descriptor_index: self.read_u16()? as usize,
                },
                15 => MethodHandle {
                    reference_kind: self.read_u8()?,
                    reference_index: self.read_u16()? as usize,
                },
                16 => MethodType(self.read_u16()? as usize),
                17 => Dynamic {
                    bootstrap_method_attr_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                18 => InvokeDynamic {
                    bootstrap_method_attr_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                19 => Module(self.read_u16()? as usize),
                20 => Package(self.read_u16()? as usize),
                unknown => {
                    bail!("Unknown constant: {unknown}");
                }
            };
            match constant {
                Long(_) | Double(_) => {
                    cp.push(constant);
                    cp.push(Dummy);
                    i += 1;
                }
                _ => {
                    cp.push(constant);
                }
            }
            i += 1;
        }
        Ok(cp)
    }

    /// Read and interpret the next bytes as the class methods.
    pub(crate) fn parse_methods(
        &mut self,
        cp: &ConstantPool,
        method_count: usize,
        min_ver: u16,
        maj_ver: u16,
    ) -> Result<Vec<Method>> {
        let mut methods = Vec::with_capacity(method_count);
        for _ in 0..method_count {
            let method = self.parse_method(cp, min_ver, maj_ver)?;
            methods.push(method);
        }
        Ok(methods)
    }

    /// Read and interpret the next bytes as the method attributes.
    fn parse_method_attr(&mut self, cp: &ConstantPool, method: &mut Method) -> Result<()> {
        let mut sig_present = false;
        let mut exc_present = false;
        let attr_count = self.read_u16()? as usize;

        for _ in 0..attr_count {
            let name_index = self.read_u16()?;
            let name = cp.get_utf8(name_index as usize).context(
                "Method attribute's name_index didn't point to a UTF8 in the constant pool.",
            )?;
            let attr_len = self.read_u32()? as usize;
            // cursor position must always be read after reading attribute length
            let cursor = self.position();
            match name.as_bytes() {
                CODE => method.code = self.parse_code_attr(cp)?,
                EXCEPTIONS => {
                    if exc_present {
                        bail!("Method cannot have more than one Exceptions attribute.");
                    }
                    exc_present = true;
                    let exc_len = self.read_u16()? as usize;
                    let names = self
                        .read_u16s(exc_len)?
                        .iter()
                        .map(|index| {
                            let Some(Constant::Class(name_index)) = cp.get(*index as usize) else {
                                bail!("Expected Constant::Class while parsing Exception table.");
                            };

                            let name = cp.get_utf8(*name_index)?;
                            Ok(name.to_string())
                        })
                        .collect::<Result<Vec<String>>>()?;

                    method.exceptions = Exceptions(names);
                }
                SIGNATURE => {
                    if sig_present {
                        bail!("Method cannot have more than one Signature attribute.");
                    }
                    sig_present = true;
                    let signature = cp.get_utf8(self.read_u16()? as usize)?;
                    method.signature = signature.to_string();
                }
                METHOD_PARAMETERS => {
                    let params_count = self.read_u8()? as usize;
                    method.method_params = (0..params_count)
                        .map(|_| {
                            let name_index = self.read_u16()? as usize;

                            // a name_index of 0 indicates a formal parameter with no name.
                            if name_index != 0 {
                                Self::verify_utf8(
                                    cp,
                                    name_index,
                                    "Failed to validate the name index of a MethodParameter.",
                                )?;
                            }
                            let access_flags = MethodParamAccessFlags::from_bits(self.read_u16()?)
                                .context("Failed to parse MethodParamAccessFlags.")?;
                            Ok(MethodParameters {
                                name_index,
                                access_flags,
                            })
                        })
                        .collect::<Result<Vec<MethodParameters>>>()?;
                }
                DEPRECATED => {
                    if attr_len != 0 {
                        bail!("Synthetic attribute length was not 0");
                    }
                    method.deprecated = true;
                }
                SYNTHETIC => {
                    if attr_len != 0 {
                        bail!("Synthetic attribute length was not 0");
                    }
                    method.synthetic = true;
                }
                name @ (RUNTIME_VISIBLE_ANNOTATIONS
                | RUNTIME_INVISIBLE_ANNOTATIONS
                | RUNTIME_INVISIBLE_TYPE_ANNOTATIONS
                | RUNTIME_VISIBLE_TYPE_ANNOTATIONS) => {
                    method
                        .runtime_annotations
                        .push(self.parse_runtime_anno(cp, name)?);
                }
                unrecognized => {
                    let _ = self.parse_unrec_attr(attr_len, unrecognized, "method")?;
                }
            };
            validate_cursor(self.position(), cursor + attr_len as u64)?;
        }
        Ok(())
    }

    /// Read the next `length` bytes and do not attempt to interpret.
    fn parse_unrec_attr(
        &mut self,
        length: usize,
        bytes: &[u8],
        context: &'static str,
    ) -> Result<Vec<u8>> {
        log_unrec_attr(
            &bytes.iter().map(|b| *b as char).collect::<String>(),
            context,
        )?;

        let bytes = self.read_bytes(length)?;
        Ok(bytes)
    }

    /// Read and interpret the next bytes as the `Code` attribute.
    fn parse_code_attr(&mut self, cp: &ConstantPool) -> Result<Code> {
        let mut code = Code {
            max_stack: self.read_u16()?,
            max_locals: self.read_u16()?,
            ..<_>::default()
        };

        let code_len = self.read_u32()? as usize;
        code.op_code = Instruction::from_bytes(&self.read_bytes(code_len)?)?;

        code.exception_table = self.parse_exceptions_table(cp)?;

        let attrs_len = self.read_u16()? as usize;
        for _ in 0..attrs_len {
            self.parse_code_inner_attr(cp, &mut code)?;
        }

        Ok(code)
    }

    /// Read and interpret the next bytes as the Exceptions table.
    fn parse_exceptions_table(&mut self, cp: &ConstantPool) -> Result<Vec<Exception>> {
        let table_len = self.read_u16()? as usize;
        let table = (0..table_len)
            .map(|_| {
                let start_pc = self.read_u16()?;
                let end_pc = self.read_u16()?;
                let handler_pc = self.read_u16()?;
                let catch_type = self.read_u16()?;
                let class = cp.get(catch_type as usize);
                // if catch_type == 0 then this exception handler is called for all exceptions
                // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.3
                if catch_type != 0 && !matches!(class, Some(Class { .. })) {
                    bail!("Entry in Code attributes exception table didn't point to a Class.");
                }
                Ok(Exception {
                    start_pc,
                    end_pc,
                    handler_pc,
                    catch_type,
                })
            })
            .collect::<Result<Vec<Exception>>>()?;
        Ok(table)
    }

    /// Read and interpret the next bytes as the inner attributes of the `Code`
    /// attribute.
    fn parse_code_inner_attr(&mut self, cp: &ConstantPool, code: &mut Code) -> Result<()> {
        let code_attr_name = self.read_u16()?;
        let name = cp.get_utf8(code_attr_name as usize).context(
            "Code attribute's attribute name index didn't point to a UTF8 in the constant pool.",
        )?;

        let attr_len = self.read_u32()? as usize;
        // cursor position must always be read after reading attribute length
        let cursor = self.position();
        match name.as_bytes() {
            LINE_NUMBER_TABLE => code.line_number_table = self.parse_line_number_table()?,
            LOCAL_VARIABLE_TABLE => code.local_var_table = self.parse_local_var_table(cp)?,
            STACK_MAP_TABLE => code.stack_map_table = self.parse_stack_map_table()?,
            LOCAL_VARIABLE_TYPE_TABLE => {
                code.local_var_type_table = self.parse_local_type_table(cp)?
            }
            unrecognized => {
                let attribute = self.parse_unrec_attr(attr_len, unrecognized, "code")?;
                code.attributes.extend(attribute.iter());
            }
        };
        validate_cursor(self.position(), cursor + attr_len as u64)?;
        Ok(())
    }

    /// Read and interpret the next bytes as the `LineNumberTable`.
    fn parse_line_number_table(&mut self) -> Result<LineNumberTable> {
        let table_len = self.read_u16()?;
        let table = (0..table_len)
            .map(|_| {
                Ok(LineNumberTableEntry {
                    start_pc: self.read_u16()?,
                    line_number: self.read_u16()?,
                })
            })
            .collect::<Result<Vec<LineNumberTableEntry>>>()?;
        Ok(LineNumberTable(table))
    }

    /// Read and interpret the next bytes as the `LocalVariableTable`.
    fn parse_local_var_table(&mut self, cp: &ConstantPool) -> Result<LocalVariableTable> {
        let table_len = self.read_u16()?;
        let table = (0..table_len)
            .map(|_| {
                let start_pc = self.read_u16()?;
                let length = self.read_u16()?;
                let name_index = self.read_u16()? as usize;
                Self::verify_utf8(
                    cp,
                    name_index,
                    "LocalVariableTable entry's name_index didn't point to a UTF8 in the constant pool.",
                )?;
                let descriptor_index = self.read_u16()? as usize;
                let index = self.read_u16()? as usize;

                Ok(LocalVarTableEntry {
                    start_pc,
                    len: length,
                    name_index,
                    descriptor_index,
                    index,
                })
            })
            .collect::<Result<Vec<LocalVarTableEntry>>>()?;
        Ok(LocalVariableTable(table))
    }

    /// Read and interpret the next bytes as the `StackMapTable`.
    fn parse_stack_map_table(&mut self) -> Result<StackMapTable> {
        let table_len = self.read_u16()?;
        let entries = (0..table_len)
            .map(|_| self.read_smf())
            .collect::<Result<Vec<StackMapFrame>>>()?;
        Ok(StackMapTable(entries))
    }

    /// Read and interpret the next bytes as the `LocalVariableTypeTable`.
    fn parse_local_type_table(&mut self, cp: &ConstantPool) -> Result<LocalVariableTypeTable> {
        let lvtt_len = self.read_u16()?;
        let lvtt= (0..lvtt_len).map(|_| {
            let start_pc = self.read_u16()?;
            let len = self.read_u16()?;
            let name_index = self.read_u16()? as usize;
            let signature_index = self.read_u16()? as usize;
            if cp.get_utf8(signature_index).is_err() {
                bail!("LocalVariableTypeTable signature_index did not point to a valid UTF8 in the constant pool.");
            }
            let index = self.read_u16()? as usize;
            Ok(LocalVarTypeEntry {
                start_pc,
                len,
                name_index,
                signature_index,
                index,
            })
        }).collect::<Result<Vec<LocalVarTypeEntry>>>()?;
        Ok(LocalVariableTypeTable(lvtt))
    }

    /// Read and interpret the next bytes as the `StackMapFrame`.
    fn read_smf(&mut self) -> Result<StackMapFrame> {
        let smf = match self.read_u8()? {
            0..=63 => StackMapFrame::SameFrame,
            64..=127 => StackMapFrame::SameLocals(self.get_v_type()?),
            247 => {
                let offset = self.read_u16()?;
                let v_type = self.get_v_type()?;
                StackMapFrame::SameLocalsExt(offset, v_type)
            }
            248..=250 => StackMapFrame::Chop(self.read_u16()?),
            251 => StackMapFrame::SameFrameExt(self.read_u16()?),
            num @ 252..=254 => {
                let offset = self.read_u16()?;
                let locals = self.v_type_vec((num - 251) as usize)?;
                StackMapFrame::Append(offset, locals)
            }
            255 => {
                let offset = self.read_u16()?;
                let local_len = self.read_u16()? as usize;
                let locals = self.v_type_vec(local_len)?;
                let stack_len = self.read_u16()? as usize;
                let stack = self.v_type_vec(stack_len)?;
                StackMapFrame::Full {
                    offset,
                    locals,
                    stack,
                }
            }
            invalid => {
                bail!("Invalid byte {{{invalid}}} for StackMapFrame.");
            }
        };
        Ok(smf)
    }

    /// Read and interpret the next bytes as `size` verification_type
    /// attributes.
    fn v_type_vec(&mut self, size: usize) -> Result<Vec<VType>> {
        let vec = (0..size)
            .map(|_| self.get_v_type())
            .collect::<Result<Vec<VType>>>()?;
        Ok(vec)
    }

    /// Read and interpret the next bytes as the verification_type attribute.
    fn get_v_type(&mut self) -> Result<VType> {
        let mut v_type = VType::try_from(self.read_u8()?)?;
        match v_type {
            VType::Object(ref mut index) => {
                *index = self.read_u16()? as usize;
            }
            VType::UninitVar(ref mut index) => {
                *index = self.read_u16()? as usize;
            }
            _ => {}
        }
        Ok(v_type)
    }

    /// Read and interpret the next bytes as the class fields.
    pub(crate) fn parse_fields(&mut self, cp: &ConstantPool) -> Result<Vec<Field>> {
        let fields_len = self.read_u16()?;
        let fields = (0..fields_len)
            .map(|_| self.parse_field(cp))
            .collect::<Result<Vec<Field>>>()?;
        Ok(fields)
    }

    /// Read and interpret the next bytes as a `Method`.
    fn parse_method(&mut self, cp: &ConstantPool, _min_ver: u16, _maj_ver: u16) -> Result<Method> {
        let mut method = Method {
            access_flags: MethodAccessFlags::from_bits(self.read_u16()?)
                .context("Invalid access flags on method.")?,
            ..Default::default()
        };
        // TODO: uncomment when implement
        // access_flags.verify_flags(min_ver, maj_ver, /*is_interface*/)

        let name = cp.get_utf8(self.read_u16()? as usize)?;
        method.name = name.to_string();

        let descriptor = cp.get_utf8(self.read_u16()? as usize)?;
        method.descriptor = descriptor.to_string();
        method.parsed_descriptor = descriptor.parse::<MethodDescriptor>()?;

        self.parse_method_attr(cp, &mut method)?;

        Ok(method)
    }

    /// Read and interpret the next bytes as a `Field`.
    fn parse_field(&mut self, cp: &ConstantPool) -> Result<Field> {
        let mut field = Field::default();

        let flag = self.read_u16()?;
        field.access_flags = FieldAccessFlags::from_bits(flag)
            .context(format!("Invalid Field access flag: {flag}"))?;

        let name = cp.get_utf8(self.read_u16()? as usize)?;
        field.name = name.to_string();

        let descriptor = cp.get_utf8(self.read_u16()? as usize)?;
        field.descriptor = descriptor.to_string();
        field.parsed_descriptor = descriptor.parse::<FieldDescriptor>()?;

        let attributes_count = self.read_u16()? as usize;
        self.parse_field_attr(cp, attributes_count, &mut field)?;

        Ok(field)
    }

    /// Read and interpret the next bytes as the `Field` attributes.
    fn parse_field_attr(
        &mut self,
        cp: &ConstantPool,
        attr_count: usize,
        field: &mut Field,
    ) -> Result<()> {
        let mut contains_sig = false;
        for _ in 0..attr_count {
            let name_index = self.read_u16()?;
            let attr_len = self.read_u32()? as usize;
            // cursor position must always be read after reading attribute length
            let cursor = self.position();
            let name = cp.get_utf8(name_index as usize)?;
            match name.as_bytes() {
                CONSTANT_VALUE => {
                    let const_index = self.read_u16()? as usize;
                    if let Some(constant) = cp.get(const_index) {
                        if !matches!(
                            constant,
                            Integer(_) | Float(_) | Long(_) | Double(_) | Constant::String { .. }
                        ) {
                            bail!("Constant at ConstantValue index was not a valid constant");
                        }
                        field.constant_value = constant.clone();
                    } else {
                        bail!("Constant Value attribute did not have a valid index")
                    }
                }
                SIGNATURE => field.signature = self.parse_sig_attr(cp, &mut contains_sig)?,
                DEPRECATED => {
                    if attr_len != 0 {
                        bail!("Synthetic attribute length was not 0");
                    }
                    field.deprecated = true;
                }
                SYNTHETIC => {
                    if attr_len != 0 {
                        bail!("Synthetic attribute length was not 0");
                    }
                    field.synthetic = true;
                }
                name @ (RUNTIME_VISIBLE_ANNOTATIONS
                | RUNTIME_INVISIBLE_ANNOTATIONS
                | RUNTIME_INVISIBLE_TYPE_ANNOTATIONS
                | RUNTIME_VISIBLE_TYPE_ANNOTATIONS) => {
                    field
                        .runtime_annotations
                        .push(self.parse_runtime_anno(cp, name)?);
                }
                unrecognized => {
                    let _ = self.parse_unrec_attr(attr_len, unrecognized, "field")?;
                }
            };
            validate_cursor(self.position(), cursor + attr_len as u64)?;
        }
        Ok(())
    }

    /// Read and interpret the next bytes as the class' list of interface
    /// indices
    pub(crate) fn parse_interfaces(&mut self) -> Result<Vec<usize>> {
        let interfaces_len = self.read_u16()?;
        let interfaces = (0..interfaces_len)
            .map(|_| {
                let i = self.read_u16()?;
                Ok(i as usize)
            })
            .collect::<Result<Vec<usize>>>()?;
        Ok(interfaces)
    }

    /// Read and interpret the next bytes as the attributes for the `ClassFile`.
    pub(crate) fn parse_class_attr(
        &mut self,
        cp: &ConstantPool,
        attr_count: usize,
        permit_sub: bool,
        attributes: &mut ClassFileAttributes,
    ) -> Result<()> {
        let mut src_dbg_ext = false;
        let mut btstrp_mthds = false;
        let mut record = false;
        let mut contains_permitted = false;
        let mut contains_sig = false;
        for _ in 0..attr_count {
            let name_index = self.read_u16()? as usize;
            Self::verify_utf8(
                cp,
                name_index,
                "ClassFile attribute's name_index did not point to a UTF8 in the constant pool.",
            )?;
            let attr_len = self.read_u32()? as usize;
            // cursor position must always be read after reading attribute length
            let cursor = self.position();

            let name = cp.get_utf8(name_index).context(
                "ClassFile attribute's name_index did not point to a UTF8 in the constant pool.",
            )?;
            match name.as_bytes() {
                INNER_CLASSES => attributes.inner_classes = self.parse_inner_class_attr(cp)?,
                ENCLOSING_METHOD => {
                    attributes.enclosing_method = EnclosingMethod {
                        class_index: self.read_u16()? as usize,
                        method_index: self.read_u16()? as usize,
                    }
                }
                SOURCE_FILE => {
                    let file_index = self.read_u16()? as usize;
                    Self::verify_utf8(
                        cp,
                        file_index,
                        "SourceFile index didn't point to a UTF8 in the constant pool.",
                    )?;
                    attributes.source_file = SourceFile(file_index);
                }
                SOURCE_DEBUG_EXTENSION => {
                    if src_dbg_ext {
                        bail!(
                            "ClassFile Attributes cannot have more than one SourceDebugExtension."
                        );
                    }
                    src_dbg_ext = true;
                    attributes.source_debug_extension =
                        SourceDebugExtension(self.read_bytes(attr_len)?);
                }
                SIGNATURE => attributes.signature = self.parse_sig_attr(cp, &mut contains_sig)?,
                BOOTSTRAP_METHODS => {
                    attributes.bootstrap_methods =
                        self.parse_btstrp_methods_attr(&mut btstrp_mthds)?
                }
                MODULE => attributes.module = self.parse_module_attr(cp)?,
                MODULE_PACKAGES => attributes.module_packages = self.parse_module_pckgs_attr(cp)?,
                MODULE_MAIN_CLASS => {
                    let main_class_index = self.read_u16()? as usize;
                    if !matches!(cp.get(main_class_index), Some(Class { .. })) {
                        bail!(
                            "ModuleMainClass' index didn't point to a Class in the constant pool."
                        );
                    }
                    attributes.module_main_class = ModuleMainClass(main_class_index);
                }
                NEST_HOST => {
                    let host_index = self.read_u16()? as usize;
                    if !matches!(cp.get(host_index), Some(Class { .. })) {
                        bail!("NestHost attribute's host_index didn't point to a Class in the constant pool.");
                    }
                    attributes.nest_host = NestHost(host_index)
                }
                NEST_MEMBERS => {
                    let classes_len = self.read_u16()? as usize;
                    let classes =
                        self.read_classes(classes_len, cp, "NestedMembers' classes entry")?;
                    attributes.nest_members = NestMembers(classes)
                }
                RECORD => attributes.record = self.parse_record_attr(cp, &mut record)?,
                name @ (RUNTIME_VISIBLE_ANNOTATIONS
                | RUNTIME_INVISIBLE_ANNOTATIONS
                | RUNTIME_INVISIBLE_TYPE_ANNOTATIONS
                | RUNTIME_VISIBLE_TYPE_ANNOTATIONS) => {
                    attributes
                        .runtime_annotations
                        .push(self.parse_runtime_anno(cp, name)?);
                }
                PERMITTED_SUBCLASSES => {
                    if !permit_sub {
                        bail!("Class attributes contained a PermittedSubclass attribute for a 'final' class.");
                    } else if contains_permitted {
                        bail!("Multiple PermittedSubclasses attributes found in class attributes.");
                    }
                    contains_permitted = true;
                    let classes_len = self.read_u16()? as usize;
                    let classes = self.read_classes(classes_len, cp, "PermittedSubclasses")?;
                    attributes.permitted_subclasses = PermittedSubclasses(classes);
                }
                unrecognized => {
                    let _ = self.parse_unrec_attr(attr_len, unrecognized, "class")?;
                }
            };
            validate_cursor(self.position(), cursor + attr_len as u64)?;
        }
        Ok(())
    }

    /// Read and interpret the next bytes as the `Record` attributes.
    fn parse_record_attr(&mut self, cp: &ConstantPool, rcrd_parsed: &mut bool) -> Result<Record> {
        if *rcrd_parsed {
            bail!("There can only be at most one Record attribute per class.");
        }
        *rcrd_parsed = true;
        let components_len = self.read_u16()? as usize;
        let components = (0..components_len)
            .map(|_| self.parse_rec_comp(cp))
            .collect::<Result<Vec<RecordComponent>>>()?;

        Ok(Record(components))
    }

    /// Read and interpret the next bytes as the `RecordComponent` attribute.
    fn parse_rec_comp(&mut self, cp: &ConstantPool) -> Result<RecordComponent> {
        let name_index = self.read_u16()? as usize;
        Self::verify_utf8(
            cp,
            name_index,
            "name_index in RecordComponent struct didn't point to a UTF8",
        )?;
        let descriptor_index = self.read_u16()? as usize;
        Self::verify_utf8(
            cp,
            descriptor_index,
            "descriptor_index in RecordComponent struct didn't point to a UTF8",
        )?;

        let attr_count = self.read_u16()? as usize;
        let mut runtime_annotations = Vec::with_capacity(attr_count);
        let mut signature = "".to_string();
        let mut contains_sig = false;

        for _ in 0..attr_count {
            let name_index = self.read_u16()? as usize;
            let name = cp.get_utf8(name_index)?;
            match name.as_bytes() {
                SIGNATURE => {
                    let _len = self.read_u32()?;
                    signature = self.parse_sig_attr(cp, &mut contains_sig).unwrap()
                }
                _ => {
                    let runtime_anno = self.parse_runtime_anno(cp, name.as_bytes())?;
                    runtime_annotations.push(runtime_anno);
                }
            }
        }

        Ok(RecordComponent {
            name_index,
            descriptor_index,
            signature,
            runtime_annotations,
        })
    }

    fn parse_runtime_anno(&mut self, cp: &ConstantPool, name: &[u8]) -> Result<RuntimeAnnotation> {
        let anno = match name {
            RUNTIME_VISIBLE_ANNOTATIONS => {
                RuntimeVisibleAnnotations(self.parse_invis_vis_anno(cp)?)
            }
            RUNTIME_INVISIBLE_ANNOTATIONS => {
                RuntimeInvisibleAnnotations(self.parse_invis_vis_anno(cp)?)
            }
            RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS => {
                RuntimeInvisibleParameterAnnotations(self.parse_para_anno(cp)?)
            }
            RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS => {
                RuntimeVisibleParameterAnnotations(self.parse_para_anno(cp)?)
            }
            RUNTIME_INVISIBLE_TYPE_ANNOTATIONS => {
                RuntimeInvisibleTypeAnnotations(self.parse_type_anno(cp)?)
            }
            RUNTIME_VISIBLE_TYPE_ANNOTATIONS => {
                RuntimeVisibleTypeAnnotations(self.parse_type_anno(cp)?)
            }
            ANNOTATION_DEFAULT => AnnotationDefault(ElementPairs(
                self.read_u16()? as usize,
                self.parse_element_value(cp)?,
            )),
            name => {
                // TODO This might not be spec; We may have to ignore unrecognized annotations
                bail!("Invalid annotation name: {}", String::from_utf8_lossy(name));
            }
        };
        Ok(anno)
    }

    /// Read and interpret the next bytes as the `RuntimeVisible` and
    /// `RuntimeInvisible` annotations.
    fn parse_invis_vis_anno(&mut self, cp: &ConstantPool) -> Result<Vec<Annotation>> {
        let num_anno = self.read_u16()? as usize;
        (0..num_anno)
            .map(|_| self.parse_invis_vis_inner_anno(cp))
            .collect::<Result<Vec<Annotation>>>()
    }

    /// Read and interpret the next bytes as the list of parameter annotations.
    fn parse_para_anno(&mut self, cp: &ConstantPool) -> Result<Vec<ParameterAnnotation>> {
        let num_para = self.read_u8()? as usize;
        (0..num_para)
            .map(|_| Ok(ParameterAnnotation(self.parse_invis_vis_anno(cp)?)))
            .collect::<Result<Vec<ParameterAnnotation>>>()
    }

    /// Read and interpret the next bytes as a list of type annotations.
    fn parse_type_anno(&mut self, cp: &ConstantPool) -> Result<Vec<TypeAnnotation>> {
        /*
            TODO This function may need to be changed due to different target_types
            meaning different things despite mapping to the same TypeTarget.
            https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.7.20
        */
        let anno_len = self.read_u16()?;
        (0..anno_len)
            .map(|_| {
                let type_target = self.parse_type_target()?;
                let type_path = self.parse_type_path()?;
                let type_index = self.read_u16()? as usize;
                let value_pairs = self.parse_element_pairs(cp)?;

                Ok(TypeAnnotation {
                    type_target,
                    type_path,
                    type_index,
                    value_pairs,
                })
            })
            .collect::<Result<Vec<TypeAnnotation>>>()
    }

    /// Read and interpret the next bytes as the `Signature` attribute.
    fn parse_sig_attr(&mut self, cp: &ConstantPool, contains_sig: &mut bool) -> Result<String> {
        if *contains_sig {
            bail!("Attribute cannot have more that one Signature attribute")
        }
        *contains_sig = true;
        let sig = cp.get_utf8(self.read_u16()? as usize)?;
        // TODO Implement more type checks in the JVM 21 spec
        Ok(sig.to_string())
    }

    /// Read and interpret the next bytes as a `TypePath`.
    fn parse_type_path(&mut self) -> Result<TypePath> {
        let path_len = self.read_u8()?;
        let entries = (0..path_len)
            .map(|_| {
                Ok(TypePathEntry {
                    type_path_kind: self.read_u8()?,
                    type_arg_index: self.read_u8()? as usize,
                })
            })
            .collect::<Result<Vec<TypePathEntry>>>()?;
        Ok(TypePath(entries))
    }

    /// Read and interpret the next bytes as a `TypeTarget`.
    fn parse_type_target(&mut self) -> Result<TypeTarget> {
        let value = self.read_u8()?;
        let type_target = match value {
            0x00 | 0x01 => TypeParameter {
                value,
                type_param_index: self.read_u16()? as usize,
            },
            0x10 => SuperType {
                value,
                supertype_index: self.read_u16()? as usize,
            },
            0x11 | 0x12 => TypeParameterBound {
                value,
                type_parameter_index: self.read_u16()? as usize,
                bound_index: self.read_u16()? as usize,
            },
            0x13..=0x15 => Empty(value),
            0x16 => FormalParameter {
                value,
                formal_param_index: self.read_u16()? as usize,
            },
            0x17 => Throws {
                value,
                throws_type_index: self.read_u16()? as usize,
            },
            0x40 | 0x41 => {
                let table_len = self.read_u16()?;
                let target_table = (0..table_len)
                    .map(|_| {
                        Ok(LocalVarTargetTableEntry {
                            start_pc: self.read_u16()?,
                            length: self.read_u16()?,
                            index: self.read_u16()? as usize,
                        })
                    })
                    .collect::<Result<Vec<LocalVarTargetTableEntry>>>()?;
                LocalVar {
                    value,
                    target_table: LocalVarTargetTable(target_table),
                }
            }
            0x42 => Catch {
                value,
                exception_table_index: self.read_u16()? as usize,
            },
            0x43..=0x46 => Offset {
                value,
                offset: self.read_u16()?,
            },
            0x47..=0x4B => TypeArgument {
                value,
                type_arg_index: self.read_u16()? as usize,
            },
            _ => {
                bail!("Invalid TypeTarget value.");
            }
        };
        Ok(type_target)
    }

    /// Read and interpret the next bytes as the `ModulePackages` attribute.
    fn parse_module_pckgs_attr(&mut self, cp: &ConstantPool) -> Result<ModulePackages> {
        let package_len = self.read_u16()? as usize;
        let package_index = (0..package_len).map(|_| {
            let index = self.read_u16()? as usize;
            if !matches!(cp.get(index), Some(Package { .. })) {
                bail!("ModulePackages attribute's package_index entry didn't point to a Package in the constant pool.");
            }
            Ok(index)
        }).collect::<Result<Vec<usize>>>()?;
        Ok(ModulePackages(package_index))
    }

    /// Read and interpret the next bytes as `BootstrapMethods`.
    fn parse_btstrp_methods_attr(&mut self, btstr_prased: &mut bool) -> Result<BootstrapMethods> {
        if *btstr_prased {
            bail!("ClassFile Attributes cannot have more than one BootstrapMethods.");
        }
        *btstr_prased = true;
        let methods_len = self.read_u16()? as usize;
        let methods = (0..methods_len)
            .map(|_| {
                let btstr_mthd_ref = self.read_u16()?;
                let num_bts_args = self.read_u16()? as usize;
                let btstr_args = self.read_u16s(num_bts_args)?;
                Ok(BootstrapMethod {
                    btstr_mthd_ref,
                    btstr_args,
                })
            })
            .collect::<Result<Vec<BootstrapMethod>>>()?;
        Ok(BootstrapMethods(methods))
    }

    /// Read and interpret the next bytes as the inner class attributes.
    fn parse_inner_class_attr(&mut self, cp: &ConstantPool) -> Result<InnerClasses> {
        let class_len = self.read_u16()? as usize;
        let mut classes = Vec::with_capacity(class_len);
        for _ in 0..class_len {
            /*
                if inner_class_info_index, outer_class_info_index, or inner_name_index are 0
                that indicates that the class or interface is anonymous and thus will not have
                valid entries in the constant pool.
            */

            let inner_class_info_index = self.read_u16()? as usize;
            if inner_class_info_index != 0
                && !matches!(cp.get(inner_class_info_index), Some(Class { .. }))
            {
                bail!("ClassFile InnerClass Attributes inner_class_info_index didn't point to a UTF8 in the constant pool.");
            }
            let outer_class_info_index = self.read_u16()? as usize;
            if outer_class_info_index != 0
                && !matches!(cp.get(inner_class_info_index), Some(Class { .. }))
            {
                bail!("ClassFile InnerClass Attributes outer_class_info_index didn't point to a UTF8 in the constant pool.");
            }
            let inner_name_index = self.read_u16()? as usize;
            if inner_name_index != 0 {
                Self::verify_utf8(
                    cp,
                    inner_name_index,
                    "ClassFile InnerClass Attribute's inner_name_index didn't point to a UTF8 in constant pool.")?;
            }
            let access_flags = self.read_u16()?;
            let inner_class_access_flags = InnerClassAccessFlags::from_bits(access_flags)
                .context(format!("Invalid Class access flag: {access_flags}."))?;
            classes.push(InnerClassInfo {
                inner_class_info_index,
                outer_class_info_index,
                inner_name_index,
                inner_class_access_flags,
            });
        }
        Ok(InnerClasses(classes))
    }

    /// Read and interpret the next bytes as `Module` attributes.
    fn parse_module_attr(&mut self, cp: &ConstantPool) -> Result<attribute::Module> {
        let module_name_index = self.read_u16()? as usize;

        let flag = self.read_u16()?;
        let module_flags =
            ModuleFlags::from_bits(flag).context(format!("Invalid Module flag: {flag}"))?;

        let module_ver_index = self.read_u16()? as usize;

        let requires_len = self.read_u16()? as usize;
        let requires = (0..requires_len)
            .map(|_| self.parse_requires(cp))
            .collect::<Result<Vec<Requires>>>()?;

        let exports_len = self.read_u16()? as usize;
        let exports = (0..exports_len)
            .map(|_| self.parse_exports(cp))
            .collect::<Result<Vec<Exports>>>()?;

        let opens_len = self.read_u16()? as usize;
        let opens = (0..opens_len)
            .map(|_| self.parse_opens(cp))
            .collect::<Result<Vec<Opens>>>()?;

        let uses_len = self.read_u16()? as usize;
        let uses = self.read_classes(uses_len, cp, "Module's attribute's uses_index")?;

        let provides_len = self.read_u16()? as usize;
        let provides = (0..provides_len)
            .map(|_| self.parse_provides(cp))
            .collect::<Result<Vec<Provides>>>()?;

        Ok(attribute::Module {
            module_name_index,
            module_flags,
            module_ver_index,
            requires,
            exports,
            opens,
            uses,
            provides,
        })
    }

    /// Read and interpret the next bytes as the `Requires` attribute.
    fn parse_requires(&mut self, cp: &ConstantPool) -> Result<Requires> {
        let requires_index = self.read_u16()? as usize;
        if !matches!(cp.get(requires_index), Some(Module { .. })) {
            bail!(
                "Requires' requires_index didn't point to a Module constant in the constant pool."
            );
        }
        let flag = self.read_u16()?;
        let requires_flags =
            RequiresFlags::from_bits(flag).context(format!("Invalid Requires flag: {flag}"))?;
        let requires_ver_index = self.read_u16()? as usize;
        Self::verify_utf8(
            cp,
            requires_ver_index,
            "Requires_ver_index didn't point to a UTF8 in the constant pool.",
        )?;
        Ok(Requires {
            requires_index,
            requires_flags,
            requires_ver_index,
        })
    }

    /// Read and interpret the next bytes as the `Exports` attribute.
    fn parse_exports(&mut self, cp: &ConstantPool) -> Result<Exports> {
        let exports_index = self.read_u16()? as usize;
        if !matches!(cp.get(exports_index), Some(Package { .. })) {
            bail!(
                "Exports' exports_index didn't point to a Package constant in the constant pool."
            );
        }
        let flag = self.read_u16()?;
        let exports_flags =
            ExportFlags::from_bits(flag).context(format!("Invalid Exports flag: {flag}"))?;
        let exports_to_index = {
            let exports_len = self.read_u16()? as usize;
            let mut export_to_index = Vec::with_capacity(exports_len);
            for _ in 0..exports_len {
                let const_index = self.read_u16()? as usize;
                if !matches!(cp.get(const_index), Some(Package { .. })) {
                    bail!("An Exprots' exports_to_index entry didn't point to a Package in the constant pool.");
                }
                export_to_index.push(const_index);
            }
            export_to_index
        };

        Ok(Exports {
            exports_index,
            exports_flags,
            exports_to_index,
        })
    }

    /// Read and interpret the next bytes as a `Opens`.
    fn parse_opens(&mut self, cp: &ConstantPool) -> Result<Opens> {
        let opens_index = self.read_u16()? as usize;
        if !matches!(cp.get(opens_index), Some(Package { .. })) {
            bail!(
                "Exports' exports_index didn't point to a Package constant in the constant pool."
            );
        }
        let flag = self.read_u16()?;
        let opens_flags =
            OpenFlags::from_bits(flag).context(format!("Invalid Opens flag: {flag}"))?;
        let opens_to_index = {
            let opens_len = self.read_u16()? as usize;
            let mut opens_to_index = Vec::with_capacity(opens_len);
            for _ in 0..opens_len {
                let const_index = self.read_u16()?;
                if !matches!(cp.get(const_index as usize), Some(Package { .. })) {
                    bail!("An Opens' opens_to_index entry didn't point to a Package in the constant pool.");
                }
                opens_to_index.push(const_index);
            }
            opens_to_index
        };
        Ok(Opens {
            opens_index,
            opens_flags,
            opens_to_index,
        })
    }

    /// Read and interpret the next bytes as a `Provides`.
    fn parse_provides(&mut self, cp: &ConstantPool) -> Result<Provides> {
        let provides_index = self.read_u16()? as usize;
        if !matches!(cp.get(provides_index), Some(Class { .. })) {
            bail!("Provides' provides_index didn't point to a Class in the constant pool.");
        }

        let provides_len = self.read_u16()? as usize;
        let provides_with_index = self.read_classes(provides_len, cp, "Provides struct")?;
        Ok(Provides {
            provides_index,
            provides_with_index,
        })
    }

    /// Read the next `length` bytes. No interpretation implemented.
    #[inline]
    pub(crate) fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        (0..length)
            .map(|_| self.read_u8())
            .collect::<Result<Vec<u8>>>()
    }

    /// Read the next `length` `u16`s. No interpretation implemented.
    #[inline]
    pub(crate) fn read_u16s(&mut self, length: usize) -> Result<Vec<u16>> {
        (0..length)
            .map(|_| self.read_u16())
            .collect::<Result<Vec<u16>>>()
    }

    /// Read and interpret the next `length` bytes as indices to the constant
    /// pool, where `Constant::Class` entries can be found.
    fn read_classes(
        &mut self,
        length: usize,
        cp: &ConstantPool,
        context: &'static str,
    ) -> Result<Vec<usize>> {
        let classes =(0..length).map(|_| {
            let class = self.read_u16()? as usize;
            if !matches!(cp.get(class), Some(Class { .. })) {
                bail!("While parsing {context}, class index didn't point to a Class in the constant pool.");
            }
            Ok(class)
        }).collect::<Result<Vec<usize>>>()?;
        Ok(classes)
    }

    /// Verifies the constant pool entry at the provided `index` is a
    /// `Constant::UTF8`.
    #[inline]
    fn verify_utf8(cp: &ConstantPool, index: usize, message: &'static str) -> Result<()> {
        if cp.get_utf8(index).is_err() {
            bail!(message);
        }
        Ok(())
    }

    /// Read and interpret the next bytes as the `ElementValue` attribute.
    fn parse_element_value(&mut self, cp: &ConstantPool) -> Result<ElementValue> {
        let tag = self.read_u8()? as char;
        let element_value = match tag {
            'B' | 'C' | 'I' | 'Z' | 'S' => {
                let const_type_index = self.read_u16()? as usize;
                if !matches!(cp.get(const_type_index), Some(Integer(_))) {
                    bail!("const_type_index didn't point to a Integer in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'D' => {
                let const_type_index = self.read_u16()? as usize;
                if !matches!(cp.get(const_type_index), Some(Double(_))) {
                    bail!("const_type_index didn't point to a Double in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'F' => {
                let const_type_index = self.read_u16()? as usize;
                if !matches!(cp.get(const_type_index), Some(Float(_))) {
                    bail!("const_type_index didn't point to a Float in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'J' => {
                let const_type_index = self.read_u16()? as usize;
                if !matches!(cp.get(const_type_index), Some(Long(_))) {
                    bail!("const_type_index didn't point to a Long in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            's' => {
                let const_type_index = self.read_u16()? as usize;
                if !matches!(cp.get(const_type_index), Some(UTF8(_))) {
                    bail!("const_type_index didn't point to a Float in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'e' => {
                let type_name_index = self.read_u16()? as usize;
                Self::verify_utf8(
                    cp,
                    type_name_index,
                    "type_name_index didn't point to a UTF8 in the constant pool.",
                )?;

                let const_name_index = self.read_u16()? as usize;
                Self::verify_utf8(
                    cp,
                    const_name_index,
                    "const_name_index didn't point to a UTF8 in the constant pool.",
                )?;

                ElementValue::EnumConst(type_name_index, const_name_index)
            }
            'c' => ClassIndex(self.read_u16()? as usize),
            '@' => AnnotationValue(self.parse_invis_vis_inner_anno(cp)?),
            '[' => {
                let values = {
                    let len = self.read_u16()? as usize;
                    let mut values = Vec::with_capacity(len);
                    for _ in 0..len {
                        values.push(self.parse_element_value(cp)?)
                    }
                    values
                };
                Array(values)
            }
            _ => {
                bail!("Invalid tag: [{tag}] while parsing ElementValue.");
            }
        };
        Ok(element_value)
    }

    /// Helper method for parsing `RuntimeVisible` and `RuntimeInvisible`
    /// annotations.
    fn parse_invis_vis_inner_anno(&mut self, cp: &ConstantPool) -> Result<Annotation> {
        let type_index = self.read_u16()? as usize;
        Self::verify_utf8(
            cp,
            type_index,
            "Annotation's type_index didn't point to a UTF8.",
        )?;
        let value_pairs = self.parse_element_pairs(cp)?;
        Ok(Annotation {
            type_index,
            value_pairs,
        })
    }

    /// Read and interpret the next bytes as an array of `ElementPairs`
    /// attributes.
    fn parse_element_pairs(&mut self, cp: &ConstantPool) -> Result<Vec<ElementPairs>> {
        let pairs_len = self.read_u16()? as usize;
        (0..pairs_len)
            .map(|_| {
                let name_index = self.read_u16()? as usize;
                Self::verify_utf8(
                    cp,
                    name_index,
                    "ElementPair's element_name_index didn't point to a UTF8 in the constant pool.",
                )?;
                Ok(ElementPairs(name_index, self.parse_element_value(cp)?))
            })
            .collect::<Result<Vec<ElementPairs>>>()
    }

    /// Read and interpret the given bytes as a JVM compliant Utf8 string.
    fn parse_jvm_utf8(bytes: &[u8]) -> Result<String> {
        fn parse_u32(decoded_string: &mut String, code: u32) -> Result<()> {
            match char::from_u32(code) {
                Some(decoded) => decoded_string.push(decoded),
                None => bail!("Error while decoding string!"),
            }
            Ok(())
        }

        let mut decoded_string = String::new();
        let mut index = 0;

        while index < bytes.len() {
            let byte = bytes[index];

            if byte & 0x80 == 0 {
                // Single-byte character
                decoded_string.push(byte as char);
                index += 1;
            } else if byte & 0xE0 == 0xC0 {
                // Two-byte character
                if index + 1 < bytes.len() {
                    let code_point =
                        (((byte & 0x1F) as u32) << 6) | ((bytes[index + 1] & 0x3F) as u32);
                    parse_u32(&mut decoded_string, code_point)?;
                    index += 2;
                } else {
                    bail!("Incomplete two-byte character");
                }
            } else if byte & 0xF0 == 0xE0 {
                // Three-byte character
                if index + 2 < bytes.len() {
                    let code_point = (((byte & 0x0F) as u32) << 12)
                        | (((bytes[index + 1] & 0x3F) as u32) << 6)
                        | ((bytes[index + 2] & 0x3F) as u32);
                    parse_u32(&mut decoded_string, code_point)?;
                    index += 3;
                } else {
                    bail!("Incomplete three-byte character");
                }
            } else if byte == 0xED {
                // 2 * 6 byte character (UTF-8 representation of surrogate pair)
                // 0x10000 + ((v & 0x0f) << 16) + ((w & 0x3f) << 10) +
                // ((y & 0x0f) << 6) + (z & 0x3f)
                if index + 5 < bytes.len() {
                    let v: u32 = bytes[index + 1] as u32;
                    let w: u32 = bytes[index + 2] as u32;
                    let y: u32 = bytes[index + 4] as u32;
                    let z: u32 = bytes[index + 5] as u32;

                    let code_point = 0x10000
                        + ((v & 0x0F) << 16)
                        + ((w & 0x3F) << 10)
                        + ((y & 0x0F) << 6)
                        + (z & 0x3F);
                    parse_u32(&mut decoded_string, code_point)?;
                } else {
                    bail!("Incomplete four-byte character");
                }
            } else {
                bail!("Invalid byte sequence");
            }
        }
        Ok(decoded_string)
    }
}

impl Deref for ClassReader {
    type Target = Cursor<Vec<u8>>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for ClassReader {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

fn log_unrec_attr(name: &str, context: &str) -> Result<()> {
    //FIXME: Properly handle file not found error. Currently is found or not found
    // depending on if the program is run using cargo test or cargo run.
    let mut file = match OpenOptions::new()
        .append(true)
        .open("parser/unrec_attrs.txt")
    {
        Ok(file) => file,
        Err(_) => return Ok(()),
    };

    let message = format!(
        "{:?}: Unrecognized attribute name {name} while parsing {context} attributes.\n",
        SystemTime::now()
    );
    file.write_all(message.as_bytes())?;
    Ok(())
}

/// Helper function that confirms the `ClassReader`'s cursor is where it is
/// expected.
#[inline]
fn validate_cursor(curr: u64, expect: u64) -> Result<()> {
    if curr != expect {
        bail!("Invalid cursor position. Current pos: {curr}. Expected pos: {expect}");
    }
    Ok(())
}
