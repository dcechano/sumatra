use std::cmp::min;
use std::io::Cursor;
use crate::access_flags::{FieldAccessFlags, MethodAccessFlags};
use crate::attribute::{Attribute, AttributeParser};
use crate::class_file::Constant::{
    Class, Double, Dummy, Dynamic, FieldRef, Float, Integer, InterfaceMethodRef, InvokeDynamic,
    Long, MethodHandle, MethodRef, MethodType, Module, NameAndType, Package, UTF8,
};
use crate::class_reader::ClassReader;
use crate::constant::Constant;
use crate::constant_pool::ConstantPool;
use crate::field::Field;
use crate::method::Method;
use byteorder::{BigEndian, ReadBytesExt};
use std::path::Path;

/// Represents the format of a JVM `.class` file
#[derive(Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn parse_class<P: AsRef<Path>>(path: P) -> Result<ClassFile, String> {
        let mut class_reader = ClassReader::new(path.as_ref()).unwrap();

        let magic = class_reader.read_u32::<BigEndian>().unwrap();
        let minor_version = class_reader.read_u16::<BigEndian>().unwrap();
        let major_version = class_reader.read_u16::<BigEndian>().unwrap();
        let constant_pool_count = class_reader.read_u16::<BigEndian>().unwrap() as usize;
        let constant_pool = Self::parse_cp(constant_pool_count, &mut class_reader).unwrap();
        let access_flags = class_reader.read_u16::<BigEndian>().unwrap();
        let this_class = class_reader.read_u16::<BigEndian>().unwrap();
        let super_class = class_reader.read_u16::<BigEndian>().unwrap();
        let interfaces_count = class_reader.read_u16::<BigEndian>().unwrap() as usize;
        let interfaces = Self::parse_interfaces(interfaces_count, &mut class_reader)?;
        let fields_count = class_reader.read_u16::<BigEndian>().unwrap();
        let fields = todo!();
        let method_count = class_reader.read_u16::<BigEndian>().unwrap() as usize;
        let methods = Self::parse_methods(
            &constant_pool,
            &mut class_reader,
            method_count,
            minor_version,
            major_version,
        )
        .unwrap();
        let attributes_count = class_reader.read_u16::<BigEndian>().unwrap();
        let attributes =
            ClassFile::parse_attr(&constant_pool, &mut class_reader, attributes_count).unwrap();

        Ok(ClassFile {
            magic,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    fn parse_cp(cp_count: usize, class_reader: &mut ClassReader) -> Result<ConstantPool, String> {
        let mut constant_pool = ConstantPool::new(cp_count);
        constant_pool.push(Dummy);
        for _ in 0..cp_count - 1 {
            let constant = match class_reader.read_u8().unwrap() {
                1 => {
                    let length = class_reader.read_u16::<BigEndian>().unwrap() as usize;
                    let bytes = Self::parse_bytes(length, class_reader).unwrap();
                    let string = ClassFile::parse_jvm_utf8(&bytes);
                    UTF8 { string }
                }
                3 => Integer(class_reader.read_u32::<BigEndian>().unwrap() as i32),
                4 => Float(f32::from(class_reader.read_u16::<BigEndian>().unwrap())),
                5 => Long(class_reader.read_i64::<BigEndian>().unwrap()),
                6 => Double(class_reader.read_f64::<BigEndian>().unwrap()),
                7 => Class {
                    name_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                8 => Constant::String {
                    string_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                9 => FieldRef {
                    class_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                    name_and_type_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                10 => MethodRef {
                    class_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                    name_and_type_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                11 => InterfaceMethodRef {
                    class_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                    name_and_type_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                12 => NameAndType {
                    name_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                    descriptor_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                15 => MethodHandle {
                    reference_kind: class_reader.read_u8().unwrap(),
                    reference_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                16 => MethodType {
                    descriptor_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                17 => Dynamic {
                    bootstrap_method_attr_index: class_reader.read_u16::<BigEndian>().unwrap()
                        as usize,
                    name_and_type_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                18 => InvokeDynamic {
                    bootstrap_method_attr_index: class_reader.read_u16::<BigEndian>().unwrap()
                        as usize,
                    name_and_type_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                19 => Module {
                    name_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                20 => Package {
                    name_index: class_reader.read_u16::<BigEndian>().unwrap() as usize,
                },
                unknown => panic!("Unknown constant: {unknown}"),
            };
            match constant {
                Long(_) | Double(_) => {
                    constant_pool.push(constant);
                    constant_pool.push(Dummy);
                }
                _ => {
                    constant_pool.push(constant);
                }
            }
        }
        Ok(constant_pool)
    }
    fn parse_methods(
        constant_pool: &ConstantPool,
        class_reader: &mut ClassReader,
        method_count: usize,
        min_ver: u16,
        maj_ver: u16,
    ) -> Result<Vec<Method>, String> {
        let mut methods = Vec::with_capacity(method_count);
        for _ in 0..method_count {}
        Ok(methods)
    }

    fn parse_method(
        constant_pool: &ConstantPool,
        class_reader: &mut ClassReader,
        min_ver: u16,
        maj_ver: u16,
    ) -> Result<Method, String> {
        let access_flags =
            MethodAccessFlags::from_bits(class_reader.read_u16::<BigEndian>().unwrap()).unwrap();
            
        // TODO: uncomment when implement
        // access_flags.verify_flags(min_ver, maj_ver, /*is_interface*/)
        
        let name_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
        let descriptor_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
        let attributes_count = class_reader.read_u16::<BigEndian>().unwrap();
        let attributes = Method::parse_attr(constant_pool, class_reader, attributes_count).unwrap();

        Err("Couldn't parse method.".to_string())
    }

    pub(crate) fn parse_fields(
        fields_count: u16,
        class_reader: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<Field>, String> {
        let mut fields = Vec::with_capacity(fields_count as usize);
        for _ in 0..fields_count {
            let access_flags = class_reader.read_u16::<BigEndian>().unwrap();
            let name_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
            let descriptor_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
            let attributes_count = class_reader.read_u16::<BigEndian>().unwrap();
            let attributes = todo!();
            let access_flags = match FieldAccessFlags::from_bits(access_flags) {
                None => {
                    eprintln!("unrecognized Field access flag: {access_flags}");
                    eprintln!("name_index: {name_index}");
                    eprintln!("descriptor_index: {descriptor_index}");
                    panic!();
                }
                Some(access_flags) => access_flags,
            };

            fields.push(Field {
                access_flags,
                name_index,
                descriptor_index,
                attributes,
            });
        }
        Ok(fields)
    }

    fn parse_interfaces(
        interfaces_count: usize,
        class_reader: &mut ClassReader,
    ) -> Result<Vec<u16>, String> {
        let mut interfaces = Vec::with_capacity(interfaces_count);
        for _ in 0..interfaces_count {
            interfaces.push(class_reader.read_u16::<BigEndian>().unwrap());
        }
        Ok(interfaces)
    }

    pub(crate) fn parse_bytes(
        length: usize,
        class_reader: &mut ClassReader,
    ) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            bytes.push(class_reader.read_u8().unwrap());
        }
        Ok(bytes)
    }

    fn parse_jvm_utf8(bytes: &[u8]) -> String {
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
                    result.push(char::from_u32(ch).unwrap());
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
                    result.push(char::from_u32(ch).unwrap());
                    i += 3;
                } else {
                    break;
                }
            } else {
                // Invalid UTF-8 sequence
                break;
            }
        }
        result
    }

    pub(crate) fn get_utf8(&self, index: usize) -> Result<&str, String> {
        self.constant_pool.get_utf8(index)
    }
}

impl AttributeParser for ClassFile {
    fn parse_attr(
        constant_pool: &ConstantPool,
        class_reader: &mut ClassReader,
        count: u16,
    ) -> Result<Vec<Attribute>, String> {
        todo!()
    }
}
