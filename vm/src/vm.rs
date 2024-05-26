use std::path::PathBuf;

use anyhow::{bail, Result};

use sumatra_parser::{constant::Constant, instruction::Instruction, method::Method};

use crate::{
    call_frame::CallFrame,
    class::Class,
    compare::Compare,
    lli::{class_manager::ClassManager, response::Response},
    method_area::MethodArea,
    static_data::StaticData,
    value::Value,
};

const MAIN: &str = "main([Ljava/lang/String;)V";
const CLINIT: &str = "<clinit>()V";
const INIT: &str = "<init>()V";

const DEFAULT_VEC_SIZE: usize = 128;

pub struct VM {
    pub(crate) frames: Vec<CallFrame>,
    pub(crate) method_area: MethodArea,
    pub(crate) class_manager: ClassManager,
}

impl VM {
    /// create the VM.
    pub fn init(jdk: PathBuf, c_path: PathBuf) -> Self {
        //TODO find good allocation size for vectors
        let method_area = match MethodArea::new() {
            Ok(method_area) => method_area,
            Err(_) => panic!("Memory Allocation Error while starting Sumatra VM"),
        };

        Self {
            frames: Vec::with_capacity(DEFAULT_VEC_SIZE),
            method_area,
            class_manager: ClassManager::new(jdk, c_path),
        }
    }

    /// Entry point of the JVM. `c_entry` is the initial class loaded
    /// by the VM to spin up the Java program.
    pub fn run(&mut self, c_entry: &str) -> Result<()> {
        let c_data = if !c_entry.ends_with(".class") {
            self.load_class(&format!("{c_entry}{}", ".class"))?
        } else {
            self.load_class(c_entry)?
        };
        let frame = self.construct_main(c_data)?;
        self.frames.push(frame);
        self.execute_frame()?;
        Ok(())
    }

