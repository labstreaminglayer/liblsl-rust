use std::env;


fn main() {
   // TODO: find out if liblsl already present on system and usable (if so, link to that instead)
   // println!("cargo:warning={}", "rebuilding...");
   build_liblsl();
}


fn build_liblsl() {
   // invoke cmake
   let mut cfg = cmake::Config::new("liblsl");
   cfg
       .define("LSL_NO_FANCY_LIBNAME", "ON")
       .define("LSL_BUILD_STATIC", "ON");
   let install_dir  = cfg.build();

   // derive paths
   let libdir = install_dir.join("lib");
   let libname = "lsl-static";

   // emit link directives to cargo
   println!("cargo:rustc-link-search=native={}", libdir.to_str().unwrap());
   println!("cargo:rustc-link-lib=static={}", libname);

   // make sure we also link the C++ stdlib (using its platform-specific name)
   let target  = env::var("TARGET").unwrap();
   if target.contains("linux") {
      println!("cargo:rustc-link-lib=dylib=stdc++");
   } else {
      println!("cargo:rustc-link-lib=dylib=c++");
   }
}
