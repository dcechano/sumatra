use crate::access_flags::FieldAccessFlags;
use crate::attribute::Attribute;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug)]
pub struct Field {
    pub access_flags: FieldAccessFlags,
    pub name_index: usize,
    pub descriptor_index: usize,
    pub attributes: Vec<Attribute>,
}

impl Field {
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
            let attributes =
                Attribute::parse_attributes(attributes_count as usize, class_reader).unwrap();
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
}