    /// Executes the top most `CallFrame` on the stack. Frame
    /// gets popped off the stack before method returns.
    fn execute_frame(&mut self) -> Result<Option<Value>> {
        let code = &self.frame().method.code;
        let op_code = &code.op_code;
        println!("\nExecuting method: {}", self.frame().method.name);
        while let Some(code) = op_code.get(self.frame().pc) {
            // if self.frame().method.name != "<clinit>" {
            println!("\t{code:?}");
            // }
            match code {
                Instruction::AaLoad => todo!(),
                Instruction::AaStore => todo!(),
                Instruction::AaConstNull => self.frame_mut().stack.push(Value::Null),
                Instruction::ALoad(index) => self.a_load(*index as usize)?,
                Instruction::ALoad0 => self.a_load(0)?,
                Instruction::ALoad1 => self.a_load(1)?,
                Instruction::ALoad2 => self.a_load(2)?,
                Instruction::ALoad3 => self.a_load(3)?,
                Instruction::ANewArray(_) => todo!(),
                Instruction::AReturn => todo!(),
                Instruction::ArrayLength => todo!(),
                Instruction::AStore(_) => todo!(),
                Instruction::AStore0 => self.a_store_n(0)?,
                Instruction::AStore1 => self.a_store_n(1)?,
                Instruction::AStore2 => self.a_store_n(2)?,
                Instruction::AStore3 => self.a_store_n(3)?,
                Instruction::AThrow => todo!(),
                Instruction::BaLoad => todo!(),
                Instruction::BaStore => todo!(),
                Instruction::BiPush(byte) => self.frame_mut().stack.push(Value::Int(*byte as i32)),
                Instruction::CaLoad => todo!(),
                Instruction::CaStore => todo!(),
                Instruction::Checkcast(_) => todo!(),
                Instruction::D2F => todo!(),
                Instruction::D2I => todo!(),
                Instruction::D2L => todo!(),
                Instruction::DAdd => self.dadd()?,
                Instruction::DaLoad => todo!(),
                Instruction::DaStore => todo!(),
                Instruction::Dcmpg => todo!(),
                Instruction::Dcmpl => todo!(),
                Instruction::DConst0 => todo!(),
                Instruction::DConst1 => todo!(),
                Instruction::DDiv => todo!(),
                Instruction::DLoad(local_index) => self.dload_n(*local_index as usize)?,
                Instruction::DLoad0 => self.dload_n(0)?,
                Instruction::DLoad1 => self.dload_n(1)?,
                Instruction::DLoad2 => self.dload_n(2)?,
                Instruction::DLoad3 => self.dload_n(3)?,
                Instruction::DMul => todo!(),
                Instruction::DNeg => todo!(),
                Instruction::DRem => todo!(),
                Instruction::DReturn => return Ok(Some(self.return_val())),
                Instruction::DStore(local_index) => self.dstore_n(*local_index as usize)?,
                Instruction::DStore0 => self.dstore_n(0)?,
                Instruction::DStore1 => self.dstore_n(1)?,
                Instruction::DStore2 => self.dstore_n(2)?,
                Instruction::DStore3 => self.dstore_n(3)?,
                Instruction::DSub => todo!(),
                Instruction::Dup => self.dup()?,
                Instruction::DupX1 => todo!(),
                Instruction::DupX2 => todo!(),
                Instruction::Dup2 => todo!(),
                Instruction::Dup2X1 => todo!(),
                Instruction::Dup2X2 => todo!(),
                Instruction::F2D => todo!(),
                Instruction::F2I => todo!(),
                Instruction::F2L => todo!(),
                Instruction::FAdd => todo!(),
                Instruction::FaLoad => todo!(),
                Instruction::FaStore => todo!(),
                Instruction::Fcmpg => todo!(),
                Instruction::Fcmpl => todo!(),
                Instruction::FConst0 => todo!(),
                Instruction::FConst1 => todo!(),
                Instruction::FConst2 => todo!(),
                Instruction::FDiv => todo!(),
                Instruction::FLoad(_) => todo!(),
                Instruction::FLoad0 => todo!(),
                Instruction::FLoad1 => todo!(),
                Instruction::FLoad2 => todo!(),
                Instruction::FLoad3 => todo!(),
                Instruction::FMul => todo!(),
                Instruction::FNeg => todo!(),
                Instruction::FRem => todo!(),
                Instruction::FReturn => return Ok(Some(self.return_val())),
                Instruction::FStore(_) => todo!(),
                Instruction::FStore0 => todo!(),
                Instruction::FStore1 => todo!(),
                Instruction::FStore2 => todo!(),
                Instruction::FStore3 => todo!(),
                Instruction::FSub => todo!(),
                Instruction::GetField(_) => todo!(),
                Instruction::GetStatic(index) => self.get_static(*index)?,
                Instruction::GoTo(instr) => {
                    self.frame_mut().pc = *instr;
                    continue;
                }
                Instruction::GoToW(_) => todo!(),
                Instruction::I2B => todo!(),
                Instruction::I2C => todo!(),
                Instruction::I2D => todo!(),
                Instruction::I2F => todo!(),
                Instruction::I2L => todo!(),
                Instruction::I2S => todo!(),
                Instruction::IAdd => todo!(),
                Instruction::IaLoad => todo!(),
                Instruction::IAnd => todo!(),
                Instruction::IaStore => todo!(),
                Instruction::IConstM1 => self.iconst_n(-1),
                Instruction::IConst0 => self.iconst_n(0),
                Instruction::IConst1 => self.iconst_n(1),
                Instruction::IConst2 => self.iconst_n(2),
                Instruction::IConst3 => self.iconst_n(3),
                Instruction::IConst4 => self.iconst_n(4),
                Instruction::IConst5 => self.iconst_n(5),
                Instruction::IDiv => todo!(),
                Instruction::IfAcmpeq(index) => {
                    if self.ifcmp(*index, Compare::Equal) {
                        continue;
                    }
                }
                Instruction::IfAcmpne(index) => {
                    if self.ifcmp(*index, Compare::NotEqual) {
                        continue;
                    }
                }
                Instruction::IfIcmpeq(index) => {
                    if self.ifcmp(*index, Compare::Equal) {
                        continue;
                    }
                }
                Instruction::IfIcmpne(index) => {
                    if self.ifcmp(*index, Compare::NotEqual) {
                        continue;
                    }
                }
                Instruction::IfIcmplt(index) => {
                    if self.ifcmp(*index, Compare::LessThan) {
                        continue;
                    }
                }
                Instruction::IfIcmpge(index) => {
                    if self.ifcmp(*index, Compare::GreaterOrEqual) {
                        continue;
                    }
                }
                Instruction::IfIcmpgt(index) => {
                    if self.ifcmp(*index, Compare::GreaterThan) {
                        continue;
                    }
                }
                Instruction::IfIcmple(index) => {
                    if self.ifcmp(*index, Compare::LessOrEqual) {
                        continue;
                    }
                }
                Instruction::Ifeq(index) => {
                    if self.if_cond(*index, Compare::Equal) {
                        continue;
                    }
                }
                Instruction::Ifne(index) => {
                    if self.if_cond(*index, Compare::NotEqual) {
                        continue;
                    }
                }
                Instruction::Iflt(index) => {
                    if self.if_cond(*index, Compare::LessThan) {
                        continue;
                    }
                }
                Instruction::Ifge(index) => {
                    if self.if_cond(*index, Compare::GreaterOrEqual) {
                        continue;
                    }
                }
                Instruction::Ifgt(index) => {
                    if self.if_cond(*index, Compare::GreaterThan) {
                        continue;
                    }
                }
                Instruction::Ifle(index) => {
                    if self.if_cond(*index, Compare::LessOrEqual) {
                        continue;
                    }
                }
                Instruction::IfNonNull(_) => todo!(),
                Instruction::IfNull(_) => todo!(),
                Instruction::Iinc(index, inc) => self.iinc(*index as usize, *inc as i32),
                Instruction::ILoad(local_index) => self.iload_n(*local_index as usize)?,
                Instruction::ILoad0 => self.iload_n(0)?,
                Instruction::ILoad1 => self.iload_n(1)?,
                Instruction::ILoad2 => self.iload_n(2)?,
                Instruction::ILoad3 => self.iload_n(3)?,
                Instruction::IMul => todo!(),
                Instruction::INeg => todo!(),
                Instruction::InstanceOf(_) => todo!(),
                Instruction::InvokeDynamic(index, _, _) => println!("\t    InvokeDynamic: {index}"),
                Instruction::InvokeInterface(_, _, _) => todo!(),
                Instruction::InvokeSpecial(method_index) => {
                    if let Some(value) = self.invoke_special(*method_index as usize)? {
                        if let Value::Double(_) | Value::Long(_) = value {
                            self.frame_mut().stack.push(value.clone());
                        }
                        self.frame_mut().stack.push(value);
                    }
                }
                Instruction::InvokeStatic(method_index) => {
                    if let Some(value) = self.invoke_static(*method_index as usize)? {
                        if let Value::Double(_) | Value::Long(_) = value {
                            self.frame_mut().stack.push(value.clone());
                        }
                        self.frame_mut().stack.push(value);
                    }
                }
                Instruction::InvokeVirtual(method_index) => {
                    if let Some(value) = self.invoke_virtual(method_index)? {
                        if let Value::Double(_) | Value::Long(_) = value {
                            self.frame_mut().stack.push(value.clone());
                        }
                        self.frame_mut().stack.push(value);
                    }
                }
                Instruction::IOr => todo!(),
                Instruction::IRem => self.irem()?,
                Instruction::IReturn => return Ok(Some(self.return_val())),
                Instruction::IShL => todo!(),
                Instruction::IShR => todo!(),
                Instruction::IStore(local_index) => self.istore_n(*local_index as usize)?,
                Instruction::IStore0 => self.istore_n(0)?,
                Instruction::IStore1 => self.istore_n(1)?,
                Instruction::IStore2 => self.istore_n(2)?,
                Instruction::IStore3 => self.istore_n(3)?,
                Instruction::ISub => todo!(),
                Instruction::IuShR => todo!(),
                Instruction::IxOr => todo!(),
                Instruction::Jsr(_) => todo!(),
                Instruction::JsrW(_) => todo!(),
                Instruction::L2D => todo!(),
                Instruction::L2F => todo!(),
                Instruction::L2I => todo!(),
                Instruction::LAdd => todo!(),
                Instruction::LaLoad => todo!(),
                Instruction::LAnd => todo!(),
                Instruction::LaStore => todo!(),
                Instruction::Lcmp => todo!(),
                Instruction::LConst0 => todo!(),
                Instruction::LConst1 => todo!(),
                Instruction::Ldc(index) => self.load_const(index)?,
                Instruction::LdcW(index) => self.load_const(index)?,
                Instruction::Ldc2W(index) => self.load_const2(index)?,
                Instruction::LDiv => todo!(),
                Instruction::LLoad(_) => todo!(),
                Instruction::LLoad0 => todo!(),
                Instruction::LLoad1 => todo!(),
                Instruction::LLoad2 => todo!(),
                Instruction::LLoad3 => todo!(),
                Instruction::LMul => todo!(),
                Instruction::LNeg => todo!(),
                Instruction::LookUpSwitch { .. } => todo!(),
                Instruction::LOr => todo!(),
                Instruction::LRem => todo!(),
                Instruction::LReturn => return Ok(Some(self.return_val())),
                Instruction::LShL => todo!(),
                Instruction::LShR => todo!(),
                Instruction::LStore(_) => todo!(),
                Instruction::LStore0 => todo!(),
                Instruction::LStore1 => todo!(),
                Instruction::LStore2 => todo!(),
                Instruction::LStore3 => todo!(),
                Instruction::LSub => todo!(),
                Instruction::LuShR => todo!(),
                Instruction::LxOr => todo!(),
                Instruction::MonitorEnter => todo!(),
                Instruction::MonitorExit => todo!(),
                Instruction::MultiaNewArray(_, _) => todo!(),
                Instruction::New(class_index) => self.new_obj(*class_index as usize)?,
                Instruction::NewArray(_) => todo!(),
                Instruction::Nop => todo!(),
                Instruction::Pop => self.pop(),
                Instruction::Pop2 => self.pop2(),
                Instruction::PutField(_) => todo!(),
                Instruction::PutStatic(field_index) => self.put_static(*field_index as usize)?,
                Instruction::Ret(_) => todo!(),
                Instruction::Return => break,
                Instruction::SaLoad => todo!(),
                Instruction::SaStore => todo!(),
                Instruction::SiPush(_) => todo!(),
                Instruction::Swap => todo!(),
                Instruction::TableSwitch { .. } => todo!(),
                Instruction::Wide(winstr) => todo!(),
            }
            // println!("\tSTACK: {:?}", self.frame().stack);
            self.frame_mut().pc += 1;
        }
        println!("Exiting method: {}", self.frame().method.name);
        self.frames.pop();
        Ok(None)
    }

