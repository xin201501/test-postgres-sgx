fn main() {
    println!("cargo:rerun-if-changed=test.cpp");
    //fixed c/c++ flags for C enclave functions
    let enclave_common_c_flags= "-ffreestanding -nostdinc -fvisibility=hidden -fpie -fno-strict-overflow -fno-delete-null-pointer-checks".split(' ');
    let enclave_common_cxx_flags = enclave_common_c_flags.chain(vec!["-nostdinc++"]);
    
    let mut builder = cc::Build::new();
    //if compile c++ files use this option
    builder.cpp(true);
    //if compile c file use variable `enclave_common_c_flags`,if compile c++ file use variable `enclave_common_cxx_flags`
    for flag in enclave_common_cxx_flags {
        builder.flag(flag);
    }
    //custom build options
    builder
        .flag("-std=c++20")
        .opt_level(2)
        .debug(true)
        .file("test.cpp")
        .compile("test_c_enclave_function");
}
