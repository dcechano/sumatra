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
use bitflags::Flags;
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    annotation::{
        Annotation, ElementPairs, ElementValue,
        ElementValue::{AnnotationValue, Array, ClassIndex, ConstIndex},
    },
    attribute::{
        attr_constants::{
            BOOTSTRAP_METHODS, CODE, CONSTANT_VALUE, DEPRECATED, ENCLOSING_METHOD, EXCEPTIONS,
            INNER_CLASSES, LINE_NUMBER_TABLE, LOCAL_VARIABLE_TABLE, LOCAL_VARIABLE_TYPE_TABLE,
            MODULE, MODULE_MAIN_CLASS, MODULE_PACKAGES, NEST_HOST, NEST_MEMBERS,
            PERMITTED_SUBCLASSES, RECORD, RUNTIME_INVISIBLE_ANNOTATIONS,
            RUNTIME_VISIBLE_ANNOTATIONS, SIGNATURE, SOURCE_DEBUG_EXTENSION, SOURCE_FILE,
            STACK_MAP_TABLE, SYNTHETIC,
        },
        Attribute,
        Attribute::{
            BootstrapMethods, Code, ConstantValue, Custom, Deprecated, EnclosingMethod, Exceptions,
            InnerClasses, LineNumberTable, LocalVariableTable, LocalVariableTypeTable,
            ModuleMainClass, ModulePackages, NestHost, NestMembers, PermittedSubclasses, Record,
            RuntimeInvisibleAnnotations, RuntimeVisibleAnnotations, Signature,
            SourceDebugExtension, SourceFile, StackMapTable, Synthetic,
        },
        BootstrapMethod, Exception, Exports, InnerClassInfo, LineNumberTableEntry,
        LocalVarTableEntry, LocalVarTypeEntry, Opens, Provides, RecordComponent, Requires,
        StackMapFrame,
    },
    constant::{
        Constant,
        Constant::{
            Class, Double, Dummy, Dynamic, FieldRef, Float, Integer, InterfaceMethodRef,
            InvokeDynamic, Long, MethodHandle, MethodRef, MethodType, Module, NameAndType, Package,
            UTF8,
        },
    },
    constant_pool::ConstantPool,
    field::Field,
    flags::{
        ExportFlags, FieldAccessFlags, InnerClassAccessFlags, MethodAccessFlags, ModuleFlags,
        OpenFlags, RequiresFlags,
    },
    instruction::Instruction,
    method::Method,
    type_verification::VType,
};

const CLASS_EXT: &[u8] = b"class";
pub(crate) struct ClassReader(Cursor<Vec<u8>>);

impl ClassReader {
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

    #[inline]
    pub(crate) fn with_buffer(buffer: &[u8]) -> Self {
        Self(Cursor::new(buffer.into()))
    }

    #[inline]
    pub(crate) fn read_u8(&mut self) -> Result<u8> {
        self.0.read_u8().context("Failed to read u8")
    }

    #[inline]
    pub(crate) fn read_i8(&mut self) -> Result<i8> {
        self.0.read_i8().context("Failed to read i8")
    }

    #[inline]
    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        self.0.read_u16::<BigEndian>().context("Failed to read u16")
    }

    #[inline]
    pub(crate) fn read_i16(&mut self) -> Result<i16> {
        self.0.read_i16::<BigEndian>().context("Failed to read i16")
    }

    #[inline]
    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        self.0.read_u32::<BigEndian>().context("Failed to read u32")
    }

    #[inline]
    pub(crate) fn read_i32(&mut self) -> Result<i32> {
        self.0.read_i32::<BigEndian>().context("Failed to read i32")
    }

    #[inline]
    pub(crate) fn read_u64(&mut self) -> Result<u64> {
        self.0.read_u64::<BigEndian>().context("Failed to read u32")
    }
}

