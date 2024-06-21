use std::convert::TryFrom;

use anyhow::{bail, Result};

use crate::{
    class_reader::ClassReader,
    instruction::{
        ArrayType::{Boolean, Byte, Char, Double, Dummy, Float, Int, Long, Short},
        Instruction::*,
        WideInstruction::Invalid,
    },
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Instruction {
    /// Load reference from array
    AaLoad,
    AaStore,
    /// Push null
    AConstNull,
    /// Load reference from local variable
    ALoad(u8),
    /// Load reference from local variable
    ALoad0,
    /// Load reference from local variable
    ALoad1,
    /// Load reference from local variable
    ALoad2,
    /// Load reference from local variable
    ALoad3,
    /// Create new array of reference
    ANewArray(u16),
    /// Return reference on top of stack.
    AReturn,
    /// Put array length on top of stack
    ArrayLength,
    /// Store reference into local variable
    AStore(u8),
    /// Store reference into local variable
    AStore0,
    /// Store reference into local variable
    AStore1,
    /// Store reference into local variable
    AStore2,
    /// Store reference into local variable
    AStore3,
    AThrow,
    BaLoad,
    BaStore,
    /// The immediate byte is sign-extended to an int value. That value is
    /// pushed onto the operand stack.
    BiPush(i8),
    /// Load char from array and put onto stack
    CaLoad,
    /// Store char into char array
    CaStore,
    /// Check whether object is of a given type
    Checkcast(u16),
    D2F,
    D2I,
    D2L,
    /// Add double
    DAdd,
    /// Load double from array
    DaLoad,
    DaStore,
    Dcmpg,
    Dcmpl,
    DConst0,
    DConst1,
    DDiv,
    /// Load double from local variable
    DLoad(u8),
    /// Load double from local variable 0
    DLoad0,
    /// Load double from local variable 1
    DLoad1,
    /// Load double from local variable 2
    DLoad2,
    /// Load double from local variable 3
    DLoad3,
    DMul,
    DNeg,
    DRem,
    /// Return double from current java method
    DReturn,
    /// Store double into local variable n and n + 1.
    DStore(u8),
    /// Store double into local variable 0 and 1.
    DStore0,
    /// Store double into local variable 1 and 2.
    DStore1,
    /// Store double into local variable 2 and 3.
    DStore2,
    /// Store double into local variable 3 and 4.
    DStore3,
    DSub,
    /// Duplicate the value at the top of the operand stack.
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    F2D,
    F2I,
    F2L,
    FAdd,
    FaLoad,
    FaStore,
    Fcmpg,
    Fcmpl,
    FConst0,
    FConst1,
    FConst2,
    FDiv,
    /// Loads float from local at provided index
    FLoad(u8),
    /// Loads float local at index 0
    FLoad0,
    /// Loads float local at index 1
    FLoad1,
    /// Loads float local at index 2
    FLoad2,
    /// Loads float local at index 3
    FLoad3,
    FMul,
    FNeg,
    FRem,
    /// Return float from current java method
    FReturn,
    FStore(u8),
    FStore0,
    FStore1,
    FStore2,
    FStore3,
    FSub,
    /// get field from obj and add to stack,
    /// where the operand is index of class in the constant pool.
    GetField(u16),
    /// get `static` field from class and add to stack,
    /// where the operand is index of class in the constant pool.
    GetStatic(usize),
    /// Branch to instruction at offset.
    GoTo(usize),
    GoToW(usize),
    /// Convert int to byte
    I2B,
    I2C,
    I2D,
    I2F,
    I2L,
    I2S,
    /// Add ints
    IAdd,
    IaLoad,
    IAnd,
    /// Store into int array.
    IaStore,
    /// Push int constant -1.
    IConstM1,
    /// Push int constant 0.
    IConst0,
    /// Push int constant 1.
    IConst1,
    /// Push int constant 2.
    IConst2,
    /// Push int constant 3.
    IConst3,
    /// Push int constant 4.
    IConst4,
    /// Push int constant 5.
    IConst5,
    /// Divide ints
    IDiv,
    IfAcmpeq(usize),
    IfAcmpne(usize),
    IfIcmpeq(usize),
    IfIcmpne(usize),
    IfIcmplt(usize),
    IfIcmpge(usize),
    IfIcmpgt(usize),
    IfIcmple(usize),
    /// Branch to offset if int on operand stack == 0
    Ifeq(usize),
    /// Branch to offset if int on operand stack != 0
    Ifne(usize),
    /// Branch to offset if int on operand stack < 0
    Iflt(usize),
    /// Branch to offset if int on operand stack >= 0
    Ifge(usize),
    /// Branch to offset if int on operand stack > 0
    Ifgt(usize),
    /// Branch to offset if int on operand stack <= 0
    Ifle(usize),
    /// Branch to offset if reference is not null.
    IfNonNull(usize),
    IfNull(usize),
    /// Increment local variable at first index by second argument
    Iinc(u8, i8), // TODO double check the spec on this one. It was confusing.
    /// load local int variable at supplied index and push to operand stack.
    ILoad(u8),
    /// load local int variable at index 0 and push to operand stack.
    ILoad0,
    /// load local int variable at index 1 and push to operand stack.
    ILoad1,
    /// load local int variable at index 2 and push to operand stack.
    ILoad2,
    /// load local int variable at index 3 and push to operand stack.
    ILoad3,
    IMul,
    INeg,
    /// Determine a reference is of a given type.
    InstanceOf(u16),
    InvokeDynamic(u16, u8, u8), // last 2 bytes are always 0
    InvokeInterface(u16, u8, u8),
    /// Invoke instance method; direct invocation of instance initialization
    /// methods and methods of the current class and its supertypes
    InvokeSpecial(u16),
    /// Invoke a class (static) method.
    /// The provided u16 is an index to the run-time constant pool where the
    /// entry is a symbolic reference to a method or interface method.
    InvokeStatic(u16),
    /// Invoke instance method; dispatch based on class
    InvokeVirtual(usize),
    IOr,
    /// Pop two integers off the operand stack, calculate the
    /// remainder and push the result.
    IRem,
    /// Return integer at the top of the operand stack.
    IReturn,
    IShL,
    IShR,
    /// Store int into local variable
    IStore(u8),
    /// Store int into local variable 0.
    IStore0,
    /// Store int into local variable 1.
    IStore1,
    /// Store int into local variable 2.
    IStore2,
    /// Store int into local variable 3.
    IStore3,
    /// Subtract ints and store result
    ISub,
    /// Logical shift int right
    IuShR,
    IxOr,
    Jsr(i16),
    JsrW(i32),
    L2D,
    L2F,
    L2I,
    LAdd,
    LaLoad,
    LAnd,
    LaStore,
    Lcmp,
    LConst0,
    LConst1,
    /// Push item from run-time constant pool.
    Ldc(usize),
    /// Push item from run-time constant pool (wide index).
    LdcW(usize),
    /// Push long or double from run-time constant pool (wide index).
    Ldc2W(usize),
    LDiv,
    LLoad(u8),
    LLoad0,
    LLoad1,
    LLoad2,
    LLoad3,
    LMul,
    LNeg,
    LookUpSwitch {
        default_index: i32,
        jump_table: Vec<(i32, i32)>,
    },
    LOr,
    LRem,
    /// Return long from current java method.
    LReturn,
    LShL,
    LShR,
    LStore(u8),
    LStore0,
    LStore1,
    LStore2,
    LStore3,
    LSub,
    LuShR,
    LxOr,
    /// Enter monitor for object
    MonitorEnter,
    /// Exit monitor for object
    MonitorExit,
    MultiaNewArray(u16, u8),
    /// Create new obj class at provided constant pool index
    New(u16),
    /// Construct a new array for the provided type.
    NewArray(ArrayType),
    Nop,
    /// Pops one value from the operand stack. This instruction is never
    /// used if the value at the top of the stack is a Java long or double.
    Pop,
    /// Pops one or two values from the operand stack.
    Pop2,
    /// Set field in object
    PutField(u16),
    /// Assign value on top of operand stack to the
    /// class field found in the runtime constant pool
    /// at the given index.
    PutStatic(u16),
    Ret(u8),
    /// Used to return from a `void` method.
    Return,
    SaLoad,
    SaStore,
    /// Push short to operand stack.
    SiPush(i16),
    Swap,
    TableSwitch {
        default_index: i32,
        low: i32,
        high: i32,
        jump_offsets: Vec<i32>,
    },
    Wide(WideInstruction),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum WideInstruction {
    Iinc(u16, u16),
    ILoad(u16),
    FLoad(u16),
    ALoad(u16),
    LLoad(u16),
    DLoad(u16),
    IStore(u16),
    FStore(u16),
    AStore(u16),
    LStore(u16),
    DStore(u16),
    Ret(u16),
    Invalid,
}

impl Instruction {
    pub fn from_bytes(bytes: &[u8]) -> Result<Vec<Self>> {
        let mut cursor = ClassReader::with_buffer(bytes);
        let mut instructions = Vec::with_capacity(bytes.len());

        /*
            This vector is to keep track of how many bytes have been processed at any instruction.
            This helps to figure out what offset to give the Goto instruction since the class file
            uses bytes as the Goto index not the instruction index (sumatra uses instruction index).
            If Goto has byte n in bytecode, the instruction index for that byte is at byte_to_instr[n].
        */
        let mut byte_to_instr = vec![0; bytes.len()];
        /*
            The jmp_registry contains all entries in the byte_to_instr that contain a "jmp" instruction.
            the key is index of the instruction in the instructions vector, the value is the byte to jump to.
        */
        let mut jmp_registry = Vec::with_capacity(bytes.len());

        while cursor.position() != bytes.len() as u64 {
            *byte_to_instr.get_mut(cursor.position() as usize).unwrap() = instructions.len();
            let instruction = match cursor.read_u8()? {
                50 => AaLoad,
                83 => AaStore,
                1 => AConstNull,
                25 => ALoad(cursor.read_u8()?),
                42 => ALoad0,
                43 => ALoad1,
                44 => ALoad2,
                45 => ALoad3,
                189 => ANewArray(cursor.read_u16()?),
                176 => AReturn,
                190 => ArrayLength,
                58 => AStore(cursor.read_u8()?),
                75 => AStore0,
                76 => AStore1,
                77 => AStore2,
                78 => AStore3,
                191 => AThrow,
                51 => BaLoad,
                84 => BaStore,
                16 => BiPush(cursor.read_i8()?),
                52 => CaLoad,
                85 => CaStore,
                192 => Checkcast(cursor.read_u16()?),
                144 => D2F,
                142 => D2I,
                143 => D2L,
                99 => DAdd,
                49 => DaLoad,
                82 => DaStore,
                152 => Dcmpg,
                151 => Dcmpl,
                14 => DConst0,
                15 => DConst1,
                111 => DDiv,
                24 => DLoad(cursor.read_u8()?),
                38 => DLoad0,
                39 => DLoad1,
                40 => DLoad2,
                41 => DLoad3,
                107 => DMul,
                119 => DNeg,
                115 => DRem,
                175 => DReturn,
                57 => DStore(cursor.read_u8()?),
                71 => DStore0,
                72 => DStore1,
                73 => DStore2,
                74 => DStore3,
                103 => DSub,
                89 => Dup,
                90 => DupX1,
                91 => DupX2,
                92 => Dup2,
                93 => Dup2X1,
                94 => Dup2X2,
                141 => F2D,
                139 => F2I,
                140 => F2L,
                98 => FAdd,
                48 => FaLoad,
                81 => FaStore,
                150 => Fcmpg,
                149 => Fcmpl,
                11 => FConst0,
                12 => FConst1,
                13 => FConst2,
                110 => FDiv,
                23 => FLoad(cursor.read_u8()?),
                34 => FLoad0,
                35 => FLoad1,
                36 => FLoad2,
                37 => FLoad3,
                106 => FMul,
                118 => FNeg,
                114 => FRem,
                174 => FReturn,
                56 => FStore(cursor.read_u8()?),
                67 => FStore0,
                68 => FStore1,
                69 => FStore2,
                70 => FStore3,
                102 => FSub,
                180 => GetField(cursor.read_u16()?),
                178 => GetStatic(cursor.read_u16()? as usize),
                167 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    GoTo(0)
                }
                200 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    GoToW(0)
                }
                145 => I2B,
                146 => I2C,
                135 => I2D,
                134 => I2F,
                133 => I2L,
                147 => I2S,
                96 => IAdd,
                46 => IaLoad,
                126 => IAnd,
                79 => IaStore,
                2 => IConstM1,
                3 => IConst0,
                4 => IConst1,
                5 => IConst2,
                6 => IConst3,
                7 => IConst4,
                8 => IConst5,
                108 => IDiv,
                165 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfAcmpeq(0)
                }
                166 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfAcmpne(0)
                }
                159 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfIcmpeq(0)
                }
                160 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfIcmpne(0)
                }
                161 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfIcmplt(0)
                }
                162 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfIcmpge(0)
                }
                163 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfIcmpgt(0)
                }
                164 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfIcmple(0)
                }
                153 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    Ifeq(0)
                }
                154 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    Ifne(0)
                }
                155 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    Iflt(0)
                }
                156 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    Ifge(0)
                }
                157 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    Ifgt(0)
                }
                158 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    Ifle(0)
                }
                199 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfNonNull(0)
                }
                198 => {
                    jmp_registry.push((
                        instructions.len(),
                        (cursor.position() - 1) as i16 + cursor.read_i16()?,
                    ));
                    IfNull(0)
                }
                132 => Iinc(cursor.read_u8()?, cursor.read_i8()?),
                21 => ILoad(cursor.read_u8()?),
                26 => ILoad0,
                27 => ILoad1,
                28 => ILoad2,
                29 => ILoad3,
                104 => IMul,
                116 => INeg,
                193 => InstanceOf(cursor.read_u16()?),
                186 => InvokeDynamic(cursor.read_u16()?, cursor.read_u8()?, cursor.read_u8()?),
                185 => InvokeInterface(cursor.read_u16()?, cursor.read_u8()?, cursor.read_u8()?),
                183 => InvokeSpecial(cursor.read_u16()?),
                184 => InvokeStatic(cursor.read_u16()?),
                182 => InvokeVirtual(cursor.read_u16()? as usize),
                128 => IOr,
                112 => IRem,
                172 => IReturn,
                120 => IShL,
                122 => IShR,
                54 => IStore(cursor.read_u8()?),
                59 => IStore0,
                60 => IStore1,
                61 => IStore2,
                62 => IStore3,
                100 => ISub,
                124 => IuShR,
                130 => IxOr,
                168 => Jsr(cursor.read_i16()?),
                201 => JsrW(cursor.read_i32()?),
                138 => L2D,
                137 => L2F,
                136 => L2I,
                97 => LAdd,
                47 => LaLoad,
                127 => LAnd,
                80 => LaStore,
                148 => Lcmp,
                9 => LConst0,
                10 => LConst1,
                18 => Ldc(cursor.read_u8()? as usize),
                19 => LdcW(cursor.read_u16()? as usize),
                20 => Ldc2W(cursor.read_u16()? as usize),
                109 => LDiv,
                22 => LLoad(cursor.read_u8()?),
                30 => LLoad0,
                31 => LLoad1,
                32 => LLoad2,
                33 => LLoad3,
                105 => LMul,
                117 => LNeg,
                171 => {
                    let pad_len = if cursor.position() % 4 == 0 {
                        0
                    } else {
                        4 - cursor.position() % 4
                    };
                    (0..pad_len)
                        .map(|_| cursor.read_u8())
                        .collect::<Result<Vec<u8>>>()?;
                    let default_index = cursor.read_i32()?;

                    let pairs = {
                        let pair_len = cursor.read_i32()?;
                        if pair_len < 0 {
                            bail!("Invalid number of pairs while parsing LookUpSwitch.");
                        }
                        let mut pairs = Vec::with_capacity(pair_len as usize);
                        for _ in 0..pair_len {
                            pairs.push((cursor.read_i32()?, cursor.read_i32()?));
                        }
                        pairs
                    };
                    LookUpSwitch {
                        default_index,
                        jump_table: pairs,
                    }
                }
                129 => LOr,
                113 => LRem,
                173 => LReturn,
                121 => LShL,
                123 => LShR,
                55 => LStore(cursor.read_u8()?),
                63 => LStore0,
                64 => LStore1,
                65 => LStore2,
                66 => LStore3,
                101 => LSub,
                125 => LuShR,
                131 => LxOr,
                194 => MonitorEnter,
                195 => MonitorExit,
                197 => MultiaNewArray(cursor.read_u16()?, cursor.read_u8()?),
                188 => NewArray(ArrayType::try_from(cursor.read_u8()?)?),
                187 => New(cursor.read_u16()?),
                0 => Nop,
                87 => Pop,
                88 => Pop2,
                181 => PutField(cursor.read_u16()?),
                179 => PutStatic(cursor.read_u16()?),
                169 => Ret(cursor.read_u8()?),
                177 => Return,
                53 => SaLoad,
                86 => SaStore,
                17 => SiPush(cursor.read_i16()?),
                95 => Swap,
                170 => {
                    let pad_len = if cursor.position() % 4 == 0 {
                        0
                    } else {
                        4 - cursor.position() % 4
                    };
                    (0..pad_len)
                        .map(|_| cursor.read_u8())
                        .collect::<Result<Vec<u8>>>()?;
                    let default_index = cursor.read_i32()?;
                    let low = cursor.read_i32()?;
                    let high = cursor.read_i32()?;
                    if low > high {
                        bail!("TableSwitch low > high invariant violated.");
                    }
                    let jump_offsets = (0..(high - low + 1))
                        .map(|_| cursor.read_i32())
                        .collect::<Result<Vec<i32>>>()?;

                    TableSwitch {
                        default_index,
                        low,
                        high,
                        jump_offsets,
                    }
                }
                196 => Wide(Self::parse_wide_opcode(&mut cursor)?),
                unknown => {
                    bail!("Unknown instruction {unknown}");
                }
            };
            instructions.push(instruction)
        }
        if !jmp_registry.is_empty() {
            Self::fix_jmps(jmp_registry, &byte_to_instr, &mut instructions);
        }
        instructions.shrink_to_fit();
        Ok(instructions)
    }

    fn parse_wide_opcode(cursor: &mut ClassReader) -> Result<WideInstruction> {
        let mut opcode = WideInstruction::try_from(Instruction::try_from(cursor.read_u8()?)?)?;
        match opcode {
            WideInstruction::Iinc(ref mut a, ref mut b) => {
                *a = cursor.read_u16()?;
                *b = cursor.read_u16()?;
            }
            WideInstruction::ILoad(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::FLoad(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::ALoad(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::LLoad(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::DLoad(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::IStore(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::FStore(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::AStore(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::LStore(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::DStore(ref mut a) => *a = cursor.read_u16()?,
            WideInstruction::Ret(ref mut a) => *a = cursor.read_u16()?,
            invalid => bail!("Invalid Opcode for Wide instruction: {invalid:?}."),
        };
        Ok(opcode)
    }

    fn fix_jmps(registry: Vec<(usize, i16)>, byte_to_instr: &[usize], instrs: &mut [Instruction]) {
        /*
            1. Get jmp_instr_index and jmp_byte from registry.
            2. Use jmp_instr_index to get the jmp instruction to be modified via instrs slice.
            3. mutate the operand in the retrieved jmp instruction to point to an instruction by
               converting the jmp_byte to an instruction via byte_to_instr slice.
        */
        for (jmp_instr_index, jmp_byte) in registry.into_iter() {
            let jmp_instr = instrs.get_mut(jmp_instr_index).unwrap();
            match jmp_instr {
                IfAcmpeq(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfAcmpne(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfIcmpeq(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfIcmpne(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfIcmplt(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfIcmpge(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfIcmpgt(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfIcmple(ind) => *ind = byte_to_instr[jmp_byte as usize],
                Ifeq(ind) => *ind = byte_to_instr[jmp_byte as usize],
                Ifne(ind) => *ind = byte_to_instr[jmp_byte as usize],
                Iflt(ind) => *ind = byte_to_instr[jmp_byte as usize],
                Ifge(ind) => *ind = byte_to_instr[jmp_byte as usize],
                Ifgt(ind) => *ind = byte_to_instr[jmp_byte as usize],
                Ifle(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfNonNull(ind) => *ind = byte_to_instr[jmp_byte as usize],
                IfNull(ind) => *ind = byte_to_instr[jmp_byte as usize],
                GoTo(ind) => *ind = byte_to_instr[jmp_byte as usize],
                GoToW(ind) => *ind = byte_to_instr[jmp_byte as usize],
                _ => unreachable!(),
            }
        }
    }
}
impl TryFrom<u8> for Instruction {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        let instruction = match value {
            50 => AaLoad,
            83 => AaStore,
            1 => AConstNull,
            25 => ALoad(0),
            42 => ALoad0,
            43 => ALoad1,
            44 => ALoad2,
            45 => ALoad3,
            189 => ANewArray(0),
            176 => AReturn,
            190 => ArrayLength,
            58 => AStore(0),
            75 => AStore0,
            76 => AStore1,
            77 => AStore2,
            78 => AStore3,
            191 => AThrow,
            51 => BaLoad,
            84 => BaStore,
            16 => BiPush(0),
            52 => CaLoad,
            85 => CaStore,
            192 => Checkcast(0),
            144 => D2F,
            142 => D2I,
            143 => D2L,
            99 => DAdd,
            49 => DaLoad,
            82 => DaStore,
            152 => Dcmpg,
            151 => Dcmpl,
            14 => DConst0,
            15 => DConst1,
            111 => DDiv,
            24 => DLoad(0),
            38 => DLoad0,
            39 => DLoad1,
            40 => DLoad2,
            41 => DLoad3,
            107 => DMul,
            119 => DNeg,
            115 => DRem,
            175 => DReturn,
            57 => DStore(0),
            71 => DStore0,
            72 => DStore1,
            73 => DStore2,
            74 => DStore3,
            103 => DSub,
            89 => Dup,
            90 => DupX1,
            91 => DupX2,
            92 => Dup2,
            93 => Dup2X1,
            94 => Dup2X2,
            141 => F2D,
            139 => F2I,
            140 => F2L,
            98 => FAdd,
            48 => FaLoad,
            81 => FaStore,
            150 => Fcmpg,
            149 => Fcmpl,
            11 => FConst0,
            12 => FConst0,
            13 => FConst0,
            110 => FDiv,
            23 => FLoad(0),
            34 => FLoad0,
            35 => FLoad1,
            36 => FLoad2,
            37 => FLoad3,
            106 => FMul,
            118 => FNeg,
            114 => FRem,
            174 => FReturn,
            56 => FStore(0),
            67 => FStore0,
            68 => FStore1,
            69 => FStore2,
            70 => FStore3,
            180 => GetField(0),
            178 => GetStatic(0),
            167 => GoTo(0),
            200 => GoToW(0),
            145 => I2B,
            146 => I2C,
            135 => I2D,
            133 => I2L,
            147 => I2S,
            96 => IAdd,
            46 => IaLoad,
            126 => IAnd,
            79 => IaStore,
            2 => IConstM1,
            3 => IConst0,
            4 => IConst1,
            5 => IConst2,
            6 => IConst3,
            7 => IConst4,
            8 => IConst5,
            108 => IDiv,
            165 => IfAcmpeq(0),
            166 => IfAcmpne(0),
            159 => IfIcmpeq(0),
            160 => IfIcmpne(0),
            161 => IfIcmplt(0),
            162 => IfIcmpge(0),
            163 => IfIcmpgt(0),
            164 => IfIcmple(0),
            153 => Ifeq(0),
            154 => Ifne(0),
            155 => Iflt(0),
            156 => Ifge(0),
            157 => Ifgt(0),
            158 => Ifle(0),
            199 => IfNonNull(0),
            198 => IfNull(0),
            132 => Iinc(0, 0),
            21 => ILoad(0),
            26 => ILoad0,
            27 => ILoad1,
            28 => ILoad2,
            29 => ILoad3,
            104 => IMul,
            116 => INeg,
            193 => InstanceOf(0),
            186 => InvokeDynamic(0, 0, 0),
            183 => InvokeSpecial(0),
            184 => InvokeStatic(0),
            182 => InvokeVirtual(0),
            128 => IOr,
            112 => IRem,
            172 => IReturn,
            120 => IShL,
            122 => IShR,
            54 => IStore(0),
            59 => IStore0,
            60 => IStore1,
            61 => IStore2,
            62 => IStore3,
            100 => ISub,
            124 => IuShR,
            130 => IxOr,
            168 => Jsr(0),
            201 => JsrW(0),
            138 => L2D,
            137 => L2F,
            136 => L2I,
            97 => LAdd,
            47 => LaLoad,
            127 => LAnd,
            80 => LaStore,
            148 => Lcmp,
            9 => LConst0,
            10 => LConst1,
            18 => Ldc(0),
            19 => LdcW(0),
            20 => Ldc2W(0),
            109 => LDiv,
            22 => LLoad(0),
            30 => LLoad0,
            31 => LLoad1,
            32 => LLoad2,
            33 => LLoad3,
            105 => LMul,
            117 => LNeg,
            171 => LookUpSwitch {
                default_index: 0,
                jump_table: vec![],
            },
            129 => LOr,
            113 => LRem,
            173 => LReturn,
            121 => LShL,
            123 => LShR,
            55 => LStore(0),
            63 => LStore0,
            64 => LStore1,
            65 => LStore2,
            66 => LStore3,
            101 => LSub,
            125 => LuShR,
            131 => LxOr,
            194 => MonitorEnter,
            195 => MonitorExit,
            197 => MultiaNewArray(0, 0),
            188 => NewArray(Dummy),
            0 => Nop,
            87 => Pop,
            88 => Pop2,
            181 => PutField(0),
            179 => PutStatic(0),
            169 => Ret(0),
            177 => Return,
            53 => SaLoad,
            86 => SaStore,
            17 => SiPush(0),
            95 => Swap,
            170 => TableSwitch {
                default_index: 0,
                low: 0,
                high: 0,
                jump_offsets: vec![],
            },
            196 => Wide(Invalid),
            unknown => {
                bail!("Unknown instruction {unknown}");
            }
        };
        Ok(instruction)
    }
}

impl TryFrom<Instruction> for WideInstruction {
    type Error = anyhow::Error;

    fn try_from(inst: Instruction) -> Result<Self> {
        let wcode = match inst {
            Iinc(_, _) => WideInstruction::Iinc(0, 0),
            ILoad(_) => WideInstruction::ILoad(0),
            FLoad(_) => WideInstruction::FLoad(0),
            ALoad(_) => WideInstruction::ALoad(0),
            LLoad(_) => WideInstruction::LLoad(0),
            DLoad(_) => WideInstruction::DLoad(0),
            IStore(_) => WideInstruction::IStore(0),
            FStore(_) => WideInstruction::FStore(0),
            AStore(_) => WideInstruction::AStore(0),
            LStore(_) => WideInstruction::LStore(0),
            DStore(_) => WideInstruction::DStore(0),
            Ret(_) => WideInstruction::Ret(0),
            invalid => bail!("Invalid Opcode for Wide instruction: {invalid:?}."),
        };
        Ok(wcode)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ArrayType {
    Boolean,
    Char,
    Float,
    Double,
    Byte,
    Short,
    Int,
    Long,
    Dummy,
    // not in spec but used for array construction in oop.rs
    Ref,
}

impl TryFrom<u8> for ArrayType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        let array_type = match value {
            4 => Boolean,
            5 => Char,
            6 => Float,
            7 => Double,
            8 => Byte,
            9 => Short,
            10 => Int,
            11 => Long,
            invalid => {
                bail!("Invalid ArrayType tag: {invalid}");
            }
        };
        Ok(array_type)
    }
}
