#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;

use sql_docs::SqlDoc;

fuzz_target!(|data: String| {
    let _ = SqlDoc::from_str(&data).build();

    let _ = SqlDoc::from_str(&data)
        .deny(&data)
        .build();

    let _ = SqlDoc::from_str(&data)
        .flatten_multiline()
        .build();

    let _ = SqlDoc::from_str(&data)
        .flatten_multiline_with("")
        .build();

    let _ = SqlDoc::from_str(&data)
        .flatten_multiline_with(" ")
        .build();

    let _ = SqlDoc::from_str(&data)
        .flatten_multiline_with("\n")
        .build();

    let _ = SqlDoc::from_str(&data)
        .flatten_multiline_with(" | ")
        .build();

    let _ = SqlDoc::from_str(&data)
        .preserve_multiline()
        .build();

    let _ = SqlDoc::from_str(&data)
        .flatten_multiline()
        .preserve_multiline()
        .build();

    let _ = SqlDoc::from_str(&data)
        .preserve_multiline()
        .flatten_multiline()
        .build();

    let _ = SqlDoc::from_str(&data)
        .deny("nonexistent.sql")
        .flatten_multiline()
        .build();

    let _ = SqlDoc::from_str(&data)
        .deny("nonexistent.sql")
        .flatten_multiline_with(" ")
        .build();

    let _ = SqlDoc::from_str(&data)
        .deny(&data)
        .flatten_multiline_with("\n")
        .build();

    let _ = SqlDoc::from_str(&data)
        .deny("a.sql")
        .deny("b.sql")
        .flatten_multiline_with(" | ")
        .preserve_multiline()
        .flatten_multiline()
        .build();

});