impl ClassReader {
    pub(crate) fn parse_cp(&mut self) -> Result<ConstantPool> {
        let cp_count = self.read_u16()? as usize;
        let mut cp = ConstantPool::new(cp_count);
        cp.push(Dummy);
        for _ in 0..cp_count - 1 {
            let constant = match self.read_u8()? {
                1 => {
                    let length = self.read_u16()? as usize;
                    let bytes = self.read_bytes(length)?;
                    let string = Self::parse_jvm_utf8(&bytes)?;
                    UTF8(string)
                }
                3 => Integer(self.read_u32()? as i32),
                4 => Float(f32::from(self.read_u16()?)),
                5 => Long(self.read_i64::<BigEndian>()?),
                6 => Double(self.read_f64::<BigEndian>()?),
                7 => Class {
                    name_index: self.read_u16()? as usize,
                },
                8 => Constant::String {
                    string_index: self.read_u16()? as usize,
                },
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
                16 => MethodType {
                    descriptor_index: self.read_u16()? as usize,
                },
                17 => Dynamic {
                    bootstrap_method_attr_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                18 => InvokeDynamic {
                    bootstrap_method_attr_index: self.read_u16()? as usize,
                    name_and_type_index: self.read_u16()? as usize,
                },
                19 => Module {
                    name_index: self.read_u16()? as usize,
                },
                20 => Package {
                    name_index: self.read_u16()? as usize,
                },
                unknown => bail!("Unknown constant: {unknown}"),
            };
            match constant {
                Long(_) | Double(_) => {
                    cp.push(constant);
                    cp.push(Dummy);
                }
                _ => {
                    cp.push(constant);
                }
            }
        }
        Ok(cp)
    }

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

    fn parse_method_attr(&mut self, cp: &ConstantPool) -> Result<Vec<Attribute>> {
        let mut sig_present = false;
        let mut exc_present = false;
        let attr_count = self.read_u16()? as usize;
        let mut attributes = Vec::with_capacity(attr_count);

        for _ in 0..attr_count {
            let name_index = self.read_u16()?;
            let name = cp.get_utf8(name_index as usize).context(
                "Method attribute's name_index didn't point to a UTF8 in the constant pool.",
            )?;
            let attr_len = self.read_u32()? as usize;
            // cursor position must always be read after reading attribute length
            let cursor = self.position();
            let attribute = match name.as_bytes() {
                CODE => self.parse_code_attr(cp)?,
                EXCEPTIONS => {
                    if exc_present {
                        bail!("Method cannot have more than one Exceptions attribute.");
                    }
                    exc_present = true;
                    let exc_len = self.read_u16()? as usize;
                    let exc_index_table = self.read_u16s(exc_len)?;
                    Exceptions(exc_index_table)
                }
                SIGNATURE => {
                    if sig_present {
                        bail!("Method cannot have more than one Signature attribute.");
                    }
                    sig_present = true;
                    Signature
                }
                RUNTIME_INVISIBLE_ANNOTATIONS => {
                    let anno_len = self.read_u16()?;
                    let annotations = (0..anno_len)
                        .map(|_| self.parse_annotation(cp))
                        .collect::<Result<Vec<Annotation>>>()?;
                    RuntimeInvisibleAnnotations(annotations)
                }
                RUNTIME_VISIBLE_ANNOTATIONS => {
                    let anno_len = self.read_u16()?;
                    let annotations = (0..anno_len)
                        .map(|_| self.parse_annotation(cp))
                        .collect::<Result<Vec<Annotation>>>()?;
                    RuntimeVisibleAnnotations(annotations)
                }
                unrecognized => self.parse_unrec_attr(attr_len, unrecognized, "method")?,
            };
            validate_cursor(self.position(), cursor + attr_len as u64)?;
            attributes.push(attribute);
        }
        Ok(attributes)
    }

    fn parse_unrec_attr(
        &mut self,
        length: usize,
        bytes: &[u8],
        context: &'static str,
    ) -> Result<Attribute> {
        log_unrec_attr(
            &bytes.into_iter().map(|b| *b as char).collect::<String>(),
            context,
        )?;

        let bytes = self.read_bytes(length)?;
        Ok(Custom(bytes))
    }

    fn parse_code_attr(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let max_stack = self.read_u16()?;
        let max_locals = self.read_u16()?;
        let code_len = self.read_u32()? as usize;
        let code = Instruction::from_bytes(&self.read_bytes(code_len)?)?;
        let exception_table = self.parse_exceptions_table(cp)?;
        let attrs_len = self.read_u16()? as usize;
        let attributes = (0..attrs_len)
            .map(|_| self.parse_code_inner_attr(cp))
            .collect::<Result<Vec<Attribute>>>()?;
        Ok(Code {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        })
    }

    fn parse_exceptions_table(&mut self, cp: &ConstantPool) -> Result<Vec<Exception>> {
        let table_len = self.read_u16()? as usize;
        let table = (0..table_len)
            .map(|_| {
                let start_pc = self.read_u16()?;
                let end_pc = self.read_u16()?;
                let handler_pc = self.read_u16()?;
                let catch_type = self.read_u16()?;
                let class = cp.get(catch_type as usize);
                if !matches!(class, Some(Class { .. })) {
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

    fn parse_code_inner_attr(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let code_attr_name = self.read_u16()?;
        let name = cp.get_utf8(code_attr_name as usize).context(
            "Code attribute's attribute name index didn't point to a UTF8 in the constant pool.",
        )?;

        let attr_len = self.read_u32()? as usize;
        // cursor position must always be read after reading attribute length
        let cursor = self.position();
        let attribute = match name.as_bytes() {
            LINE_NUMBER_TABLE => self.parse_line_number_table()?,
            LOCAL_VARIABLE_TABLE => self.parse_local_var_table(cp)?,
            STACK_MAP_TABLE => self.parse_stack_map_table()?,
            LOCAL_VARIABLE_TYPE_TABLE => self.parse_local_type_table(cp)?,
            unrecognized => self.parse_unrec_attr(attr_len, unrecognized, "method")?,
        };
        validate_cursor(self.position(), cursor + attr_len as u64)?;
        Ok(attribute)
    }

    fn parse_line_number_table(&mut self) -> Result<Attribute> {
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

    fn parse_local_var_table(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let table_len = self.read_u16()?;
        let table = (0..table_len)
            .map(|_| {
                let start_pc = self.read_u16()?;
                let length = self.read_u16()?;
                let name_index = self.read_u16()?;
                Self::verify_utf8(
                    cp,
                    name_index as usize,
                    "LocalVariableTable entry's name_index didn't point to a UTF8 in the constant pool.",
                )?;
                let descriptor_index = self.read_u16()?;
                let index = self.read_u16()?;

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

    fn parse_stack_map_table(&mut self) -> Result<Attribute> {
        let table_len = self.read_u16()?;
        let entries = (0..table_len)
            .map(|_| self.read_smf())
            .collect::<Result<Vec<StackMapFrame>>>()?;
        Ok(StackMapTable(entries))
    }

    fn parse_local_type_table(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let lvtt_len = self.read_u16()?;
        let lvtt= (0..lvtt_len).map(|_| {
            let start_pc = self.read_u16()?;
            let len = self.read_u16()?;
            let name_index = self.read_u16()?;
            let signature_index = self.read_u16()?;
            if cp.get_utf8(signature_index as usize).is_err() {
                bail!("LocalVariableTypeTable signature_index did not point to a valid UTF8 in the constant pool.");
            }
            let index = self.read_u16()?;
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

    fn read_smf(&mut self) -> Result<StackMapFrame> {
        let smf = match self.read_u8()? {
            0..=63 => StackMapFrame::SameFrame,
            64..=127 => StackMapFrame::SameLocals,
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

    fn v_type_vec(&mut self, size: usize) -> Result<Vec<VType>> {
        let vec = (0..size)
            .map(|_| self.get_v_type())
            .collect::<Result<Vec<VType>>>()?;
        Ok(vec)
    }

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

    pub(crate) fn parse_fields(&mut self, cp: &ConstantPool) -> Result<Vec<Field>> {
        let fields_len = self.read_u16()?;
        let fields = (0..fields_len)
            .map(|_| self.parse_field(cp))
            .collect::<Result<Vec<Field>>>()?;
        Ok(fields)
    }

    fn parse_method(&mut self, cp: &ConstantPool, min_ver: u16, maj_ver: u16) -> Result<Method> {
        let access_flags = MethodAccessFlags::from_bits(self.read_u16()?)
            .context("Invalid access flags on method.")?;
        // TODO: uncomment when implement
        // access_flags.verify_flags(min_ver, maj_ver, /*is_interface*/)

        let name_index = self.read_u16()? as usize;
        Self::verify_utf8(
            cp,
            name_index,
            "Method's name index didn't point to a UTF8 in constant pool.",
        )?;

        let descriptor_index = self.read_u16()? as usize;
        Self::verify_utf8(
            cp,
            descriptor_index,
            "Method's descriptor index didn't point to a UTF8 in the constant pool.",
        )?;

        let attributes = self.parse_method_attr(cp)?;

        Ok(Method {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }

    fn parse_field(&mut self, cp: &ConstantPool) -> Result<Field> {
        let flag = self.read_u16()?;
        let access_flags = FieldAccessFlags::from_bits(flag)
            .context(format!("Invalid Field access flag: {flag}"))?;
        let name_index = self.read_u16()? as usize;
        if !matches!(cp.get(name_index), Some(UTF8(_))) {
            bail!("name_index on field did not point to a UTF8 in the constant pool.");
        }
        let descriptor_index = self.read_u16()? as usize;
        if !matches!(cp.get(descriptor_index), Some(UTF8(_))) {
            bail!("descriptor_index on field did not point to a UTF8 in the constant pool.");
        }
        let attributes_count = self.read_u16()? as usize;
        let attributes = self.parse_field_attr(cp, attributes_count)?;

        Ok(Field {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }

    fn parse_field_attr(&mut self, cp: &ConstantPool, attr_count: usize) -> Result<Vec<Attribute>> {
        let mut contains_sig = false;
        let mut attributes = Vec::with_capacity(attr_count);
        for _ in 0..attr_count {
            let name_index = self.read_u16()?;
            let attr_len = self.read_u32()? as usize;
            // cursor position must always be read after reading attribute length
            let cursor = self.position();
            let name = cp.get_utf8(name_index as usize)?;
            let attribute = match name.as_bytes() {
                CONSTANT_VALUE => {
                    let const_index = self.read_u16()? as usize;
                    if let Some(constant) = cp.get(const_index) {
                        if !matches!(
                            constant,
                            Integer(_) | Float(_) | Long(_) | Double(_) | Constant::String { .. }
                        ) {
                            bail!("Constant at ConstantValue index was not a valid constant");
                        }
                        ConstantValue {
                            attribute_name_index: name_index,
                            attribute_info: vec![],
                        }
                    } else {
                        bail!("Constant Value attribute did not have a valid index")
                    }
                }
                SIGNATURE => {
                    if contains_sig {
                        bail!("Field attribute cannot have more that one Signature attribute")
                    }
                    contains_sig = true;
                    let sig_index = self.read_u16()? as usize;
                    Self::verify_utf8(
                        cp,
                        sig_index,
                        "Signature attribute index did not point to a UTF8 in constant pool.",
                    )?;
                    // TODO Implement more type checks in the JVM 21 spec
                    Signature
                }
                DEPRECATED => {
                    if attr_len != 0 {
                        bail!("Synthetic attribute length was not 0");
                    }
                    Deprecated
                }
                SYNTHETIC => {
                    if attr_len != 0 {
                        bail!("Synthetic attribute length was not 0");
                    }
                    Synthetic
                }
                unrecognized => self.parse_unrec_attr(attr_len, unrecognized, "field")?,
            };
            validate_cursor(self.position(), cursor + attr_len as u64)?;
            attributes.push(attribute)
        }
        Ok(attributes)
    }

    pub(crate) fn parse_interfaces(&mut self) -> Result<Vec<u16>> {
        let interfaces_len = self.read_u16()?;
        let interfaces = (0..interfaces_len)
            .map(|_| self.read_u16())
            .collect::<Result<Vec<u16>>>()?;
        Ok(interfaces)
    }

    pub(crate) fn parse_class_attr(
        &mut self,
        cp: &ConstantPool,
        attr_count: usize,
        permit_sub: bool,
    ) -> Result<Vec<Attribute>> {
        let mut src_dbg_ext = false;
        let mut btstrp_mthds = false;
        let mut record = false;
        let mut contains_permitted = false;
        let mut attributes = Vec::with_capacity(attr_count);
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
            let attribute = match name.as_bytes() {
                INNER_CLASSES => self.parse_inner_class_attr(cp)?,
                ENCLOSING_METHOD => EnclosingMethod {
                    class_index: self.read_u16()?,
                    method_index: self.read_u16()?,
                },
                SOURCE_FILE => {
                    let file_index = self.read_u16()?;
                    Self::verify_utf8(
                        cp,
                        file_index as usize,
                        "SourceFile index didn't point to a UTF8 in the constant pool.",
                    )?;
                    SourceFile(file_index)
                }
                SOURCE_DEBUG_EXTENSION => {
                    if src_dbg_ext {
                        bail!(
                            "ClassFile Attributes cannot have more than one SourceDebugExtension."
                        );
                    }
                    src_dbg_ext = true;
                    SourceDebugExtension(self.read_bytes(attr_len)?)
                }
                BOOTSTRAP_METHODS => self.parse_btstrp_methods_attr(&mut btstrp_mthds)?,
                MODULE => self.parse_module_attr(cp)?,
                MODULE_PACKAGES => self.parse_module_pckgs_attr(cp)?,
                MODULE_MAIN_CLASS => {
                    let main_class_index = self.read_u16()?;
                    if !matches!(cp.get(main_class_index as usize), Some(Class { .. })) {
                        bail!(
                            "ModuleMainClass' index didn't point to a Class in the constant pool."
                        );
                    }
                    ModuleMainClass(main_class_index)
                }
                NEST_HOST => {
                    let host_index = self.read_u16()?;
                    if !matches!(cp.get(host_index as usize), Some(Class { .. })) {
                        bail!("NestHost attribute's host_index didn't point to a Class in the constant pool.");
                    }
                    NestHost(host_index)
                }
                NEST_MEMBERS => {
                    let classes_len = self.read_u16()? as usize;
                    let classes =
                        self.read_classes(classes_len, cp, "NestedMembers' classes entry")?;
                    NestMembers(classes)
                }
                RECORD => self.parse_record_attr(cp, &mut record)?,
                PERMITTED_SUBCLASSES => {
                    if !permit_sub {
                        bail!("Class attributes contained a PermittedSubclass attribute for a 'final' class.");
                    } else if contains_permitted {
                        bail!("Multiple PermittedSubclasses attributes found in class attributes.");
                    }
                    let classes_len = self.read_u16()? as usize;
                    let classes = self.read_classes(classes_len, cp, "PermittedSubclasses")?;
                    PermittedSubclasses(classes)
                }
                unrecognized => self.parse_unrec_attr(attr_len, &unrecognized, "method")?,
            };
            validate_cursor(self.position(), cursor + attr_len as u64)?;
            attributes.push(attribute);
        }
        Ok(attributes)
    }

    fn parse_record_attr(
        &mut self,
        cp: &ConstantPool,
        rcrd_parsed: &mut bool,
    ) -> Result<Attribute> {
        if *rcrd_parsed {
            bail!("There can only be at most one Record attribute per class.");
        }
        *rcrd_parsed = true;
        let components_len = self.read_u16()? as usize;
        let components = (0..components_len)
            .map(|_| {
                let name_index = self.read_u16()?;
                Self::verify_utf8(
                    cp,
                    name_index as usize,
                    "name_index in RecordComponent struct didn't point to a UTF8",
                )?;
                let descriptor_index = self.read_u16()?;
                let attr_len = self.read_u16()? as usize;
                let attributes = self.parse_class_attr(cp, attr_len, false)?;
                Ok(RecordComponent {
                    name_index,
                    descriptor_index,
                    attributes,
                })
            })
            .collect::<Result<Vec<RecordComponent>>>()?;

        Ok(Record(components))
    }

    fn parse_module_pckgs_attr(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let package_len = self.read_u16()? as usize;
        let package_index = (0..package_len).map(|_| {
            let index = self.read_u16()?;
            if !matches!(cp.get(index as usize), Some(Package { .. })) {
                bail!("ModulePackages attribute's package_index entry didn't point to a Package in the constant pool.");
            }
            Ok(index)
        }).collect::<Result<Vec<u16>>>()?;
        Ok(ModulePackages(package_index))
    }

    fn parse_btstrp_methods_attr(&mut self, btstr_prased: &mut bool) -> Result<Attribute> {
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

    fn parse_inner_class_attr(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let class_len = self.read_u16()? as usize;
        let mut classes = Vec::with_capacity(class_len);
        for _ in 0..class_len {
            let inner_class_info_index = self.read_u16()?;
            if !matches!(cp.get(inner_class_info_index as usize), Some(Class { .. })) {
                bail!("ClassFile InnerClass Attributes inner_class_info_index didn't point to a UTF8 in the constant pool.");
            }
            let outer_class_info_index = self.read_u16()?;
            if !matches!(cp.get(inner_class_info_index as usize), Some(Class { .. })) {
                bail!("ClassFile InnerClass Attributes outer_class_info_index didn't point to a UTF8 in the constant pool.");
            }
            let inner_name_index = self.read_u16()?;
            Self::verify_utf8(
                cp,
                inner_name_index as usize,
                "ClassFile InnerClass Attribute's inner_name_index didn't point to a UTF8 in constant pool.")?;

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

    fn parse_module_attr(&mut self, cp: &ConstantPool) -> Result<Attribute> {
        let module_name_index = self.read_u16()?;

        let flag = self.read_u16()?;
        let module_flags =
            ModuleFlags::from_bits(flag).context(format!("Invalid Module flag: {flag}"))?;

        let module_ver_index = self.read_u16()?;

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

        Ok(Attribute::Module {
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

    fn parse_requires(&mut self, cp: &ConstantPool) -> Result<Requires> {
        let requires_index = self.read_u16()?;
        if !matches!(cp.get(requires_index as usize), Some(Module { .. })) {
            bail!(
                "Requires' requires_index didn't point to a Module constant in the constant pool."
            );
        }
        let flag = self.read_u16()?;
        let requires_flags =
            RequiresFlags::from_bits(flag).context(format!("Invalid Requires flag: {flag}"))?;
        let requires_ver_index = self.read_u16()?;
        Self::verify_utf8(
            cp,
            requires_ver_index as usize,
            "Requires_ver_index didn't point to a UTF8 in the constant pool.",
        )?;
        Ok(Requires {
            requires_index,
            requires_flags,
            requires_ver_index,
        })
    }

    fn parse_exports(&mut self, cp: &ConstantPool) -> Result<Exports> {
        let exports_index = self.read_u16()?;
        if !matches!(cp.get(exports_index as usize), Some(Package { .. })) {
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
                let const_index = self.read_u16()?;
                if !matches!(cp.get(const_index as usize), Some(Package { .. })) {
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

    fn parse_opens(&mut self, cp: &ConstantPool) -> Result<Opens> {
        let opens_index = self.read_u16()?;
        if !matches!(cp.get(opens_index as usize), Some(Package { .. })) {
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

    fn parse_provides(&mut self, cp: &ConstantPool) -> Result<Provides> {
        let provides_index = self.read_u16()?;
        if !matches!(cp.get(provides_index as usize), Some(Class { .. })) {
            bail!("Provides' provides_index didn't point to a Class in the constant pool.");
        }

        let provides_len = self.read_u16()? as usize;
        let provides_with_index = self.read_classes(provides_len, cp, "Provides struct")?;
        Ok(Provides {
            provides_index,
            provides_with_index,
        })
    }

    fn parse_annotations(&mut self, cp: &ConstantPool) -> Result<Annotation> {
        let type_index = self.read_u16()?;
        Self::verify_utf8(
            cp,
            type_index as usize,
            "Annotation's type_index didn't point to a UTF8.",
        )?;
        let value_pairs = {
            let pairs_len = self.read_u16()? as usize;
            let mut value_pairs = Vec::with_capacity(pairs_len);
            for _ in 0..pairs_len {
                let el_name_index = self.read_u16()?;
                Self::verify_utf8(
                    cp,
                    el_name_index as usize,
                    "ElementPair's element_name_index didn't point to a UTF8 in the constant pool.",
                )?;
                let element_value = self.parse_element_value(cp)?;
                value_pairs.push(ElementPairs(el_name_index, element_value));
            }
            value_pairs
        };
        Ok(Annotation {
            type_index,
            value_pairs,
        })
    }

    #[inline]
    pub(crate) fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        (0..length)
            .map(|_| self.read_u8())
            .collect::<Result<Vec<u8>>>()
    }

    #[inline]
    pub(crate) fn read_u16s(&mut self, length: usize) -> Result<Vec<u16>> {
        (0..length)
            .map(|_| self.read_u16())
            .collect::<Result<Vec<u16>>>()
    }

    fn read_classes(
        &mut self,
        length: usize,
        cp: &ConstantPool,
        context: &'static str,
    ) -> Result<Vec<u16>> {
        let mut classes = Vec::with_capacity(length);
        for _ in 0..length {
            let class = self.read_u16()?;
            if !matches!(cp.get(class as usize), Some(Class { .. })) {
                bail!("While parsing {context}, class index didn't point to a Class in the constant pool.");
            }
            classes.push(class);
        }
        Ok(classes)
    }

    #[inline]
    fn verify_utf8(cp: &ConstantPool, index: usize, message: &'static str) -> Result<()> {
        if cp.get_utf8(index).is_err() {
            bail!(message);
        }
        Ok(())
    }

    fn parse_element_value(&mut self, cp: &ConstantPool) -> Result<ElementValue> {
        let tag = self.read_u8()? as char;
        let element_value = match tag {
            'B' | 'C' | 'I' | 'Z' | 'S' => {
                let const_type_index = self.read_u16()?;
                if !matches!(cp.get(const_type_index as usize), Some(Integer(_))) {
                    bail!("const_type_index didn't point to a Integer in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'D' => {
                let const_type_index = self.read_u16()?;
                if !matches!(cp.get(const_type_index as usize), Some(Double(_))) {
                    bail!("const_type_index didn't point to a Double in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'F' => {
                let const_type_index = self.read_u16()?;
                if !matches!(cp.get(const_type_index as usize), Some(Float(_))) {
                    bail!("const_type_index didn't point to a Float in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'J' => {
                let const_type_index = self.read_u16()?;
                if !matches!(cp.get(const_type_index as usize), Some(Long(_))) {
                    bail!("const_type_index didn't point to a Long in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            's' => {
                let const_type_index = self.read_u16()?;
                if !matches!(cp.get(const_type_index as usize), Some(UTF8(_))) {
                    bail!("const_type_index didn't point to a Float in the constant pool.");
                }
                ConstIndex(const_type_index)
            }
            'e' => {
                let type_name_index = self.read_u16()?;
                Self::verify_utf8(
                    cp,
                    type_name_index as usize,
                    "type_name_index didn't point to a UTF8 in the constant pool.",
                )?;

                let const_name_index = self.read_u16()?;
                Self::verify_utf8(
                    cp,
                    const_name_index as usize,
                    "const_name_index didn't point to a UTF8 in the constant pool.",
                )?;

                ElementValue::EnumConst(type_name_index, const_name_index)
            }
            'c' => ClassIndex(self.read_u16()?),
            '@' => AnnotationValue(self.parse_annotation(cp)?),
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

    fn parse_annotation(&mut self, cp: &ConstantPool) -> Result<Annotation> {
        let type_index = self.read_u16()?;
        let value_pairs = {
            let pairs_len = self.read_u16()? as usize;
            let mut value_pairs = Vec::with_capacity(pairs_len);
            for _ in 0..pairs_len {
                let name_index = self.read_u16()?;
                Self::verify_utf8(
                    cp,
                    name_index as usize,
                    "Annotation's element_name_index didn't point to a UTF8 in the constant pool.",
                )?;
                let el_value = self.parse_element_value(cp)?;
                value_pairs.push(ElementPairs(name_index, el_value));
            }
            value_pairs
        };
        Ok(Annotation {
            type_index,
            value_pairs,
        })
    }
    fn parse_jvm_utf8(bytes: &[u8]) -> Result<String> {
        let mut result = String::new();
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];
            if byte & 0x80 == 0 {
                // Single-byte character
                result.push(byte as char);
                i += 1;
            } else if byte & 0xE0 == 0xC0 {
                // Two-byte character
                if i + 1 < bytes.len() {
                    let ch = ((byte & 0x1F) as u32) << 6 | (bytes[i + 1] & 0x3F) as u32;
                    result.push(match char::from_u32(ch) {
                        None => {
                            bail!("Could not convert u32 to char.");
                        }
                        Some(c) => c,
                    });
                    i += 2;
                } else {
                    break;
                }
            } else if byte & 0xF0 == 0xE0 {
                // Three-byte character
                if i + 2 < bytes.len() {
                    let ch = ((byte & 0x0F) as u32) << 12
                        | ((bytes[i + 1] & 0x3F) as u32) << 6
                        | (bytes[i + 2] & 0x3F) as u32;
                    result.push(match char::from_u32(ch) {
                        None => {
                            bail!("Could not convert u32 to char.");
                        }
                        Some(c) => c,
                    });
                    i += 3;
                } else {
                    break;
                }
            } else {
                bail!("Invalid UTF-8 sequence encountered.");
            }
        }
        Ok(result)
    }
}

impl Deref for ClassReader {
    type Target = Cursor<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClassReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn log_unrec_attr(name: &str, context: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("./unrec_attrs.txt")?;

    let message = format!(
        "{:?}: Unrecognized attribute name {name} while parsing {context} attributes.\n",
        SystemTime::now()
    );
    file.write_all(message.as_bytes())?;
    Ok(())
}

#[inline]
fn validate_cursor(curr: u64, expect: u64) -> Result<()> {
    if curr != expect {
        bail!("Invalid cursor position. Current pos: {curr}. Expected pos: {expect}");
    }
    Ok(())
}
