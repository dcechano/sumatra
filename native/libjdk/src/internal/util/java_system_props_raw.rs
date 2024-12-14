use sumatra_vm::{
    data_types::{
        array::ArrayComp,
        object::ObjRef,
        value::{
            RefType::{self},
            Value,
        },
    },
    result::Result,
    vm::VM,
};

const DISPLAY_COUNTRY_NDX: usize = 0;
const DISPLAY_LANGUAGE_NDX: usize = DISPLAY_COUNTRY_NDX + 1;
const DISPLAY_SCRIPT_NDX: usize = DISPLAY_LANGUAGE_NDX + 1;
const DISPLAY_VARIANT_NDX: usize = DISPLAY_SCRIPT_NDX + 1;
const FILE_ENCODING_NDX: usize = DISPLAY_VARIANT_NDX + 1;
const FILE_SEPARATOR_NDX: usize = FILE_ENCODING_NDX + 1;
const FORMAT_COUNTRY_NDX: usize = FILE_SEPARATOR_NDX + 1;
const FORMAT_LANGUAGE_NDX: usize = FORMAT_COUNTRY_NDX + 1;
const FORMAT_SCRIPT_NDX: usize = FORMAT_LANGUAGE_NDX + 1;
const FORMAT_VARIANT_NDX: usize = FORMAT_SCRIPT_NDX + 1;
const FTP_NON_PROXY_HOSTS_NDX: usize = FORMAT_VARIANT_NDX + 1;
const FTP_PROXY_HOST_NDX: usize = FTP_NON_PROXY_HOSTS_NDX + 1;
const FTP_PROXY_PORT_NDX: usize = FTP_PROXY_HOST_NDX + 1;
const HTTP_NON_PROXY_HOSTS_NDX: usize = FTP_PROXY_PORT_NDX + 1;
const HTTP_PROXY_HOST_NDX: usize = HTTP_NON_PROXY_HOSTS_NDX + 1;
const HTTP_PROXY_PORT_NDX: usize = HTTP_PROXY_HOST_NDX + 1;
const HTTPS_PROXY_HOST_NDX: usize = HTTP_PROXY_PORT_NDX + 1;
const HTTPS_PROXY_PORT_NDX: usize = HTTPS_PROXY_HOST_NDX + 1;
const JAVA_IO_TMPDIR_NDX: usize = HTTPS_PROXY_PORT_NDX + 1;
const LINE_SEPARATOR_NDX: usize = JAVA_IO_TMPDIR_NDX + 1;
const OS_ARCH_NDX: usize = LINE_SEPARATOR_NDX + 1;
const OS_NAME_NDX: usize = OS_ARCH_NDX + 1;
const OS_VERSION_NDX: usize = OS_NAME_NDX + 1;
const PATH_SEPARATOR_NDX: usize = OS_VERSION_NDX + 1;
const SOCKS_NON_PROXY_HOSTS_NDX: usize = PATH_SEPARATOR_NDX + 1;
const SOCKS_PROXY_HOST_NDX: usize = SOCKS_NON_PROXY_HOSTS_NDX + 1;
const SOCKS_PROXY_PORT_NDX: usize = SOCKS_PROXY_HOST_NDX + 1;
const STDERR_ENCODING_NDX: usize = SOCKS_PROXY_PORT_NDX + 1;
const STDOUT_ENCODING_NDX: usize = STDERR_ENCODING_NDX + 1;
const SUN_ARCH_ABI_NDX: usize = STDOUT_ENCODING_NDX + 1;
const SUN_ARCH_DATA_MODEL_NDX: usize = SUN_ARCH_ABI_NDX + 1;
const SUN_CPU_ENDIAN_NDX: usize = SUN_ARCH_DATA_MODEL_NDX + 1;
const SUN_CPU_ISALIST_NDX: usize = SUN_CPU_ENDIAN_NDX + 1;
const SUN_IO_UNICODE_ENCODING_NDX: usize = SUN_CPU_ISALIST_NDX + 1;
const SUN_JNU_ENCODING_NDX: usize = SUN_IO_UNICODE_ENCODING_NDX + 1;
const SUN_OS_PATCH_LEVEL_NDX: usize = SUN_JNU_ENCODING_NDX + 1;
const USER_DIR_NDX: usize = SUN_OS_PATCH_LEVEL_NDX + 1;
const USER_HOME_NDX: usize = USER_DIR_NDX + 1;
const USER_NAME_NDX: usize = USER_HOME_NDX + 1;

