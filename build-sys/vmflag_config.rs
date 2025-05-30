/*
 * Copyright (c) 2025, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::{fs::File, io::Write};

macro_rules! declare_vm_flags {
    (
        [$category_name: ident]

        $((
            $type: ident,
            $flag_name: ident,
            $default_value: expr,
            $description: literal
        )),*
    ) => {
        {
            let mut buffer = Vec::new();
            
            $(
                buffer.push(
                    format!(
                        "pub static {}: {} = {};\n",
                        stringify!($flag_name),
                        stringify!($type),
                        stringify!($default_value)
                    )
                );
            )*

            $(
                buffer.push(
                    format!(
r#"
const Flag_{}: VMFlag = VMFlag {{
    _name: "{}",
    _type: "{}",
    _desc: "{}",
    _addr: &{} as *const _ as *mut _
}};"#,

                        stringify!($flag_name),
                        stringify!($flag_name),
                        stringify!($type),
                        $description,
                        stringify!($flag_name),
                    )
                );

                buffer.push(String::from("\n"));
            )*

            buffer.push(String::from("\n"));

            buffer.push(format!("pub static {}: Map<&str, &VMFlag> = phf_map! {{\n", stringify!($category_name)));
            $(
                buffer.push(format!(
                    "    \"{}\" => &Flag_{},\n",
                    stringify!($flag_name),
                    stringify!($flag_name)
                ));
            )*
            buffer.push(String::from("};\n"));

            buffer
        }
    };
}

fn construct_vmflag_maps() {
    let mut out = File::create("src/runtime/vmflag_map.rs").expect("Unable to create file.");

    out.write("// Generated by build.rs\n\n".as_bytes()).expect("Bad out stream.");

    let maps = vec![
        declare_vm_flags!(
            [RUNTIME_FLAGS]
            (bool, ExampleFlag, true, "Example."),
            (int, Test, 114514, "testing"),
            (bool, CompressedPtr, false, "Compressed pointers.")
        ),
    ];

    for m in maps {
        for n in m {
            out.write(n.as_bytes()).expect("Bad out stream.");
        }
    }
}
