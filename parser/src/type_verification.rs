use anyhow::{bail, Result};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum VType {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitThis,
    Object(usize),
    UninitVar(usize),
    Dummy,
}

impl TryFrom<u8> for VType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        let v_type = match value {
            0 => VType::Top,
            1 => VType::Integer,
            2 => VType::Float,
            3 => VType::Double,
            4 => VType::Long,
            5 => VType::Null,
            6 => VType::UninitThis,
            7 => VType::Object(0),
            8 => VType::UninitVar(0),
            invalid => {
                bail!("Invalid byte {invalid} for VType.")
            }
        };
        Ok(v_type)
    }
}
