use byteorder::{BigEndian, ReadBytesExt};

use crate::access_flags::MethodAccessFlags;
use crate::attribute::{Attribute, AttributeParser};
use crate::class_reader::ClassReader;
use crate::constant_pool::ConstantPool;

#[derive(Debug)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub attributes: Vec<Attribute>,
}

impl AttributeParser for Method {
    fn parse_attr(
        constant_pool: &ConstantPool,
        class_reader: &mut ClassReader,
        count: u16,
    ) -> Result<Vec<Attribute>, String> {
        for _ in 0..=count {
            let name_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
            let length = class_reader.read_u32::<BigEndian>().unwrap();
            let name = constant_pool.get_utf8(name_index).unwrap();
            match name.as_bytes() {
                b"Code" => {}
                _ => {}
            }
        }
        Err(String::new())
    }
}
