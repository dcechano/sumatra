use anyhow::{bail, Result};
use std::{ops::Deref, str::FromStr};

//TODO refactor APIs to implement FromStr or parse or something, where
// possible.

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub enum Primitive {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    #[default]
    Invalid,
}

impl From<u8> for Primitive {
    fn from(token: u8) -> Self {
        match token {
            b'B' => Primitive::Byte,
            b'C' => Primitive::Char,
            b'D' => Primitive::Double,
            b'F' => Primitive::Float,
            b'I' => Primitive::Int,
            b'J' => Primitive::Long,
            b'S' => Primitive::Short,
            b'Z' => Primitive::Boolean,
            _ => panic!("Invalid primitive token."),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub enum FieldType {
    Base(Primitive),
    Object(String),
    Array(ArrayType, usize),
    #[default]
    Invalid,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ArrayType {
    Object(String),
    Primitive(Primitive),
}

impl FromStr for FieldType {
    type Err = anyhow::Error;

    fn from_str(desc: &str) -> Result<Self> {
        if desc.is_empty(){
            bail!("Invalid FieldType: Descriptor cannot be empty.");
        }
        let bytes = desc.as_bytes();
        match bytes[0] {
            b'L' => {
                let (object, remaining) = Self::parse_object(desc)?;
                if !remaining.is_empty(){
                    bail!("FieldType had text remaining after parse");
                    
                }
                Ok(FieldType::Object(object))
            }
            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' => {
                let primitive = Primitive::from(bytes[0]);
                let remaining = if desc.len() == 1 { "" } else { &desc[1..] };
                if !remaining.is_empty(){
                    bail!("FieldType had text remaining after parse");

                }
                Ok(FieldType::Base(primitive))
            }
            b'[' => {
                let (array, len, remaining) = Self::parse_array(desc)?;
                if !remaining.is_empty(){
                    bail!("FieldType had text remaining after parse");

                }
                Ok(FieldType::Array(array, len))
            }
            token => bail!(
                "Invalid field type descriptor: invalid token '{}'.",
                token as char
            ),
        }
    }
}

impl FieldType {
    pub fn parse_and_remainder(desc: &str) -> Result<(Self, &str)> {
        let bytes = desc.as_bytes();
        if desc.is_empty(){
            bail!("Invalid FieldType: Descriptor cannot be empty.");
            
        }
        match bytes[0] {
            b'L' => {
                let (object, remaining) = Self::parse_object(desc)?;
                Ok((FieldType::Object(object), remaining))
            }
            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' => {
                let primitive = Primitive::from(bytes[0]);
                let remaining = if desc.len() == 1 { "" } else { &desc[1..] };
                Ok((FieldType::Base(primitive), remaining))
            }
            b'[' => {
                let (array, len, remaining) = Self::parse_array(desc)?;
                Ok((FieldType::Array(array, len), remaining))
            }
            token => bail!(
                "Invalid field type descriptor: invalid token '{}'.",
                token as char
            ),
        }
    }

    fn parse_object(desc: &str) -> Result<(String, &str)> {
        if desc.len() <= 2 {
            bail!("Invalid object descriptor: input was not long enough to be a valid descriptor.");
        } else if !desc.starts_with('L') {
            bail!("Invalid object descriptor: input did not start with 'L'.");
        }
        let bytes = desc.as_bytes();
        match desc.find(';') {
            None => bail!("Invalid object descriptor: No terminating ';'."),
            Some(index) => {
                let string = bytes[1..=index]
                    .iter()
                    .map(|byte| *byte as char)
                    .collect::<String>();
                let remaining = if index + 1 < bytes.len() {
                    &desc[index + 1..]
                } else {
                    ""
                };
                Ok((string, remaining))
            }
        }
    }
    
    fn parse_array(desc: &str) -> Result<(ArrayType, usize, &str)> {
        if desc.len() <= 2 {
            bail!("Invalid array descriptor: descriptor not long enough.");
        }
        let bytes = desc.as_bytes();
        let mut i = 0;
        while bytes[i] == b'[' {
            i += 1;
            if i == bytes.len() {
                bail!("Invalid array descriptor: No Class provided.");
            }
        }

        match bytes[i] {
            b'L' => {
                let (name, remaining) = Self::parse_object(&desc[i..])?;
                Ok((ArrayType::Object(name), i, remaining))
            }
            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' => {
                let remaining = if i + 1 == desc.len() {
                    ""
                } else {
                    &desc[i + 1..]
                };
                Ok((
                    ArrayType::Primitive(Primitive::from(bytes[i])),
                    i,
                    remaining,
                ))
            }
            token => bail!(
                "Invalid array descriptor: Invalid token '{}'.",
                token as char
            ),
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub enum ReturnDescriptor {
    NonVoid(FieldType),
    Void,
    #[default]
    Invalid,
}

impl FromStr for ReturnDescriptor {
    type Err = anyhow::Error;

    fn from_str(desc: &str) -> Result<Self> {
        if desc.is_empty(){
            bail!("Invalid return descriptor: Descriptor cannot be empty.");
        }
        if desc.starts_with('V') {
            if desc.len() != 1 {
                bail!("Invalid return descriptor: Nothing should follow a 'V' token.");
            }
            return Ok(Self::Void);
        }

        let (f_type, remainder) = FieldType::parse_and_remainder(desc)?;
        if !remainder.is_empty() {
            bail!("Invalid return token: Nothing should follow the field type.");
        }
        Ok(Self::NonVoid(f_type))    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct Params(Vec<FieldType>);

impl Deref for Params {
    type Target = Vec<FieldType>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
pub struct MethodDescriptor(Params, ReturnDescriptor);

impl FromStr for MethodDescriptor {
    type Err = anyhow::Error;

    fn from_str(desc: &str) -> Result<Self> {
        if desc.is_empty(){
            bail!("Invalid method descriptor: Descriptor cannot be empty.");

        }
        if !desc.starts_with('(') {
            bail!("Invalid method descriptor: Expected '('.");
        }
        if desc.len() < 2 {
            bail!("Invalid method descriptor: descriptor is too short.");
        }
        let mut remaining = &desc[1..];
        let mut params = Vec::new();
        while !remaining.is_empty() && !remaining.starts_with(')') {
            let (f_type, remainder) = FieldType::parse_and_remainder(remaining)?;
            remaining = remainder;
            params.push(f_type);
        }
        // &remaining[1..] to skip over the ')'
        let r_type = remaining[1..].parse::<ReturnDescriptor>()?;
        Ok(Self(Params(params), r_type))
    }
}

#[derive(Debug, Default, Eq, PartialEq, Clone)]
pub struct FieldDescriptor(FieldType);

impl FromStr for FieldDescriptor {
    type Err = anyhow::Error;

    fn from_str(desc: &str) -> Result<Self> {
        let f_type = desc.parse::<FieldType>()?;
        Ok(Self(f_type))
    }
}

#[cfg(test)]
mod tests {
    use crate::desc_types::{
        ArrayType,
        ArrayType::{Object, Primitive as ArrayPrimitive},
        FieldDescriptor, FieldType, MethodDescriptor, Params, Primitive, ReturnDescriptor,
    };

    const STRING: &str = "java/lang/String;";
    const OBJECT: &str = "java/lang/Object;";
    const X509: &str = "javax/security/cert/X509Certificate;";

    #[test]
    fn test_ftype_object() {
        let desc = format!("{}{}", "L", STRING);
        let f_type= desc.parse::<FieldType>().unwrap();
        assert_eq!(f_type, FieldType::Object(STRING.to_string()))
    }

    #[test]
    fn test_ftype_base() {
        let desc = "I";
        let f_type= desc.parse::<FieldType>().unwrap();
        assert_eq!(f_type, FieldType::Base(Primitive::Int))
    }

    #[test]
    fn test_ftype_array_object() {
        let desc = "[[Ljava/lang/String;";
        let (f_type, remainder) = FieldType::parse_and_remainder(desc).unwrap();
        assert!(remainder.is_empty());
        assert_eq!(f_type, FieldType::Array(Object(STRING.to_string()), 2))
    }

    #[test]
    fn test_ftype_array_primitive() {
        let desc = "[[[J";
        let f_type= desc.parse::<FieldType>().unwrap();
        assert_eq!(f_type, FieldType::Array(ArrayPrimitive(Primitive::Long), 3))
    }

    #[test]
    fn test_return_desc_void() {
        let desc = "V";
        let r_type = desc.parse::<ReturnDescriptor>().unwrap();
        assert_eq!(r_type, ReturnDescriptor::Void);
    }

    #[test]
    fn test_return_desc_non_void() {
        let desc = "[[[[[[[[[[Z";
        let r_type = desc.parse::<ReturnDescriptor>().unwrap();
        assert_eq!(
            r_type,
            ReturnDescriptor::NonVoid(FieldType::Array(
                ArrayType::Primitive(Primitive::Boolean),
                10
            ))
        );
    }

    #[test]
    fn test_method_desc_non_void() {
        let desc = "(Ljava/lang/String;I)Z";
        let should_be = MethodDescriptor(
            Params(vec![
                FieldType::Object(STRING.to_string()),
                FieldType::Base(Primitive::Int),
            ]),
            ReturnDescriptor::NonVoid(FieldType::Base(Primitive::Boolean)),
        );
        let m_desc = desc.parse::<MethodDescriptor>().unwrap();
        assert_eq!(m_desc, should_be);
    }

    #[test]
    fn test_method_desc2_non_void() {
        let desc = "(Ljava/lang/String;IJSBLjava/lang/Object;)[[Z";
        let should_be = MethodDescriptor(
            Params(vec![
                FieldType::Object(STRING.to_string()),
                FieldType::Base(Primitive::Int),
                FieldType::Base(Primitive::Long),
                FieldType::Base(Primitive::Short),
                FieldType::Base(Primitive::Byte),
                FieldType::Object(OBJECT.to_string()),
            ]),
            ReturnDescriptor::NonVoid(FieldType::Array(
                ArrayType::Primitive(Primitive::Boolean),
                2,
            )),
        );
        let m_desc = desc.parse::<MethodDescriptor>().unwrap();
        assert_eq!(m_desc, should_be);
    }

    #[test]
    fn test_method_desc3_non_void() {
        let desc = "([B)Ljavax/security/cert/X509Certificate;";
        let should_be = MethodDescriptor(
            Params(vec![FieldType::Array(
                ArrayType::Primitive(Primitive::Byte),
                1,
            )]),
            ReturnDescriptor::NonVoid(FieldType::Object(X509.to_string())),
        );
        let m_desc = desc.parse::<MethodDescriptor>().unwrap();
        assert_eq!(m_desc, should_be);
    }

    #[test]
    fn test_field_desc() {
        let desc = "[[[J";
        let should_be = FieldDescriptor(FieldType::Array(ArrayType::Primitive(Primitive::Long), 3));
        let f_desc = desc.parse::<FieldDescriptor>().unwrap();
        assert_eq!(f_desc, should_be);
    }

    #[test]
    #[should_panic]
    fn invalid_field_desc_empty() {
        let desc = "";
        let _ = desc.parse::<FieldDescriptor>().unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_field_desc_array() {
        let desc = "[";
        let _ = desc.parse::<FieldDescriptor>().unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_ftype() {
        let desc = "java/lang/Object";
        let _ = desc.parse::<FieldType>().unwrap();
    }


    #[test]
    #[should_panic]
    fn invalid_ftype_empty() {
        let desc = "";
        let _ = desc.parse::<FieldType>().unwrap();
    }


    #[test]
    #[should_panic]
    fn invalid_method_desc() {
        let desc = "(Ljava/lang/Object;V";
        let _ = desc.parse::<MethodDescriptor>().unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_method_desc2() {
        let desc = "()";
        let _ = desc.parse::<MethodDescriptor>().unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_method_desc_empty() {
        let desc = "";
        let _ = desc.parse::<MethodDescriptor>().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn invalid_return_desc_extra_text() {
        let desc = "IJ";
        let _ = desc.parse::<ReturnDescriptor>().unwrap();
    }
    
    #[test]
    #[should_panic]
    fn invalid_return_desc_empty() {
        let desc = "";
        let _ = desc.parse::<ReturnDescriptor>().unwrap();
    }
}