// unlisted in SystemProps.java but very important. See
// SystemProps.initProperties. Used in
// JDK_INTERNAL_UTIL_SYSTEMPROPS_RAW_vm_properties
const JAVA_HOME_NDX: usize = USER_NAME_NDX + 1;
const FIXED_LENGTH: usize = JAVA_HOME_NDX + 1;

// The number of static fields in jdk.internal.util.SystemProps.Raw.java
// https://github.com/openjdk/jdk/blob/jdk-21%2B35/src/java.base/share/classes/jdk/internal/util/SystemProps.java#L214
const NUM_PLATFORM_PROPS: usize = 40;
const NUM_VM_PROPS: usize = NUM_PLATFORM_PROPS + 1;

pub(crate) const PLATFORM_PROPS_SIG: &str = "platformProperties()[Ljava/lang/String;";
pub(crate) const VM_PROPS_SIG: &str = "vmProperties()[Ljava/lang/String;";

const STRING_CLASS: &str = "java/lang/String";

#[no_mangle]
pub fn JDK_INTERNAL_UTIL_SYSTEMPROPS_RAW_platform_properties(
    vm: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut string_array = vm.heap().new_array(
        NUM_PLATFORM_PROPS,
        ArrayComp::Class(STRING_CLASS.to_string()),
    );

    let mut prop = |s| {
        let obj = vm.create_java_string(s, true);
        Value::Ref(RefType::Object(obj))
    };

    string_array.insert(DISPLAY_COUNTRY_NDX, prop("US"));
    string_array.insert(DISPLAY_LANGUAGE_NDX, prop("English"));
    string_array.insert(DISPLAY_SCRIPT_NDX, prop("Latn"));
    string_array.insert(DISPLAY_VARIANT_NDX, prop(""));

    string_array.insert(FILE_ENCODING_NDX, prop("UTF-8"));
    string_array.insert(FILE_SEPARATOR_NDX, prop("/"));

    string_array.insert(FORMAT_COUNTRY_NDX, prop("US"));
    string_array.insert(FORMAT_LANGUAGE_NDX, prop("en_US"));
    string_array.insert(FORMAT_SCRIPT_NDX, prop("Latin"));
    string_array.insert(FORMAT_VARIANT_NDX, prop(""));

    string_array.insert(FTP_NON_PROXY_HOSTS_NDX, prop(""));
    string_array.insert(FTP_PROXY_HOST_NDX, prop(""));
    string_array.insert(FTP_PROXY_PORT_NDX, prop(""));

    string_array.insert(HTTP_NON_PROXY_HOSTS_NDX, prop(""));
    string_array.insert(HTTP_PROXY_HOST_NDX, prop(""));
    string_array.insert(HTTP_PROXY_PORT_NDX, prop(""));
    string_array.insert(HTTPS_PROXY_HOST_NDX, prop(""));
    string_array.insert(HTTPS_PROXY_PORT_NDX, prop(""));

    string_array.insert(JAVA_IO_TMPDIR_NDX, prop("/tmp"));
    string_array.insert(LINE_SEPARATOR_NDX, prop("\n"));
    string_array.insert(PATH_SEPARATOR_NDX, prop(":"));

    string_array.insert(OS_ARCH_NDX, prop("x86_64"));
    string_array.insert(OS_NAME_NDX, prop("Linux"));
    string_array.insert(OS_VERSION_NDX, prop("5.15"));

    string_array.insert(SOCKS_NON_PROXY_HOSTS_NDX, prop(""));
    string_array.insert(SOCKS_PROXY_HOST_NDX, prop(""));
    string_array.insert(SOCKS_PROXY_PORT_NDX, prop(""));

    string_array.insert(STDERR_ENCODING_NDX, prop("UTF-8"));
    string_array.insert(STDOUT_ENCODING_NDX, prop("UTF-8"));

    string_array.insert(SUN_ARCH_ABI_NDX, prop(""));
    string_array.insert(SUN_ARCH_DATA_MODEL_NDX, prop("64"));
    string_array.insert(SUN_CPU_ENDIAN_NDX, prop("little"));
    string_array.insert(SUN_CPU_ISALIST_NDX, prop("x86_64"));
    string_array.insert(SUN_IO_UNICODE_ENCODING_NDX, prop(""));
    string_array.insert(SUN_JNU_ENCODING_NDX, prop("UTF-8"));
    string_array.insert(SUN_OS_PATCH_LEVEL_NDX, prop(""));

    string_array.insert(USER_DIR_NDX, prop("/home/dylan"));
    string_array.insert(USER_HOME_NDX, prop("/home/dylan"));
    string_array.insert(USER_NAME_NDX, prop("dylan"));

    Ok(Some(Value::new_array(string_array)))
}

