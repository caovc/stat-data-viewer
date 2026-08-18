use std::env;
use std::path::{Path, PathBuf};

fn readstat_sources(src: &Path) -> Vec<PathBuf> {
    let files = [
        "CKHashTable.c",
        "readstat_bits.c",
        "readstat_convert.c",
        "readstat_error.c",
        "readstat_io_unistd.c",
        "readstat_malloc.c",
        "readstat_metadata.c",
        "readstat_parser.c",
        "readstat_value.c",
        "readstat_variable.c",
        "readstat_writer.c",
        "sas/ieee.c",
        "sas/readstat_sas.c",
        "sas/readstat_sas7bcat_read.c",
        "sas/readstat_sas7bcat_write.c",
        "sas/readstat_sas7bdat_read.c",
        "sas/readstat_sas7bdat_write.c",
        "sas/readstat_sas_rle.c",
        "sas/readstat_xport.c",
        "sas/readstat_xport_read.c",
        "sas/readstat_xport_write.c",
        "sas/readstat_xport_parse_format.c",
        "spss/readstat_por.c",
        "spss/readstat_por_parse.c",
        "spss/readstat_por_read.c",
        "spss/readstat_por_write.c",
        "spss/readstat_sav.c",
        "spss/readstat_sav_compress.c",
        "spss/readstat_sav_parse.c",
        "spss/readstat_sav_parse_timestamp.c",
        "spss/readstat_sav_read.c",
        "spss/readstat_sav_write.c",
        "spss/readstat_spss.c",
        "spss/readstat_spss_parse.c",
        "spss/readstat_zsav_compress.c",
        "spss/readstat_zsav_read.c",
        "spss/readstat_zsav_write.c",
        "stata/readstat_dta.c",
        "stata/readstat_dta_parse_timestamp.c",
        "stata/readstat_dta_read.c",
        "stata/readstat_dta_write.c",
        "txt/commands_util.c",
        "txt/readstat_copy.c",
        "txt/readstat_sas_commands_read.c",
        "txt/readstat_spss_commands_read.c",
        "txt/readstat_schema.c",
        "txt/readstat_stata_dictionary_read.c",
        "txt/readstat_txt_read.c",
    ];
    files.iter().map(|f| src.join(f)).collect()
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor_src = manifest_dir.join("vendor/ReadStat/src");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=vendor");

    if !vendor_src.join("readstat.h").exists() {
        panic!(
            "Vendored ReadStat is missing at {}. Restore crates/readstat-sys/vendor/ReadStat (v1.1.9).",
            vendor_src.display()
        );
    }

    let mut build = cc::Build::new();
    build
        .include(&vendor_src)
        .define("HAVE_ZLIB", "1")
        .flag_if_supported("-std=c99")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .files(readstat_sources(&vendor_src));

    if let Ok(include) = env::var("DEP_Z_INCLUDE") {
        build.include(include);
    }
    if cfg!(target_os = "macos") {
        build.flag("-mmacosx-version-min=11.0");
    }

    if cfg!(target_os = "windows") {
        let iconv_dir = manifest_dir.join("vendor/win-iconv");
        if !iconv_dir.join("win_iconv.c").exists() {
            panic!(
                "Vendored win-iconv is missing at {}. Restore crates/readstat-sys/vendor/win-iconv.",
                iconv_dir.display()
            );
        }
        build.include(&iconv_dir);
        build.file(iconv_dir.join("win_iconv.c"));
        build.define("_CRT_SECURE_NO_WARNINGS", None);
        build.define("WIN32_LEAN_AND_MEAN", None);
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=iconv");
    } else {
        // glibc provides iconv in libc. Distros without it (Alpine) need libiconv.
        println!("cargo:rustc-link-lib=m");
        if env::var("READSTAT_LINK_ICONV").ok().as_deref() == Some("1") {
            println!("cargo:rustc-link-lib=iconv");
        }
    }

    build.compile("readstat");

    let mut bindgen = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", vendor_src.display()))
        .allowlist_function("readstat_.*")
        .allowlist_type("readstat_.*")
        .allowlist_var("READSTAT_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if cfg!(target_os = "windows") {
        bindgen = bindgen.clang_arg(format!(
            "-I{}",
            manifest_dir.join("vendor/win-iconv").display()
        ));
    }

    let bindings = bindgen
        .generate()
        .expect("Unable to generate ReadStat bindings (needs libclang / LLVM)");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
