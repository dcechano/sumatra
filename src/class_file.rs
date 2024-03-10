use crate::attribute::Attribute;
use crate::class_file::Constant::{
    Class, Double, Dummy, Dynamic, FieldRef, Float, Integer, InterfaceMethodRef, InvokeDynamic,
    Long, MethodHandle, MethodRef, MethodType, Module, NameAndType, Package, UTF8,
};
use crate::constant::Constant;
use crate::field::Field;
use crate::method::Method;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Cursor, Read};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

/// Represents the format of a JVM `.class` file
#[derive(Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Constant>,
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
        match path.as_ref().extension() {
            None => {
                return Err("Path to file must be a .class file.".into());
            }
            Some(ext) => {
                if ext.as_bytes() != b"class" {
                    return Err("Path to file must be a .class file.".into());
                }
            }
        };
        let mut class_reader = {
            let mut buffer = Vec::new();
            let mut file = match File::open(path.as_ref()) {
                Ok(file) => file,
                Err(_) => {
                    panic!();
                }
            };
            file.read_to_end(&mut buffer).unwrap();
            Cursor::new(buffer)
        };

        let magic = class_reader.read_u32::<BigEndian>().unwrap();
        let minor_version = class_reader.read_u16::<BigEndian>().unwrap();
        let major_version = class_reader.read_u16::<BigEndian>().unwrap();
        let constant_pool_count = class_reader.read_u16::<BigEndian>().unwrap() as usize;
        let constant_pool = Self::parse_cp(constant_pool_count, &mut class_reader).unwrap();
        let access_flags = class_reader.read_u16::<BigEndian>().unwrap();
        let this_class = class_reader.read_u16::<BigEndian>().unwrap();
        let super_class = class_reader.read_u16::<BigEndian>().unwrap();
        let interfaces_count = class_reader.read_u16::<BigEndian>().unwrap();
        let interfaces = Self::parse_interfaces(interfaces_count as usize, &mut class_reader)?;
        let fields_count = class_reader.read_u16::<BigEndian>().unwrap();
        let fields = Field::parse_fields(fields_count, &mut class_reader).unwrap();
        let method_count = class_reader.read_u16::<BigEndian>().unwrap();
        let methods = Method::parse_methods(method_count as usize, &mut class_reader).unwrap();
        let attributes_count = class_reader.read_u16::<BigEndian>().unwrap();
        let attributes = Attribute::parse_attributes(attributes_count as usize, &mut class_reader)?;

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

    fn parse_cp(
        cp_count: usize,
        class_reader: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<Constant>, String> {
        let mut constant_pool = Vec::with_capacity(cp_count);
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

    fn parse_interfaces(
        interfaces_count: usize,
        class_reader: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<u16>, String> {
        let mut interfaces = Vec::with_capacity(interfaces_count);
        for _ in 0..interfaces_count {
            interfaces.push(class_reader.read_u16::<BigEndian>().unwrap());
        }
        Ok(interfaces)
    }

    pub(crate) fn parse_bytes(
        length: usize,
        class_reader: &mut Cursor<Vec<u8>>,
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
}
