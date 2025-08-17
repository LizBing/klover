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

use cafebabe::{descriptors::FieldType, AccessFlags, ClassFile, FieldAccessFlags, FieldInfo};

use crate::{align_up, common::universe, oops::{klass::Klass, obj_desc::{self, ObjDesc}}, runtime::runtime_globals::{self, USE_COMPRESSED_OOPS}, utils::{align, global_defs::{self, address, naddr}}};

#[derive(Debug)]
pub struct Field<'a> {
    _info: &'a FieldInfo<'a>,
    _offs: usize,   // offset from the obj header(ObjPtr)
}

impl<'a> Field<'a> {
    pub fn new(info: &'a FieldInfo, offs: usize) -> Self {
        Self {
            _info: info,
            _offs: offs,
        }
    }
}

impl Field<'_> {
    // the offset from ObjPtr
    pub fn offset(&self) -> usize {
        self._offs
    }
}

#[derive(Debug)]
pub struct Fields<'a> {
    pub instance_fields: Vec<Field<'a>>,
    pub size_of_instance: usize,
    pub offs_of_instance_refs: usize,
    pub instance_refs: usize,

    // describes klass mirror
    pub static_fields: Vec<Field<'a>>,
    pub size_of_statics: usize,
    pub offs_of_static_refs: usize,
    pub static_refs: usize
}

struct SortResult<'a> {
    _1_byte_ins: Vec<&'a FieldInfo<'a>>,
    _2_bytes_ins: Vec<&'a FieldInfo<'a>>,
    _4_bytes_ins: Vec<&'a FieldInfo<'a>>,
    _8_bytes_ins: Vec<&'a FieldInfo<'a>>,
    ref_ins: Vec<&'a FieldInfo<'a>>,
    
    _1_byte_sta: Vec<&'a FieldInfo<'a>>,
    _2_bytes_sta: Vec<&'a FieldInfo<'a>>,
    _4_bytes_sta: Vec<&'a FieldInfo<'a>>,
    _8_bytes_sta: Vec<&'a FieldInfo<'a>>,
    ref_sta: Vec<&'a FieldInfo<'a>>,
}

impl<'a> Fields<'a> {
    fn sort(infos: &'a Vec<FieldInfo<'a>>) -> SortResult<'a> {
        let mut res = SortResult {
            _1_byte_ins: Vec::new(),
            _2_bytes_ins: Vec::new(),
            _4_bytes_ins: Vec::new(),
            _8_bytes_ins: Vec::new(),
            ref_ins: Vec::new(),

            _1_byte_sta: Vec::new(),
            _2_bytes_sta: Vec::new(),
            _4_bytes_sta: Vec::new(),
            _8_bytes_sta: Vec::new(),
            ref_sta: Vec::new(),
        };

        for info in infos {
            if info.descriptor.dimensions != 0 {
                if info.access_flags.contains(FieldAccessFlags::STATIC) {
                    res.ref_sta.push(info);
                } else {
                    res.ref_ins.push(info);
                }
            } else {
                match info.descriptor.field_type {
                    FieldType::Boolean | FieldType::Byte => {
                        if info.access_flags.contains(FieldAccessFlags::STATIC) {
                            res._1_byte_sta.push(info);
                        } else {
                            res._1_byte_ins.push(info);
                        }
                    }

                    FieldType::Char | FieldType::Short => {
                        if info.access_flags.contains(FieldAccessFlags::STATIC) {
                            res._2_bytes_sta.push(info);
                        } else {
                            res._2_bytes_ins.push(info);
                        }
                    }

                    FieldType::Integer | FieldType::Float => {
                        if info.access_flags.contains(FieldAccessFlags::STATIC) {
                            res._4_bytes_sta.push(info);
                        } else {
                            res._4_bytes_ins.push(info);
                        }
                    }

                    FieldType::Long | FieldType::Double => {
                        if info.access_flags.contains(FieldAccessFlags::STATIC) {
                            res._8_bytes_sta.push(info);
                        } else {
                            res._8_bytes_ins.push(info);
                        }
                    }

                    FieldType::Object(_) => {
                        if info.access_flags.contains(FieldAccessFlags::STATIC) {
                            res.ref_sta.push(info);
                        } else {
                            res.ref_ins.push(info);
                        }
                    }
                }
            }
        }

        res
    }

    fn layout_fields(src: &Vec<&'a FieldInfo>, offs: &mut usize, align: usize) -> Vec<Field<'a>> {
        let mut res = Vec::new();

        for n in src {
            *offs = align_up!(*offs, align);
            *offs += align;
            res.push(Field::new(*n, *offs));
        }

        res
    }
}

impl<'a> Fields<'a> {
    pub fn new() -> Self {
        Self {
            instance_fields: Vec::new(),
            size_of_instance: 0,
            offs_of_instance_refs: 0,
            instance_refs: 0,
            static_fields: Vec::new(),
            size_of_statics: 0,
            offs_of_static_refs: 0,
            static_refs: 0,
        }
    }

    pub fn init(&mut self, mut offs: usize, infos: &'a Vec<FieldInfo>) {
        let ref_align;
        if USE_COMPRESSED_OOPS.get_value() {
            ref_align = size_of::<naddr>();
        } else {
            ref_align = size_of::<address>();
        }

        let sorted = Self::sort(infos);
        self.instance_refs = sorted.ref_ins.len();
        self.static_refs = sorted.ref_sta.len();

        let mut instance_fields = Vec::new();
        instance_fields.append(&mut Self::layout_fields(&sorted._8_bytes_ins, &mut offs, 8));
        instance_fields.append(&mut Self::layout_fields(&sorted._4_bytes_ins, &mut offs, 4));
        instance_fields.append(&mut Self::layout_fields(&sorted._2_bytes_ins, &mut offs, 2));
        instance_fields.append(&mut Self::layout_fields(&sorted._1_byte_ins, &mut offs, 1));
        self.offs_of_static_refs = align_up!(offs, ref_align);
        instance_fields.append(&mut Self::layout_fields(&sorted.ref_ins, &mut offs, ref_align));
        self.instance_fields = instance_fields;
        self.size_of_instance = offs;

        let mut offs_statics = ObjDesc::size_of_normal_desc() + size_of::<&Klass>();
        let mut static_fields = Vec::new();
        static_fields.append(&mut Self::layout_fields(&sorted._8_bytes_sta, &mut offs_statics, 8));
        static_fields.append(&mut Self::layout_fields(&sorted._4_bytes_sta, &mut offs_statics, 4));
        static_fields.append(&mut Self::layout_fields(&sorted._2_bytes_sta, &mut offs_statics, 2));
        static_fields.append(&mut Self::layout_fields(&sorted._1_byte_sta, &mut offs_statics, 1));
        self.offs_of_static_refs = align_up!(offs_statics, ref_align);
        static_fields.append(&mut Self::layout_fields(&sorted.ref_sta, &mut offs_statics, ref_align));
        self.size_of_statics = offs_statics;
    }

}