    /// Executes the `Instruction::AStore` instruction.`local_index`  
    /// is the index of the local variable in the currently
    /// executing frame's local variable array.
    /// https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.astore_n
    fn a_store_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let operand = frame.stack.pop().unwrap();
        match operand {
            value @ (Value::ReturnAddress(_) | Value::Ref(_) | Value::StringConst(_)) => {
                *frame.locals.get_mut(local_index).unwrap() = value;
            }
            _ => panic!(
                "Expected a Reference type or Value::ReturnAddress for the operand \
                    of instruction a_store_n.: {operand:?}"
            ),
        };
        Ok(())
    }

    /// Executes the `Instruction::ALoad` instruction. `local_index`  
    /// is the index of the local variable in the currently
    /// executing frame's local variable array.
    fn a_load(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        println!("Stack: {:?}", frame.locals);
        let object = frame.load(local_index)?;
        if !matches!(
            object,
            (Value::Ref(_) | Value::StringConst(_) | Value::Null)
        ) {
            bail!("Expected ref type for a_load instruction.");
        }

        Ok(frame.push(object))
    }

    /// Executes the `Instruction::DAdd` instruction.
    fn dadd(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value2 = frame.pop();
        let value1 = frame.pop();
        let sum = match (value2, value1) {
            (Value::Double(double2), Value::Double(double1)) => double2 + double1,
            _ => bail!("Expected 2 doubles for dadd instruction."),
        };
        frame.push(Value::Double(sum));
        Ok(frame.push(Value::Double(sum)))
    }

    /// Executes the `Instruction::DLoad<local_index>` instruction.
    /// `local_index` is the index of the local variable in the currently
    /// executing frame's local variable array.
    fn dload_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let double = frame.load(local_index)?;
        if !matches!(double, Value::Double(_)) {
            bail!("Expected ref type for d_load instruction.");
        }
        frame.push(double.clone());
        Ok(frame.push(double))
    }

    /// Executes the `Instruction::DStore<local_index>` instruction.
    /// `local_index` is the index of the local variable in the currently
    /// executing frame's local variable array.
    fn dstore_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let double = frame.pop();
        if !matches!(double, Value::Double(_)) {
            bail!("Expected a double for dstore instruction.");
        }

        *frame.locals.get_mut(local_index + 1).unwrap() = double.clone();
        Ok(*frame.locals.get_mut(local_index).unwrap() = double)
    }

    /// Executes the `Instruction::Dup` instruction.
    fn dup(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value = frame.clone_top();
        Ok(frame.push(value))
    }

    /// Executes the `Instruction::IConst` instruction. `int` is the integer
    /// to be pushed on the operand stack.
    fn iconst_n(&mut self, int: i32) { self.frame_mut().push(Value::Int(int)); }

    /// Exectutes the `Instruction::Iinc` instruction. `index` is the index of
    /// the local variable to incremented by `inc`.
    fn iinc(&mut self, index: usize, inc: i32) {
        let frame = self.frame_mut();
        if let Value::Int(num) = frame.get_local(index) {
            *num += inc;
        } else {
            panic!("Int was expected for instruction iinc.");
        }
    }

    /// Executes one of the many compare JVM instructions.
    /// The type of comparison is specified by the `compare::Compare` argument.
    /// Returns a `bool` representing the result of the comparison.
    /// Program counter is already updated if necessary.
    fn ifcmp(&mut self, offset: usize, cmp: Compare) -> bool {
        let frame = self.frame_mut();
        let value2 = frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();
        let jmp = Self::if_in(value1, value2, cmp);
        if jmp {
            self.frame_mut().pc = offset;
        }
        jmp
    }

    /// Helper function for `VM::ifcmp`.
    fn if_in(value1: Value, value2: Value, cmp: Compare) -> bool {
        match cmp {
            Compare::Equal => value1 == value2,
            Compare::NotEqual => value1 != value2,
            Compare::LessThan => value1 < value2,
            Compare::GreaterOrEqual => value1 >= value2,
            Compare::GreaterThan => value1 > value2,
            Compare::LessOrEqual => value1 <= value2,
        }
    }

    /// Similar to `VM::ifcmp` but only operates on Java ints and compares to 0.
    fn if_cond(&mut self, offset: usize, cmp: Compare) -> bool {
        let frame = self.frame_mut();
        let int = frame.pop();
        if !matches!(int, Value::Int(_)) {
            panic!("Expected int for if_cond instruction.");
        }
        let jmp = Self::if_in(int, Value::Int(0), cmp);
        if jmp {
            frame.pc = offset;
        }
        jmp
    }

    /// Executed the `Instruction::InvokeSpecial` instruction. `method_index` is
    /// the index to the `Constant::MethodRef` or `Constant::InterfaceMethodRef`
    /// in the runtime constant pool.
    fn invoke_special(&mut self, method_index: usize) -> Result<Option<Value>> {
        // This method is very involved and convoluted. The algorithm
        // for determining the target method of an `Instruction::InvokeSpecial` is
        // defined here: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.invokespecial
        match self.frame().cp.get(method_index).unwrap() {
            Constant::MethodRef {
                class_index,
                name_and_type_index,
            } => {
                let is_super = self.superclass_method(*class_index, *name_and_type_index)?;
                let mut class = if is_super {
                    // if is_super use check for method in the superclass
                    let super_class_index = self.frame().class.super_class;
                    let Constant::Class(class_name_index) =
                        self.frame().cp.get(super_class_index).unwrap()
                    else {
                        bail!("Expected class while executing invokespecial.");
                    };

                    let class_name = self.frame().cp.get_utf8(*class_name_index)?;
                    self.load_class(class_name)?.class
                } else {
                    // if not is_super use class named in the symbolic reference
                    let Constant::Class(class_name_index) =
                        self.frame().cp.get(*class_index).unwrap()
                    else {
                        bail!("Expected class while executing invokespecial.")
                    };
                    let class_name = self.frame().cp.get_utf8(*class_name_index)?;
                    self.load_class(class_name)?.class
                };

                let Constant::NameAndType {
                    name_index,
                    descriptor_index,
                } = self.frame().cp.get(*name_and_type_index).unwrap()
                else {
                    bail!("Expected NameAndType while executing invokespecial.");
                };
                let method_name = self.construct_m_name(*name_index, *descriptor_index)?;
                loop {
                    println!("Looking for {} in class {}", method_name, class.get_name());
                    match class.methods.get(&method_name) {
                        None => {
                            let super_index = class.super_class;
                            if super_index == 0 {
                                bail!("Index of superclass should not be 0 in invoke_special");
                            }
                            let Constant::Class(super_class) = class.cp.get(super_index).unwrap()
                            else {
                                bail!("Expected class while executing invokespecial.");
                            };
                            let super_name = class.cp.get_utf8(*super_class).unwrap();
                            class = self.load_class(super_name).unwrap().class;
                        }
                        Some(method) => {
                            println!("method found in {}", class.get_name());
                            if method.is_static() {
                                // TODO check for method in superclass of this class.
                                continue;
                            }
                            return self.invoke(class, method);
                        }
                    }
                }
            }
            Constant::InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => {
                let is_super = self.superclass_method(*class_index, *name_and_type_index)?;
                self.invokespec_intrfcref(*class_index, *name_and_type_index)
            }
            _ => bail!(
                "Expected symbolic reference to a method or interface method in invoke_special"
            ),
        }
    }

    /// Executed the `Instruction::InvokeStatic` instruction. `method_index` is
    /// the index to the `Constant::MethodRef` in the runtime constant pool.
    fn invoke_static(&mut self, method_index: usize) -> Result<Option<Value>> {
        let frame = self.frame();
        let Constant::MethodRef {
            class_index,
            name_and_type_index,
        } = frame.cp.get(method_index).unwrap()
        else {
            bail!("Expected Constant::MethodRef in invoke_static.");
        };

        let (name_index, desc_index, alloc) = self.unpack(*class_index, *name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &alloc)?;

        assert!(method.is_static());
        // TODO implement native method calls.
        if method.is_native() {
            println!("Method '{}' was a native method. Ignoring.", method.name);
            Ok(None)
        } else {
            self.invoke(class, method)
        }
    }

    /// Executed the `Instruction::InvokeVirtual` instruction. `method_index` is
    /// the index to the `Constant::MethodRef` in the runtime constant pool.
    fn invoke_virtual(&mut self, method_index: &usize) -> Result<Option<Value>> {
        let frame = self.frame();
        let Constant::MethodRef {
            class_index,
            name_and_type_index,
        } = frame.cp.get(*method_index).unwrap()
        else {
            bail!("Expected Constant::MethodRef in invoke_static.");
        };

        let (name_index, desc_index, alloc) = self.unpack(*class_index, *name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &alloc)?;
        assert!(!method.is_static());
        // TODO implement native method calls.
        if method.is_native() {
            println!("Method '{}' was a native method. Ignoring.", method.name);
            Ok(None)
        } else {
            self.invoke(class, method)
        }
    }

    /// Executes the `Instruction::ILoad` instruction.
    /// `local_index` is the index to the local variable in the
    /// locals array.
    fn iload_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();

        let int = frame.load(local_index)?;
        Ok(frame.push(int))
    }

    /// Executes `Instruction::IStore` instruction. `local_index`
    /// is the index of the local variable in the locals array.
    fn istore_n(&mut self, local_index: usize) -> Result<()> {
        let frame = self.frame_mut();
        let int = frame.pop();
        if !matches!(int, Value::Int(_)) {
            bail!("Expected a int for istore instruction.");
        }
        Ok(*frame.locals.get_mut(local_index).unwrap() = int)
    }

    /// Executes `Instruction::IRem` instruction and pushes
    /// result to the operand stack.
    fn irem(&mut self) -> Result<()> {
        let frame = self.frame_mut();
        let value2 = frame.pop();
        let value1 = frame.pop();
        if let Value::Int(int1) = value1 {
            if let Value::Int(int2) = value2 {
                if int2 == 0 {
                    bail!("Cannot use 0 in modular arithmetic.");
                }
                self.frame_mut().push(Value::Int(int1 % int2));
                return Ok(());
            }
        }
        bail!("expected 2 integers for irem instruction.");
    }

    /// Executes the `Instruction::Ldc`, and `Instruction::LdcW` instructions.
    /// `index` is the index of the constnat in the runtime constant pool.
    fn load_const(&mut self, index: &usize) -> Result<()> {
        let frame = self.frame_mut();
        let cp = frame.cp;
        let constant = cp.get(*index).unwrap();
        let value = match constant {
            // Only these Constants are considered to be loadable:
            //https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-4.html#jvms-4.4-310
            // Java longs or doubles are loaded by the lcd2 and lcd2_w instruction.
            // https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.ldc
            Constant::Integer(int) => Value::Int(*int),
            Constant::Float(f) => Value::Float(*f),
            Constant::Class(class_index) => Value::Class(*class_index),
            Constant::String(string_index) => {
                Value::StringConst(cp.get_utf8(*string_index)?.into())
            }
            Constant::MethodHandle {
                reference_kind,
                reference_index,
            } => Value::MethodHandle {
                reference_kind: *reference_kind,
                reference_index: *reference_index,
            },
            Constant::MethodType(type_index) => Value::MethodType(*type_index),
            Constant::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => Value::Dynamic {
                bootstrap_method_attr_index: *bootstrap_method_attr_index,
                name_and_type_index: *name_and_type_index,
            },
            _ => panic!("Non loadable constant pointed to by instruction ldc."),
        };
        frame.stack.push(value);
        Ok(())
    }

    /// Executes the `Instruction::Ldc2W` instruction.
    /// `index` is the index of the Java long or Java double
    /// in the runtime constant pool.
    fn load_const2(&mut self, index: &usize) -> Result<()> {
        let frame = self.frame_mut();
        let cp = frame.cp;
        let constant = cp.get(*index).unwrap();
        let value = match constant {
            Constant::Long(l) => Value::Long(*l),
            Constant::Double(d) => Value::Double(*d),
            _ => panic!("Non long or double constant pointed to by instruction ldc2w."),
        };
        frame.stack.push(value.clone());
        Ok(frame.stack.push(value))
    }

    /// Executes the `Instruction::New` instruction. `class_index` is an index
    /// to the runtime constant pool whose entry is a symbolic reference to
    /// a class or interface. New Java object is pushed on to the operand stack.
    fn new_obj(&mut self, class_index: usize) -> Result<()> {
        let frame = self.frame();
        let Constant::Class(name_index) = frame.cp.get(class_index).unwrap() else {
            bail!("Expected a symbolic class reference at index: {class_index}")
        };

        let class_name = frame.cp.get_utf8(*name_index)?;
        let StaticData {
            class, class_id, ..
        } = self.load_class(class_name)?;
        Ok(self.frame_mut().push(Value::new_object(class, class_id)))
    }

    /// Executes the `Instruction::GetStatic` instruction.
    /// `index` is the index of the `Constant::FieldRef` in the
    /// runtime constant pool.
    fn get_static(&mut self, index: usize) -> Result<()> {
        let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frame().cp.get(index).unwrap()
        else {
            bail!("Expected symbolic reference to a field at index: {index}");
        };

        let (name_index, _, alloc) = self.unpack(*class_index, *name_and_type_index)?;
        let field_val = alloc.get_field(self.frame().cp.get_utf8(name_index)?)?;
        if let Value::Long(_) | Value::Double(_) = field_val {
            self.frame_mut().stack.push(field_val.clone());
        }
        self.frame_mut().stack.push(field_val.clone());
        Ok(())
    }

    /// Executes the `Instruction::Pop` instruction. Nothing is returned.
    fn pop(&mut self) { self.frame_mut().pop(); }

    /// Executes the `Instruction::Pop2` instruction. Nothing is returned.
    fn pop2(&mut self) {
        let frame = self.frame_mut();
        let value = frame.pop();
        if let Value::Double(_) | Value::Long(_) = value {
            frame.pop();
        }
    }

    /// Executes the `Instruction::PutStatic` instruction.
    /// `field_index` is the index of the `Constant::FieldRef` in the
    /// runtime constant pool.
    fn put_static(&mut self, field_index: usize) -> Result<()> {
        let Constant::FieldRef {
            class_index,
            name_and_type_index,
        } = self.frame().cp.get(field_index).unwrap()
        else {
            bail!("Expected Constant::FieldRef for a put_static instruction.");
        };
        let (f_name, mut data) = self.unpack_f_name(class_index, name_and_type_index)?;
        data.set_field(&f_name, self.frame_mut().stack.pop().unwrap())?;
        Ok(())
    }

    fn return_val(&mut self) -> Value {
        println!("Exiting {}", self.frame().method.name);
        let value = self.frame_mut().pop();
        self.frames.pop();
        value
    }
}

