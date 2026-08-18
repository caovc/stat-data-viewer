#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(clippy::all)]

// Keep libz-sys in the link graph so zlib symbols (uncompress) resolve.
use libz_sys as _;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
