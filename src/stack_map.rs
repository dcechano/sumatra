use crate::type_verification::VType;

#[derive(Debug)]
pub(crate) enum StackMapFrame {
    SameFrame,
    SameLocals,
    SameLocalsExt(u16, VType),
    Chop(u16),
    SameFrameExt(u16),
    Append(u16, Vec<VType>),
    Full {
        offset: u16,
        num_locals: u16,
        locals: Vec<VType>,
        num_stack: u16,
        stack: Vec<VType>,
    },
}

impl TryFrom<u8> for StackMapFrame {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let smf = match value {
            0..=63 => StackMapFrame::SameFrame,
            64..=127 => StackMapFrame::SameLocals,
            247 => StackMapFrame::SameLocalsExt(0, VType::Dummy),
            248..=250 => StackMapFrame::Chop(0),
            251 => StackMapFrame::SameFrameExt(0),
            252..=254 => StackMapFrame::Append(0, vec![]),

            invalid => {
                return Err(format!("Invalid byte {{{invalid}}} for StackMapFrame."));
            }
        };

        Ok(smf)
    }
}
