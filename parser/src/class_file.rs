use std::path::Path;

use anyhow::{bail, Context, Result};
use bitflags::Flags;
use byteorder::ReadBytesExt;

use crate::{
    attribute::Attribute
    ,
    class_reader::ClassReader
    ,
    constant_pool::ConstantPool,
    field::Field,
    flags::ClassAccessFlags,
    method::Method,
};

/// Represents the format of a JVM `.class` file
#[derive(Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub cp: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    const MAGIC: u32 = 0xCAFEBABE;

    pub fn parse_class<P: AsRef<Path>>(path: P) -> Result<ClassFile> {
        let mut cr = ClassReader::new(path.as_ref())?;

        let magic = cr.read_u32()?;
        if magic != ClassFile::MAGIC {
            bail!("Invalid class file.".to_string());
        }
        let minor_version = cr.read_u16()?;
        let major_version = cr.read_u16()?;
        let cp = cr.parse_cp()?;
        let flag = cr.read_u16()?;
        let access_flags = ClassAccessFlags::from_bits(flag)
            .context(format!("Invalid Class access flag: {flag}."))?;
        let this_class = cr.read_u16()?;
        let super_class = cr.read_u16()?;
        let interfaces = cr.parse_interfaces()?;
        let fields = cr.parse_fields(&cp)?;
        let method_count = cr.read_u16()? as usize;
        let methods = cr.parse_methods(&cp, method_count, major_version, minor_version)?;
        let attributes_count = cr.read_u16()? as usize;
        let attributes = cr.parse_class_attr(
            &cp,
            attributes_count,
            !access_flags.contains(ClassAccessFlags::FINAL),
        )?;
        Ok(ClassFile {
            magic,
            minor_version,
            major_version,
            cp,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    pub fn get_utf8(&self, index: usize) -> Result<&str> {
        self.cp.get_utf8(index)
    }
}