// Utility functions are seperated into a different impl block for ease of
// navigation.
impl VM {
    /// Construct the local variables array and return it. Assumes there is a
    /// call frame on the stack. If constructing the locals for `main` use
    /// `construct_main_locals`
    fn construct_locals(&self, max_locals: usize, num_params: usize) -> Result<Vec<Value>> {
        if num_params > max_locals {
            bail!("number of method parameters was larger than the max locals.");
        }
        let stack_size = self.frame().stack.len();

        Ok(match (num_params, max_locals) {
            (0, 0) => vec![],
            (0, _) => Value::default_vec(max_locals),
            _ => {
                let mut locals =
                    Vec::from(&self.frame().stack[stack_size - num_params..stack_size]);
                Value::populate_locals(max_locals, &mut locals);
                locals
            }
        })
    }

    /// construct the CallFrame for the main method.
    fn construct_main(&self, c_data: StaticData) -> Result<CallFrame> {
        let main = c_data.class;
        let m_method = find_main(main)?;
        let cp = &main.cp;
        let locals = self.construct_main_locals(&m_method);
        let num_locals = locals.len();
        //TODO implement arguments to pass into main function
        Ok(CallFrame::new(main, m_method, cp, locals))
    }

    /// construct local variables for main method's `CallFrame`.
    fn construct_main_locals(&self, m_method: &Method) -> Vec<Value> {
        Value::default_vec(m_method.code.max_locals as usize)
    }

