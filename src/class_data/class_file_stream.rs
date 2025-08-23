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

use std::fs::File;
use std::io::Read;
use once_cell::sync::OnceCell;

static CLASS_PATHS: OnceCell<Vec<String>> = OnceCell::new();

pub fn initialize() {}

pub fn resolve_class_name(name: String) -> Option<Vec<u8>> {
    let relative = format!("{}.class", name);

    for n in CLASS_PATHS.get().unwrap().iter() {
        let path = format!("{}/{}", n, relative);

        if let Ok(mut f) = File::open(path) {
            let mut buf = Vec::new();

            return match f.read_to_end(&mut buf) {
                Ok(_) => Some(buf),
                _ => None,
            }
        }
    }

    None
}
