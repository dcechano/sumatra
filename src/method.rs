use crate::access_flags::MethodAccessFlags;
use crate::attribute::Attribute;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct Method {
    pub access_flags: MethodAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub attributes: Vec<Attribute>,
}

impl Method {
    pub(crate) fn parse_methods(
        method_count: usize,
        class_reader: &mut Cursor<Vec<u8>>,
    ) -> Result<Vec<Method>, String> {
        let mut methods = Vec::with_capacity(method_count);
        for _ in 0..method_count {
            let access_flags =
                MethodAccessFlags::from_bits(class_reader.read_u16::<BigEndian>().unwrap())
                    .unwrap();
            let name_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
            let descriptor_index = class_reader.read_u16::<BigEndian>().unwrap() as usize;
            let attributes_count = class_reader.read_u16::<BigEndian>().unwrap();
            let attributes = Attribute::parse_attributes(attributes_count as usize, class_reader)?;

            methods.push(Method {
                access_flags,
                name_index,
                descriptor_index,
                attributes,
            });
        }
        Ok(methods)
    }
}
