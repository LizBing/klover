/*
 * Copyright 2025 Lei Zaakjyu
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

fn add_c_src(dir: &str, src_file_name: &str, include_path: &str) {
    let src_path = format!("{}/{}", dir, src_file_name);
    let output = format!("{}_c", src_file_name);

    cc::Build::new()
        .file(&src_path)
        .include(include_path)
        .compile(&output);

    println!("cargo::rerun-if-changed={}", &src_path);
}

fn add_c_module(dir: &str, name: &str, include_path: &str) {
    let src_file_name = format!("{}.c", name);
    let header_path = format!("{}/{}.h", dir, name);
    let header_output_path = format!("{}/{}.rs", dir, name);

    add_c_src(dir, &src_file_name, include_path);

    let bindings = bindgen::Builder::default()
        .header(header_path)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(header_output_path)
        .expect("Couldn't write bindings!");
}

fn main() {
    add_c_src("src/prims", "jni_impl.c", "src/prims");
    add_c_module("src/metaspace", "klass_allocator", "src/metaspace");
}
