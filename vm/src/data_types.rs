pub mod array;
pub mod instance_data;
pub mod object;
pub mod static_data;
pub mod value;

/// Calculate the hash of a ptr as a *mut u8
fn hash(ptr: *const u8) -> i32 {
    let data = ptr as u64;
    let hash = data >> 32 ^ (data);
    let hash = (hash & ((1 << 30) - 1)) as u32;

    unsafe { std::mem::transmute(hash) }
}
