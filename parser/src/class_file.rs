use std::path::Path;

use anyhow::{bail, Context, Result};
use bitflags::Flags;
use byteorder::ReadBytesExt;

use crate::{
    attribute::ClassFileAttributes, class_reader::ClassReader, constant_pool::ConstantPool,
    field::Field, flags::ClassAccessFlags, method::Method,
};

/// Represents the format of a JVM `.class` file
#[derive(Debug, Default, Clone)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub cp: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: usize,
    pub super_class: usize,
    pub interfaces: Vec<usize>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: ClassFileAttributes,
}

impl ClassFile {
    const MAGIC: u32 = 0xCAFEBABE;

    fn parse(mut cr: ClassReader) -> Result<ClassFile> {
        let mut class_file = ClassFile::default();

        Self::valid_magic(&mut cr)?;
        class_file.minor_version = cr.read_u16()?;
        class_file.major_version = cr.read_u16()?;
        let cp = cr.parse_cp()?;
        let flag = cr.read_u16()?;
        class_file.access_flags = ClassAccessFlags::from_bits(flag)
            .context(format!("Invalid Class access flag: {flag}."))?;
        class_file.this_class = cr.read_u16()? as usize;
        class_file.super_class = cr.read_u16()? as usize;
        class_file.interfaces = cr.parse_interfaces()?;
        class_file.fields = cr.parse_fields(&cp)?;
        let method_count = cr.read_u16()? as usize;
        class_file.methods = cr.parse_methods(
            &cp,
            method_count,
            class_file.major_version,
            class_file.minor_version,
        )?;
        let attributes_count = cr.read_u16()? as usize;
        cr.parse_class_attr(
            &cp,
            attributes_count,
            !class_file.access_flags.contains(ClassAccessFlags::FINAL),
            &mut class_file.attributes,
        )?;
        class_file.cp = cp;
        Ok(class_file)
    }

    pub fn parse_class<P: AsRef<Path>>(path: P) -> Result<ClassFile> {
        let cr = ClassReader::new(path.as_ref())?;
        Self::parse(cr)
    }

    pub fn parse_from_buffer(buffer: &[u8]) -> Result<ClassFile> {
        let cr = ClassReader::with_buffer(buffer);
        Self::parse(cr)
    }

    fn valid_magic(cr: &mut ClassReader) -> Result<()> {
        let magic = cr.read_u32()?;
        if magic != ClassFile::MAGIC {
            bail!("Invalid class file.".to_string());
        }
        Ok(())
    }
    pub fn get_utf8(&self, index: usize) -> Result<&str> { self.cp.get_utf8(index) }
}
