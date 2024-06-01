use sumatra_vm::{reference_types::ObjRef, vm::VM};

const OBJ: &str = "()Ljava/lang/Object;";
const CLS: &str = "()Ljava/lang/Class;";
const CPL: &str = "()Ljdk/internal/reflect/ConstantPool;";
const STR: &str = "()Ljava/lang/String;";
const FLD: &str = "()Ljava/lang/reflect/Field;";
const MHD: &str = "()Ljava/lang/reflect/Method;";
const CTR: &str = "()Ljava/lang/reflect/Constructor;";
const PD: &str = "()Ljava/security/ProtectionDomain;";
const BA: &str = "()[B";
const RC: &str = "()Ljava/lang/reflect/RecordComponent;";

fn jvm_register_natives() { todo!() }

fn jvm_get_class(vm: &mut VM) -> ObjRef { 
    todo!()
}

fn jvm_is_instance(vm: &mut VM) { todo!() }

fn jvm_is_assignable_from(vm: &mut VM) { todo!() }

fn jvm_is_interface(vm: &mut VM) { todo!() }

fn jvm_is_array() { todo!() }

fn jvm_is_primitive() { todo!() }

fn jvm_init_class_name() { todo!() }

fn jvm_get_super_class() { todo!() }

fn jvm_get_interfaces() { todo!() }

fn jvm_get_modifiers() { todo!() }

fn jvm_get_signers() { todo!() }

fn jvm_set_signers() { todo!() }

fn jvm_get_enclosing_method() { todo!() }

fn jvm_get_declaring_class() { todo!() }

fn jvm_get_simple_binary_name() { todo!() }

fn jvm_get_protection_domain() { todo!() }

fn jvm_get_primitive_class() { todo!() }

fn jvm_get_generic_signature() { todo!() }

fn jvm_get_raw_annotations() { todo!() }

fn jvm_get_raw_type_annotions() { todo!() }

fn jvm_get_constant_pool() { todo!() }

fn jvm_get_declared_fields() { todo!() }

fn jvm_get_declared_methods() { todo!() }

fn jvm_get_declared_constructors() { todo!() }

fn jvm_get_record_components() { todo!() }

fn jvm_is_record() { todo!() }

fn jvm_desired_assertion_status() { todo!() }

fn jvm_get_nest_host() { todo!() }

fn jvm_get_nested_members() { todo!() }

fn jvm_is_hidden() { todo!() }

fn jvm_get_permitted_subclasses() { todo!() }

fn jvm_get_file_version() { todo!() }

fn jvm_get_class_access_flags_raw() { todo!() }