#[no_mangle]
pub fn JDK_INTERNAL_UTIL_SYSTEMPROPS_RAW_vm_properties(
    vm: &mut VM,
    _: Option<ObjRef>,
    _: Vec<Value>,
) -> Result<Option<Value>> {
    let mut string_array = vm
        .heap()
        .new_array(NUM_VM_PROPS * 2, ArrayComp::Class(STRING_CLASS.to_string()));

    let mut prop = |s| {
        let obj = vm.create_java_string(s, true);
        Value::Ref(RefType::Object(obj))
    };

    string_array.insert(DISPLAY_COUNTRY_NDX * 2, prop("display.country"));
    string_array.insert(DISPLAY_COUNTRY_NDX * 2 + 1, prop("US"));

    string_array.insert(DISPLAY_LANGUAGE_NDX * 2, prop("display.language"));
    string_array.insert(DISPLAY_LANGUAGE_NDX * 2 + 1, prop("English"));
    string_array.insert(DISPLAY_SCRIPT_NDX * 2, prop("display.script"));
    string_array.insert(DISPLAY_SCRIPT_NDX * 2 + 1, prop("Latn"));
    string_array.insert(DISPLAY_VARIANT_NDX * 2, prop("display.variant"));
    string_array.insert(DISPLAY_VARIANT_NDX * 2 + 1, prop(""));

    string_array.insert(FILE_ENCODING_NDX * 2, prop("file.encoding"));
    string_array.insert(FILE_ENCODING_NDX * 2 + 1, prop("UTF-8"));
    string_array.insert(FILE_SEPARATOR_NDX * 2, prop("file.separator"));
    string_array.insert(FILE_SEPARATOR_NDX * 2 + 1, prop("/"));

    string_array.insert(FORMAT_COUNTRY_NDX * 2, prop("format.country"));
    string_array.insert(FORMAT_COUNTRY_NDX * 2 + 1, prop("US"));
    string_array.insert(FORMAT_LANGUAGE_NDX * 2, prop("format.language"));
    string_array.insert(FORMAT_LANGUAGE_NDX * 2 + 1, prop("en_US"));
    string_array.insert(FORMAT_SCRIPT_NDX * 2, prop("format.script"));
    string_array.insert(FORMAT_SCRIPT_NDX * 2 + 1, prop("Latin"));
    string_array.insert(FORMAT_VARIANT_NDX * 2, prop("format.variant"));
    string_array.insert(FORMAT_VARIANT_NDX * 2 + 1, prop(""));

    string_array.insert(FTP_NON_PROXY_HOSTS_NDX * 2, prop("ftp.nonproxy.hosts"));
    string_array.insert(FTP_NON_PROXY_HOSTS_NDX * 2 + 1, prop(""));
    string_array.insert(FTP_PROXY_HOST_NDX * 2, prop("ftp.proxy.host"));
    string_array.insert(FTP_PROXY_HOST_NDX * 2 + 1, prop(""));
    string_array.insert(FTP_PROXY_PORT_NDX * 2, prop("ftp.proxy.port"));
    string_array.insert(FTP_PROXY_PORT_NDX * 2 + 1, prop(""));

    string_array.insert(HTTP_NON_PROXY_HOSTS_NDX * 2, prop("http.nonproxy.hosts"));
    string_array.insert(HTTP_NON_PROXY_HOSTS_NDX * 2 + 1, prop(""));
    string_array.insert(HTTP_PROXY_HOST_NDX * 2, prop("http.proxy.host"));
    string_array.insert(HTTP_PROXY_HOST_NDX * 2 + 1, prop(""));
    string_array.insert(HTTP_PROXY_PORT_NDX * 2, prop("http.proxy.port"));
    string_array.insert(HTTP_PROXY_PORT_NDX * 2 + 1, prop(""));
    string_array.insert(HTTPS_PROXY_HOST_NDX * 2, prop("https.proxy.host"));
    string_array.insert(HTTPS_PROXY_HOST_NDX * 2 + 1, prop(""));
    string_array.insert(HTTPS_PROXY_PORT_NDX * 2, prop("http.proxy.port"));
    string_array.insert(HTTPS_PROXY_PORT_NDX * 2 + 1, prop(""));

    string_array.insert(JAVA_IO_TMPDIR_NDX * 2, prop("java.io.tmpdir"));
    string_array.insert(JAVA_IO_TMPDIR_NDX * 2 + 1, prop("/tmp"));
    string_array.insert(LINE_SEPARATOR_NDX * 2, prop("line.separator"));
    string_array.insert(LINE_SEPARATOR_NDX * 2 + 1, prop("\n"));
    string_array.insert(PATH_SEPARATOR_NDX * 2, prop("path.separator"));
    string_array.insert(PATH_SEPARATOR_NDX * 2 + 1, prop(":"));

    string_array.insert(OS_ARCH_NDX * 2, prop("os.arch"));
    string_array.insert(OS_ARCH_NDX * 2 + 1, prop("x86_64"));
    string_array.insert(OS_NAME_NDX * 2, prop("os.name"));
    string_array.insert(OS_NAME_NDX * 2 + 1, prop("Linux"));
    string_array.insert(OS_VERSION_NDX * 2, prop("os.version"));
    string_array.insert(OS_VERSION_NDX * 2 + 1, prop("5.15"));

    string_array.insert(SOCKS_NON_PROXY_HOSTS_NDX * 2, prop("socks.nonproxy.hosts"));
    string_array.insert(SOCKS_NON_PROXY_HOSTS_NDX * 2 + 1, prop(""));
    string_array.insert(SOCKS_PROXY_HOST_NDX * 2, prop(""));
    string_array.insert(SOCKS_PROXY_HOST_NDX * 2 + 1, prop("socks.proxy.host"));
    string_array.insert(SOCKS_PROXY_PORT_NDX * 2, prop(""));
    string_array.insert(SOCKS_PROXY_PORT_NDX * 2 + 1, prop("socks.proxy.port"));

    string_array.insert(STDERR_ENCODING_NDX * 2, prop("stderr.encoding"));
    string_array.insert(STDERR_ENCODING_NDX * 2 + 1, prop("UTF-8"));
    string_array.insert(STDOUT_ENCODING_NDX * 2, prop("std.encoding"));
    string_array.insert(STDOUT_ENCODING_NDX * 2 + 1, prop("UTF-8"));

    string_array.insert(SUN_ARCH_ABI_NDX * 2, prop("sun.arch.abi"));
    string_array.insert(SUN_ARCH_ABI_NDX * 2 + 1, prop(""));
    string_array.insert(SUN_ARCH_DATA_MODEL_NDX * 2, prop("sun.arch.data.model"));
    string_array.insert(SUN_ARCH_DATA_MODEL_NDX * 2 + 1, prop("64"));
    string_array.insert(SUN_CPU_ENDIAN_NDX * 2, prop("sun.cpu.endian"));
    string_array.insert(SUN_CPU_ENDIAN_NDX * 2 + 1, prop("little"));
    string_array.insert(SUN_CPU_ISALIST_NDX * 2, prop("sun.cpu.isalist"));
    string_array.insert(SUN_CPU_ISALIST_NDX * 2 + 1, prop("x86_64"));
    string_array.insert(
        SUN_IO_UNICODE_ENCODING_NDX * 2,
        prop("sun.io.unicode.encoding"),
    );
    string_array.insert(SUN_IO_UNICODE_ENCODING_NDX * 2 + 1, prop(""));
    string_array.insert(SUN_JNU_ENCODING_NDX * 2, prop("sun.jnu.encoding"));
    string_array.insert(SUN_JNU_ENCODING_NDX * 2 + 1, prop("UTF-8"));
    string_array.insert(SUN_OS_PATCH_LEVEL_NDX * 2, prop("sun.os.patch.level"));
    string_array.insert(SUN_OS_PATCH_LEVEL_NDX * 2 + 1, prop(""));

    string_array.insert(USER_DIR_NDX * 2, prop("user.dir"));
    string_array.insert(USER_DIR_NDX * 2 + 1, prop("/home/dylan"));
    string_array.insert(USER_HOME_NDX * 2, prop("user.home"));
    string_array.insert(USER_HOME_NDX * 2 + 1, prop("/home/dylan"));
    string_array.insert(USER_NAME_NDX * 2, prop("user.name"));
    string_array.insert(USER_NAME_NDX * 2 + 1, prop("dylan"));
    string_array.insert(JAVA_HOME_NDX * 2, prop("java.home"));
    string_array.insert(JAVA_HOME_NDX * 2 + 1, prop(""));

    Ok(Some(Value::new_array(string_array)))
}
