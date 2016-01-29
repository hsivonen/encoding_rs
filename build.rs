// Copyright 2016 Mozilla Foundation. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate cheddar;

fn main() {
    cheddar::Cheddar::new().expect("could not read manifest")
        .module("ffi").expect("malformed module path")
        .run_build("target/include/encoding_rs.h");
}