    /// Construct a method name from the index to the name, and the index to the
    /// descriptor.
    #[inline]
    fn construct_m_name(&self, name_index: usize, descr_index: usize) -> Result<String> {
        let cp = self.frame().cp;
        let name = cp.get_utf8(name_index)?;
        let descr = cp.get_utf8(descr_index)?;
        Ok(format!("{name}{descr}"))
    }

    /// Return a mutable reference to the top most call frame.
    #[inline(always)]
    fn frame_mut(&mut self) -> &mut CallFrame { self.frames.last_mut().unwrap() }

    /// Return a shared reference to the top most call frame.
    #[inline(always)]
    fn frame(&self) -> &CallFrame { self.frames.last().unwrap() }

    /// Take a static reference to a class and push its '<clinit>'
    /// method as a stack frame to `vm.frames`.
    fn init_class(&mut self, class: &'static Class) -> Result<Option<Value>> {
        // A `<clinit>` is not required by the spec.
        let clinit = match class.methods.get(CLINIT) {
            None => return Ok(None),
            Some(clint) => clint,
        };
        // clinit always takes 0 arguments
        let frame = CallFrame::new(class, clinit, &class.cp, vec![]);
        self.frames.push(frame);
        self.execute_frame()
    }

    /// Helper function to construct a `CallFrame` and invoke a method.
    fn invoke(&mut self, class: &'static Class, method: &'static Method) -> Result<Option<Value>> {
        let num_params = method.parsed_descriptor.num_params();
        let max_locals = method.code.max_locals as usize;

        let frame = CallFrame::new(
            class,
            method,
            &class.cp,
            self.construct_locals(max_locals, num_params)?,
        );
        self.frames.push(frame);
        self.execute_frame()
    }

