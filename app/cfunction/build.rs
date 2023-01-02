fn main() {
    println!("cargo:rerun-if-changed=link_list.h");
    println!("cargo:rerun-if-changed=melbourne.cpp");
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++20")
        .opt_level(2)
        // .debug(true)
        .file("melbourne.cpp")
        .compile("melbourne");
}
