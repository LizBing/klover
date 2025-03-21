/*
 * Copyright (c) 2024, Lei Zaakjyu. All rights reserved.
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

use cafebabe;
use std::{fs::File, io::Read};

#[test]
fn test_classfile_parser() {
    let path = String::from("./tests/java_files/HelloWorld/HelloWorld.class");
    let mut file = File::open(path).unwrap();

    let mut bs = Vec::new();
    file.read_to_end(&mut bs).unwrap();

    let klass = cafebabe::parse_class(&mut bs).unwrap();
    println!("{:?}", klass);
}