    fn invokespec_metref(
        &mut self,
        class_index: usize,
        name_and_type_index: usize,
    ) -> Result<Option<Value>> {
        let (name_index, desc_index, alloc) = self.unpack(class_index, name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &alloc)?;
        todo!()
    }

    fn invokespec_intrfcref(
        &mut self,
        class_index: usize,
        name_and_type_index: usize,
    ) -> Result<Option<Value>> {
        todo!()
    }

    fn invokespec_resolve_class(
        &mut self,
        class_index: usize,
        name_and_type_index: usize,
    ) -> Result<&'static Class> {
        todo!()
    }

    /// Helper function to determine if the target of a
    /// `Instruction::InvokeSpecial` instruction is defined in the
    /// superclass of the current class.
    fn superclass_method(
        &mut self,
        class_index: usize,
        name_and_type_index: usize,
    ) -> Result<bool> {
        //TODO add error handling laid out here: https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-6.html#jvms-6.5.invokespecial
        // There are RuntimeExceptions to be returned if the class method is static,
        // ect.
        let frame = self.frame();
        let super_index = frame.class.super_class;

        // check if the class named by the method symbolically referenced is the
        // superclass of the current class.
        if !(class_index == super_index) {
            return Ok(false);
        }

        // from this point on the class named by the method is the superclass of the
        // current class. There are other factors to check before returning
        // true.
        if !frame.class.is_super() {
            return Ok(false);
        }

        let (name_index, desc_index, static_data) =
            self.unpack(class_index, name_and_type_index)?;
        let (class, method) = self.to_method_class(name_index, desc_index, &static_data)?;
        if class.is_interface() {
            return Ok(false);
        }

        if method.is_static() {
            //TODO replace with proper handling as indicated above.
            panic!("invokespecial target method was static!");
        }

        if method.name == "<init>".to_string() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Load the class definition specified by `name`. If
    /// the class is found in the `MethodArea`, a `StaticData` object
    /// is returned. This function handles initialization if necessary.
    fn load_class(&mut self, name: &str) -> Result<StaticData> {
        match self.class_manager.request(name, &mut self.method_area) {
            Ok(Response::InitReq(class, alloc_index)) => {
                self.init_class(class)?;
                self.method_area.class_data(alloc_index)
            }
            Ok(Response::Ready(alloc_index)) => self.method_area.class_data(alloc_index),
            Err(e) => bail!(e),
            _ => panic!("Manager returned a not found!"),
        }
    }

    /// Retrieve a static reference to a method and it's class. `name_index` is
    /// the name of the method, `desc_index` is the descriptor of the
    /// method. `static_data` is a reference to a `StaticData` object
    /// holding the class data.
    fn to_method_class(
        &self,
        name_index: usize,
        desc_index: usize,
        static_data: &StaticData,
    ) -> Result<(&'static Class, &'static Method)> {
        let class = static_data.class;
        let met_name = self.construct_m_name(name_index, desc_index)?;
        let method = class.methods.get(&met_name).unwrap();
        Ok((class, method))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns the `name_index`,
    /// `descriptor_index`,  and a `StaticData` of the class pointed
    /// to by `class_name`. The returned StaticData will have a fully
    /// initialize Class.
    fn unpack(
        &mut self,
        class_index: usize,
        name_and_type: usize,
    ) -> Result<(usize, usize, StaticData)> {
        let frame = self.frame();
        let Constant::Class(class_name) = frame.cp.get(class_index).unwrap() else {
            bail!(
                "Class index while executing `get_static` \
                    didn't point to a Class in the constant pool."
            );
        };
        let name = frame.cp.get_utf8(*class_name)?;
        let static_data = self.load_class(name)?;

        let Constant::NameAndType {
            name_index,
            descriptor_index,
        } = self.frame().cp.get(name_and_type).unwrap()
        else {
            bail!(
                "Provided name_and_type_index did not point to a \
                NameAndType constant."
            );
        };
        Ok((*name_index, *descriptor_index, static_data))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns a `String` and a `StaticData`.
    /// The `String` represents the field name. The returned `StaticData` is a
    /// wrapper around the class (fully initialized) pointed to by
    /// `class_name`, and a `&'static mut StaticFields` that can be used to
    /// mutate the static fields of the class. Although the mutable ref to
    /// `StaticFields` is `'static`, the ref should never be kept around longer
    /// than the lifetime of the (Rust) stack frame that received it. This
    /// is to avoid running afoul of the Rust aliasing rules.
    fn unpack_f_name(
        &mut self,
        class_index: &usize,
        name_and_type: &usize,
    ) -> Result<(String, StaticData)> {
        let (name_index, _, data) = self.unpack(*class_index, *name_and_type)?;
        let f_name = self.frame().cp.get_utf8(name_index)?.into();
        Ok((f_name, data))
    }

    /// Takes in constant pool indices for the `Constant::Class(class_name)` and
    /// the `Constant::NameAndType` and returns a `String` and a `StaticData`.
    /// The `String` represents the method name. The returned `StaticData` is a
    /// wrapper around the class (fully initialized) pointed to by
    /// `class_name`, and a `&'static mut StaticFields` that can be used to
    /// mutate the static fields of the class. Although the mutable ref to
    /// `StaticFields` is `'static`, the ref should never be kept around longer
    /// than the lifetime of the (Rust) stack frame that received it. This
    /// is to avoid running afoul of the Rust aliasing rules.
    fn unpack_m_name(
        &mut self,
        class_index: usize,
        name_and_type: usize,
    ) -> Result<(String, StaticData)> {
        let (name_index, descr_index, data) = self.unpack(class_index, name_and_type)?;
        let name = self.construct_m_name(name_index, descr_index)?;
        Ok((name, data))
    }
}

/// Find the main method of a java class if it exists.
fn find_main(class: &Class) -> Result<&Method> {
    match class.methods.get(MAIN) {
        None => {
            bail!("No main method found.");
        }
        Some(method) => Ok(method),
    }
}
