/*
 * Copyright (c) 2025, Lei Zaakjyu. All rights reserved.
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the "License
"* ); you may not use this file except in compliance  
w*ith the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

pub mod Opcode {
    pub const _illegal: u8              =  u8::MAX;

    // Java bytecodes
    pub const _nop: u8                  =   0; // 0x00
    pub const _aconst_null: u8          =   1; // 0x01
    pub const _iconst_m1: u8            =   2; // 0x02
    pub const _iconst_0: u8             =   3; // 0x03
    pub const _iconst_1: u8             =   4; // 0x04
    pub const _iconst_2: u8             =   5; // 0x05
    pub const _iconst_3: u8             =   6; // 0x06
    pub const _iconst_4: u8             =   7; // 0x07
    pub const _iconst_5: u8             =   8; // 0x08
    pub const _lconst_0: u8             =   9; // 0x09
    pub const _lconst_1: u8             =  10; // 0x0a
    pub const _fconst_0: u8             =  11; // 0x0b
    pub const _fconst_1: u8             =  12; // 0x0c
    pub const _fconst_2: u8             =  13; // 0x0d
    pub const _dconst_0: u8             =  14; // 0x0e
    pub const _dconst_1: u8             =  15; // 0x0f
    pub const _bipush: u8               =  16; // 0x10
    pub const _sipush: u8               =  17; // 0x11
    pub const _ldc: u8                  =  18; // 0x12
    pub const _ldc_w: u8                =  19; // 0x13
    pub const _ldc2_w: u8               =  20; // 0x14
    pub const _iload: u8                =  21; // 0x15
    pub const _lload: u8                =  22; // 0x16
    pub const _fload: u8                =  23; // 0x17
    pub const _dload: u8                =  24; // 0x18
    pub const _aload: u8                =  25; // 0x19
    pub const _iload_0: u8              =  26; // 0x1a
    pub const _iload_1: u8              =  27; // 0x1b
    pub const _iload_2: u8              =  28; // 0x1c
    pub const _iload_3: u8              =  29; // 0x1d
    pub const _lload_0: u8              =  30; // 0x1e
    pub const _lload_1: u8              =  31; // 0x1f
    pub const _lload_2: u8              =  32; // 0x20
    pub const _lload_3: u8              =  33; // 0x21
    pub const _fload_0: u8              =  34; // 0x22
    pub const _fload_1: u8              =  35; // 0x23
    pub const _fload_2: u8              =  36; // 0x24
    pub const _fload_3: u8              =  37; // 0x25
    pub const _dload_0: u8              =  38; // 0x26
    pub const _dload_1: u8              =  39; // 0x27
    pub const _dload_2: u8              =  40; // 0x28
    pub const _dload_3: u8              =  41; // 0x29
    pub const _aload_0: u8              =  42; // 0x2a
    pub const _aload_1: u8              =  43; // 0x2b
    pub const _aload_2: u8              =  44; // 0x2c
    pub const _aload_3: u8              =  45; // 0x2d
    pub const _iaload: u8               =  46; // 0x2e
    pub const _laload: u8               =  47; // 0x2f
    pub const _faload: u8               =  48; // 0x30
    pub const _daload: u8               =  49; // 0x31
    pub const _aaload: u8               =  50; // 0x32
    pub const _baload: u8               =  51; // 0x33
    pub const _caload: u8               =  52; // 0x34
    pub const _saload: u8               =  53; // 0x35
    pub const _istore: u8               =  54; // 0x36
    pub const _lstore: u8               =  55; // 0x37
    pub const _fstore: u8               =  56; // 0x38
    pub const _dstore: u8               =  57; // 0x39
    pub const _astore: u8               =  58; // 0x3a
    pub const _istore_0: u8             =  59; // 0x3b
    pub const _istore_1: u8             =  60; // 0x3c
    pub const _istore_2: u8             =  61; // 0x3d
    pub const _istore_3: u8             =  62; // 0x3e
    pub const _lstore_0: u8             =  63; // 0x3f
    pub const _lstore_1: u8             =  64; // 0x40
    pub const _lstore_2: u8             =  65; // 0x41
    pub const _lstore_3: u8             =  66; // 0x42
    pub const _fstore_0: u8             =  67; // 0x43
    pub const _fstore_1: u8             =  68; // 0x44
    pub const _fstore_2: u8             =  69; // 0x45
    pub const _fstore_3: u8             =  70; // 0x46
    pub const _dstore_0: u8             =  71; // 0x47
    pub const _dstore_1: u8             =  72; // 0x48
    pub const _dstore_2: u8             =  73; // 0x49
    pub const _dstore_3: u8             =  74; // 0x4a
    pub const _astore_0: u8             =  75; // 0x4b
    pub const _astore_1: u8             =  76; // 0x4c
    pub const _astore_2: u8             =  77; // 0x4d
    pub const _astore_3: u8             =  78; // 0x4e
    pub const _iastore: u8              =  79; // 0x4f
    pub const _lastore: u8              =  80; // 0x50
    pub const _fastore: u8              =  81; // 0x51
    pub const _dastore: u8              =  82; // 0x52
    pub const _aastore: u8              =  83; // 0x53
    pub const _bastore: u8              =  84; // 0x54
    pub const _castore: u8              =  85; // 0x55
    pub const _sastore: u8              =  86; // 0x56
    pub const _pop: u8                  =  87; // 0x57
    pub const _pop2: u8                 =  88; // 0x58
    pub const _dup: u8                  =  89; // 0x59
    pub const _dup_x1: u8               =  90; // 0x5a
    pub const _dup_x2: u8               =  91; // 0x5b
    pub const _dup2: u8                 =  92; // 0x5c
    pub const _dup2_x1: u8              =  93; // 0x5d
    pub const _dup2_x2: u8              =  94; // 0x5e
    pub const _swap: u8                 =  95; // 0x5f
    pub const _iadd: u8                 =  96; // 0x60
    pub const _ladd: u8                 =  97; // 0x61
    pub const _fadd: u8                 =  98; // 0x62
    pub const _dadd: u8                 =  99; // 0x63
    pub const _isub: u8                 = 100; // 0x64
    pub const _lsub: u8                 = 101; // 0x65
    pub const _fsub: u8                 = 102; // 0x66
    pub const _dsub: u8                 = 103; // 0x67
    pub const _imul: u8                 = 104; // 0x68
    pub const _lmul: u8                 = 105; // 0x69
    pub const _fmul: u8                 = 106; // 0x6a
    pub const _dmul: u8                 = 107; // 0x6b
    pub const _idiv: u8                 = 108; // 0x6c
    pub const _ldiv: u8                 = 109; // 0x6d
    pub const _fdiv: u8                 = 110; // 0x6e
    pub const _ddiv: u8                 = 111; // 0x6f
    pub const _irem: u8                 = 112; // 0x70
    pub const _lrem: u8                 = 113; // 0x71
    pub const _frem: u8                 = 114; // 0x72
    pub const _drem: u8                 = 115; // 0x73
    pub const _ineg: u8                 = 116; // 0x74
    pub const _lneg: u8                 = 117; // 0x75
    pub const _fneg: u8                 = 118; // 0x76
    pub const _dneg: u8                 = 119; // 0x77
    pub const _ishl: u8                 = 120; // 0x78
    pub const _lshl: u8                 = 121; // 0x79
    pub const _ishr: u8                 = 122; // 0x7a
    pub const _lshr: u8                 = 123; // 0x7b
    pub const _iushr: u8                = 124; // 0x7c
    pub const _lushr: u8                = 125; // 0x7d
    pub const _iand: u8                 = 126; // 0x7e
    pub const _land: u8                 = 127; // 0x7f
    pub const _ior: u8                  = 128; // 0x80
    pub const _lor: u8                  = 129; // 0x81
    pub const _ixor: u8                 = 130; // 0x82
    pub const _lxor: u8                 = 131; // 0x83
    pub const _iinc: u8                 = 132; // 0x84
    pub const _i2l: u8                  = 133; // 0x85
    pub const _i2f: u8                  = 134; // 0x86
    pub const _i2d: u8                  = 135; // 0x87
    pub const _l2i: u8                  = 136; // 0x88
    pub const _l2f: u8                  = 137; // 0x89
    pub const _l2d: u8                  = 138; // 0x8a
    pub const _f2i: u8                  = 139; // 0x8b
    pub const _f2l: u8                  = 140; // 0x8c
    pub const _f2d: u8                  = 141; // 0x8d
    pub const _d2i: u8                  = 142; // 0x8e
    pub const _d2l: u8                  = 143; // 0x8f
    pub const _d2f: u8                  = 144; // 0x90
    pub const _i2b: u8                  = 145; // 0x91
    pub const _i2c: u8                  = 146; // 0x92
    pub const _i2s: u8                  = 147; // 0x93
    pub const _lcmp: u8                 = 148; // 0x94
    pub const _fcmpl: u8                = 149; // 0x95
    pub const _fcmpg: u8                = 150; // 0x96
    pub const _dcmpl: u8                = 151; // 0x97
    pub const _dcmpg: u8                = 152; // 0x98
    pub const _ifeq: u8                 = 153; // 0x99
    pub const _ifne: u8                 = 154; // 0x9a
    pub const _iflt: u8                 = 155; // 0x9b
    pub const _ifge: u8                 = 156; // 0x9c
    pub const _ifgt: u8                 = 157; // 0x9d
    pub const _ifle: u8                 = 158; // 0x9e
    pub const _if_icmpeq: u8            = 159; // 0x9f
    pub const _if_icmpne: u8            = 160; // 0xa0
    pub const _if_icmplt: u8            = 161; // 0xa1
    pub const _if_icmpge: u8            = 162; // 0xa2
    pub const _if_icmpgt: u8            = 163; // 0xa3
    pub const _if_icmple: u8            = 164; // 0xa4
    pub const _if_acmpeq: u8            = 165; // 0xa5
    pub const _if_acmpne: u8            = 166; // 0xa6
    pub const _goto: u8                 = 167; // 0xa7
    pub const _jsr: u8                  = 168; // 0xa8
    pub const _ret: u8                  = 169; // 0xa9
    pub const _tableswitch: u8          = 170; // 0xaa
    pub const _lookupswitch: u8         = 171; // 0xab
    pub const _ireturn: u8              = 172; // 0xac
    pub const _lreturn: u8              = 173; // 0xad
    pub const _freturn: u8              = 174; // 0xae
    pub const _dreturn: u8              = 175; // 0xaf
    pub const _areturn: u8              = 176; // 0xb0
    pub const _return: u8               = 177; // 0xb1
    pub const _getstatic: u8            = 178; // 0xb2
    pub const _putstatic: u8            = 179; // 0xb3
    pub const _getfield: u8             = 180; // 0xb4
    pub const _putfield: u8             = 181; // 0xb5
    pub const _invokevirtual: u8        = 182; // 0xb6
    pub const _invokespecial: u8        = 183; // 0xb7
    pub const _invokestatic: u8         = 184; // 0xb8
    pub const _invokeinterface: u8      = 185; // 0xb9
    pub const _invokedynamic: u8        = 186; // 0xba
    pub const _new: u8                  = 187; // 0xbb
    pub const _newarray: u8             = 188; // 0xbc
    pub const _anewarray: u8            = 189; // 0xbd
    pub const _arraylength: u8          = 190; // 0xbe
    pub const _athrow: u8               = 191; // 0xbf
    pub const _checkcast: u8            = 192; // 0xc0
    pub const _instanceof: u8           = 193; // 0xc1
    pub const _monitorenter: u8         = 194; // 0xc2
    pub const _monitorexit: u8          = 195; // 0xc3
    pub const _wide: u8                 = 196; // 0xc4
    pub const _multianewarray: u8       = 197; // 0xc5
    pub const _ifnull: u8               = 198; // 0xc6
    pub const _ifnonnull: u8            = 199; // 0xc7
    pub const _goto_w: u8               = 200; // 0xc8
    pub const _jsr_w: u8                = 201; // 0xc9
    pub const _breakpoint: u8           = 202; // 0xca
}
