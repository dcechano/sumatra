use crate::{
    alloc::{alloc_type::NonStatic, oop::HeapAlloc},
    data_types::{instance_data::InstanceData, value::Value},
    result::Result,
};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ObjRef(*mut HeapAlloc<NonStatic>);

impl ObjRef {
    pub fn new(
        InstanceData {
            primary,
            class_id,
            super_classes,
        }: InstanceData,
    ) -> Self {
        Self(HeapAlloc::<NonStatic>::new(
            primary,
            class_id,
            super_classes,
        ))
    }

    /// Create a `ObjRef` from a raw pointer to a `HeapAlloc`.
    /// SAFETY: This method is unsafe because careless use can lead to
    /// memory problems. `raw` must be a valid pointer to a `HeapAlloc`.
    pub unsafe fn from_raw(raw: *mut HeapAlloc<NonStatic>) -> Self { Self(raw) }

    /// Calculate the hash for the ptr backing this object instance
    pub fn hash_code(&self) -> i32 {
        let ptr = self.0 as usize;
        let u_32 = (ptr & u32::MAX as usize) as u32;
        unsafe { std::mem::transmute::<u32, i32>(u_32) }
    }

    pub fn set_field(&mut self, name: &str, value: Value) -> Result<()> {
        unsafe { (*self.0).set_field(name, value) }
    }

    pub fn get_field(&self, name: &str) -> Result<&'static Value> {
        unsafe { (*self.0).get_field(name) }
    }

    /// Get the inner value. Returned as a *const to emphasize that
    /// this return type should NOT be modified except by the GC.
    /// SAFETY: Nothing can be done to the pointer that would invalidate it,
    /// such as but not limited to dereferencing it and allowing the backing
    /// instance to drop. Allowing a drop by dereference will cause
    /// undefined behavior.
    pub fn get_inner(&self) -> *const HeapAlloc<NonStatic> { self.0 }

    pub fn get_class_id(&self) -> usize {
        // SAFETY: This is safe as long as the caller did not access
        // the backing raw pointer and do something to invalidate it.
        // The only way to do that is by using the `get_inner` method.
        unsafe { HeapAlloc::get_class_id(self.0) }
    }
}

impl Debug for ObjRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_tuple("ObjRef")
                .field(&(*self.0).header.name)
                .field(&format!("0x{:x}", self.0 as usize))
                .finish()
        }
    }
}
