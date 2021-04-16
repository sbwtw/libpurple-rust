extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("purple.rs");
    let mut bindings = bindgen::builder()
        .allowlist_type("Purple.*")
        .allowlist_function("purple_.*")
        .allowlist_var("PURPLE_.*")
        .bitfield_enum("PurpleIconScaleRules|PurpleProtocolOptions|")
        .newtype_enum("PurplePluginType");

    let purple_lib = pkg_config::probe_library("purple").unwrap();

    let mut versions = purple_lib
        .version
        .split(".")
        .map(|v| v.parse::<u32>().unwrap());
    let major = versions.next().unwrap();
    let minor = versions.next().unwrap();
    if major != 2 {
        panic!("Only support libpurple 2.x.x");
    }
    for x in 0..minor + 1 {
        println!("cargo:rustc-cfg=libpurple2_{}", x);
    }

    for path in &purple_lib.include_paths {
        let mut p = path.clone();
        p.push("purple.h");
        println!("test = {:?}", p);
        if p.exists() && p.is_file() {
            println!("found = {:?}", p);
            bindings = bindings.header(p.to_str().unwrap());
            break;
        }
    }

    bindings = bindings.clang_arg("-D").clang_arg("PURPLE_PLUGINS");
    for include_path in purple_lib.include_paths {
        let include_path = include_path.to_str().unwrap();

        println!("Adding include dir: {}", include_path);

        // Add each required include dir provided by pkg-config
        bindings = bindings.clang_arg("-I").clang_arg(include_path);
    }

    // bindings.forbid_unknown_types();
    bindings = bindings.emit_builtins();
    bindings = bindings.derive_debug(false);

    match bindings.generate() {
        Ok(bindings) => {
            bindings.write_to_file(out_file).unwrap();
        }
        _ => panic!("Bindings generation failed"),
    }
}
