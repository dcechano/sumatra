use anyhow::{bail, Result};

use sumatra_parser::{constant::Constant, instruction::Instruction, method::Method};

use crate::{call_frame::CallFrame, class::Class, class_loader::ClassLoader, value::Value};

const MAIN: &str = "main([Ljava/lang/String;)V";

#[derive(Default)]
pub struct VM<'vm> {
    //TODO implement call frames and the method area
    // pub(crate) call_frames: Vec<CallFrame<'vm>>,
    pub(crate) stack: Vec<Value<'vm>>,
    pub(crate) class_loader: ClassLoader,
}

impl<'vm> VM<'vm> {
    pub fn run(&mut self, class: &'vm mut Class) -> Result<()> {
        let main = find_main(class)?;
        let cp = &class.cp;
        self.execute_frame(&mut CallFrame::construct_cf(main, cp))?;
        Ok(())
    }

    fn execute_frame(&mut self, call_frame: &mut CallFrame) -> Result<()> {
        let code = &call_frame.method.code;
        let op_code = &code.op_code;
        for code in op_code {
            match code {
                Instruction::AaLoad => {}
                Instruction::AaStore => {}
                Instruction::AaConstNull => {}
                Instruction::ALoad => {}
                Instruction::ALoad0 => {}
                Instruction::ALoad1 => {}
                Instruction::ALoad2 => {}
                Instruction::ALoad3 => {}
                Instruction::ANewArray(_) => {}
                Instruction::AReturn => {}
                Instruction::ArrayLength => {}
                Instruction::AStore(_) => {}
                Instruction::AStore0 => {}
                Instruction::AStore1 => {}
                Instruction::AStore2 => {}
                Instruction::AStore3 => {}
                Instruction::AThrow => {}
                Instruction::BaLoad => {}
                Instruction::BaStore => {}
                Instruction::BiPush(_) => {}
                Instruction::CaLoad => {}
                Instruction::CaStore => {}
                Instruction::Checkcast(_) => {}
                Instruction::D2F => {}
                Instruction::D2I => {}
                Instruction::D2L => {}
                Instruction::DAdd => {}
                Instruction::DaLoad => {}
                Instruction::DaStore => {}
                Instruction::Dcmpg => {}
                Instruction::Dcmpl => {}
                Instruction::DConst0 => {}
                Instruction::DConst1 => {}
                Instruction::DDiv => {}
                Instruction::DLoad => {}
                Instruction::DLoad0 => {}
                Instruction::DLoad1 => {}
                Instruction::DLoad2 => {}
                Instruction::DLoad3 => {}
                Instruction::DMul => {}
                Instruction::DNeg => {}
                Instruction::DRem => {}
                Instruction::DReturn => {}
                Instruction::DStore(_) => {}
                Instruction::DStore0 => {}
                Instruction::DStore1 => {}
                Instruction::DStore2 => {}
                Instruction::DStore3 => {}
                Instruction::DSub => {}
                Instruction::Dup => {}
                Instruction::DupX1 => {}
                Instruction::DupX2 => {}
                Instruction::Dup2 => {}
                Instruction::Dup2X1 => {}
                Instruction::Dup2X2 => {}
                Instruction::F2D => {}
                Instruction::F2I => {}
                Instruction::F2L => {}
                Instruction::FAdd => {}
                Instruction::FaLoad => {}
                Instruction::FaStore => {}
                Instruction::Fcmpg => {}
                Instruction::Fcmpl => {}
                Instruction::FConst0 => {}
                Instruction::FConst1 => {}
                Instruction::FConst2 => {}
                Instruction::FDiv => {}
                Instruction::FLoad(_) => {}
                Instruction::FLoad0 => {}
                Instruction::FLoad1 => {}
                Instruction::FLoad2 => {}
                Instruction::FLoad3 => {}
                Instruction::FMul => {}
                Instruction::FNeg => {}
                Instruction::FRem => {}
                Instruction::FReturn => {}
                Instruction::FStore(_) => {}
                Instruction::FStore0 => {}
                Instruction::FStore1 => {}
                Instruction::FStore2 => {}
                Instruction::FStore3 => {}
                Instruction::FSub => {}
                Instruction::GetField(_) => {}
                Instruction::GetStatic(index) => self.get_static(call_frame, *index)?,
                Instruction::GoTo(_) => {}
                Instruction::GoToW(_) => {}
                Instruction::I2B => {}
                Instruction::I2C => {}
                Instruction::I2D => {}
                Instruction::I2F => {}
                Instruction::I2L => {}
                Instruction::I2S => {}
                Instruction::IAdd => {}
                Instruction::IaLoad => {}
                Instruction::IAnd => {}
                Instruction::IaStore => {}
                Instruction::IConstM1 => {}
                Instruction::IConst0 => {}
                Instruction::IConst1 => {}
                Instruction::IConst2 => {}
                Instruction::IConst3 => {}
                Instruction::IConst4 => {}
                Instruction::IConst5 => {}
                Instruction::IDiv => {}
                Instruction::IfAcmpeq(_) => {}
                Instruction::IfAcmpne(_) => {}
                Instruction::IfIcmpeq(_) => {}
                Instruction::IfIcmpne(_) => {}
                Instruction::IfIcmplt(_) => {}
                Instruction::IfIcmpge(_) => {}
                Instruction::IfIcmpgt(_) => {}
                Instruction::IfIcmple(_) => {}
                Instruction::Ifeq(_) => {}
                Instruction::Ifne(_) => {}
                Instruction::Iflt(_) => {}
                Instruction::Ifge(_) => {}
                Instruction::Ifgt(_) => {}
                Instruction::Ifle(_) => {}
                Instruction::IfNonNull(_) => {}
                Instruction::IfNull(_) => {}
                Instruction::Iinc(_, _) => {}
                Instruction::ILoad(_) => {}
                Instruction::ILoad0 => {}
                Instruction::ILoad1 => {}
                Instruction::ILoad2 => {}
                Instruction::ILoad3 => {}
                Instruction::IMul => {}
                Instruction::INeg => {}
                Instruction::InstanceOf(_) => {}
                Instruction::InvokeDynamic(_, _, _) => {}
                Instruction::InvokeInterface(_, _, _) => {}
                Instruction::InvokeSpecial(_) => {}
                Instruction::InvokeStatic(_) => {}
                Instruction::InvokeVirtual(method_index) => {
                    println!("INVOKE_VIRTUAL!: #{method_index}");
                }
                Instruction::IOr => {}
                Instruction::IRem => {}
                Instruction::IReturn => {}
                Instruction::IShL => {}
                Instruction::IShR => {}
                Instruction::IStore => {}
                Instruction::IStore0 => {}
                Instruction::IStore1 => {}
                Instruction::IStore2 => {}
                Instruction::IStore3 => {}
                Instruction::ISub => {}
                Instruction::IuShR => {}
                Instruction::IxOr => {}
                Instruction::Jsr(_) => {}
                Instruction::JsrW(_) => {}
                Instruction::L2D => {}
                Instruction::L2F => {}
                Instruction::L2I => {}
                Instruction::LAdd => {}
                Instruction::LaLoad => {}
                Instruction::LAnd => {}
                Instruction::LaStore => {}
                Instruction::Lcmp => {}
                Instruction::LConst0 => {}
                Instruction::LConst1 => {}
                Instruction::Ldc(index) => {
                    let constant = call_frame.cp.get(*index).unwrap();
                    match constant {
                        Constant::Dummy => {
                            todo!()
                        }
                        Constant::UTF8(_) => {
                            todo!()
                        }
                        Constant::Integer(_) => {
                            todo!()
                        }
                        Constant::Float(_) => {
                            todo!()
                        }
                        Constant::Long(_) => {
                            todo!()
                        }
                        Constant::Double(_) => {
                            todo!()
                        }
                        Constant::Class(_) => {
                            todo!()
                        }
                        Constant::String(string_index) => {
                            todo!()
                            // call_frame.op_stack.push(Value::Object(string.
                            // into()));
                        }
                        Constant::FieldRef { .. } => {
                            todo!()
                        }
                        Constant::MethodRef { .. } => {
                            todo!()
                        }
                        Constant::InterfaceMethodRef { .. } => {
                            todo!()
                        }
                        Constant::NameAndType { .. } => {
                            todo!()
                        }
                        Constant::MethodHandle { .. } => {
                            todo!()
                        }
                        Constant::MethodType(_) => {
                            todo!()
                        }
                        Constant::Dynamic { .. } => {
                            todo!()
                        }
                        Constant::InvokeDynamic { .. } => {
                            todo!()
                        }
                        Constant::Module(_) => {
                            todo!()
                        }
                        Constant::Package(_) => {
                            todo!()
                        }
                    }
                }
                Instruction::LdcW(_) => {}
                Instruction::Ldc2W(_) => {}
                Instruction::LDiv => {}
                Instruction::LLoad(_) => {}
                Instruction::LLoad0 => {}
                Instruction::LLoad1 => {}
                Instruction::LLoad2 => {}
                Instruction::LLoad3 => {}
                Instruction::LMul => {}
                Instruction::LNeg => {}
                Instruction::LookUpSwitch { .. } => {}
                Instruction::LOr => {}
                Instruction::LRem => {}
                Instruction::LReturn => {}
                Instruction::LShL => {}
                Instruction::LShR => {}
                Instruction::LStore(_) => {}
                Instruction::LStore0 => {}
                Instruction::LStore1 => {}
                Instruction::LStore2 => {}
                Instruction::LStore3 => {}
                Instruction::LSub => {}
                Instruction::LuShR => {}
                Instruction::LxOr => {}
                Instruction::MonitorEnter => {}
                Instruction::MonitorExit => {}
                Instruction::MultiaNewArray(_, _) => {}
                Instruction::New(_) => {}
                Instruction::NewArray(_) => {}
                Instruction::Nop => {}
                Instruction::Pop => {}
                Instruction::Pop2 => {}
                Instruction::PutField(_) => {}
                Instruction::PutStatic(_) => {}
                Instruction::Ret => {}
                Instruction::Return => {}
                Instruction::SaLoad => {}
                Instruction::SaStore => {}
                Instruction::SiPush(_) => {}
                Instruction::Swap => {}
                Instruction::TableSwitch { .. } => {}
                Instruction::Wide(_, _, _) => {}
            }
        }
        println!(
            "Printing the call frame operand stack {:?}.",
            call_frame.op_stack
        );
        Ok(())
    }

    fn get_static(&mut self, call_frame: &mut CallFrame, index: usize) -> Result<()> {
        if let Some(Constant::FieldRef {
            class_index,
            name_and_type_index,
        }) = call_frame.cp.get(index)
        {
            if let Constant::Class(name_index) = call_frame.cp.get(*class_index).unwrap() {
                let name = call_frame.cp.get_utf8(*name_index)?;
                self.class_loader.resolve(name)?;
                println!("Loading class {name} was successful.");
                todo!();
                // call_frame.op_stack.push(Value::RefType(name.into()));
            };
            if let Constant::NameAndType {
                name_index,
                descriptor_index,
            } = call_frame.cp.get(*name_and_type_index).unwrap()
            {
                todo!()
            } else {
                bail!("Fieldref name_and_type_index did not point to a NameAndType constant.");
            }
        } else {
            bail!("get_static constant was not a FieldRef constant.");
        }
        Ok(())
    }
}

fn find_main(class: &Class) -> Result<&Method> {
    match class.methods.get(MAIN) {
        None => {
            bail!("No main method found.");
        }
        Some(method) => Ok(method),
    }
}
