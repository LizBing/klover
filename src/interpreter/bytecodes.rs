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

pub mod opcode {
    pub const _ILLEGAL: u8              =  u8::MAX;

    // Java bytecodes
    pub const _NOP: u8                  =   0; // 0x00
    pub const _ACONST_NULL: u8          =   1; // 0x01
    pub const _ICONST_M1: u8            =   2; // 0x02
    pub const _ICONST_0: u8             =   3; // 0x03
    pub const _ICONST_1: u8             =   4; // 0x04
    pub const _ICONST_2: u8             =   5; // 0x05
    pub const _ICONST_3: u8             =   6; // 0x06
    pub const _ICONST_4: u8             =   7; // 0x07
    pub const _ICONST_5: u8             =   8; // 0x08
    pub const _LCONST_0: u8             =   9; // 0x09
    pub const _LCONST_1: u8             =  10; // 0x0a
    pub const _FCONST_0: u8             =  11; // 0x0b
    pub const _FCONST_1: u8             =  12; // 0x0c
    pub const _FCONST_2: u8             =  13; // 0x0d
    pub const _DCONST_0: u8             =  14; // 0x0e
    pub const _DCONST_1: u8             =  15; // 0x0f
    pub const _BIPUSH: u8               =  16; // 0x10
    pub const _SIPUSH: u8               =  17; // 0x11
    pub const _LDC: u8                  =  18; // 0x12
    pub const _LDC_W: u8                =  19; // 0x13
    pub const _LDC2_W: u8               =  20; // 0x14
    pub const _ILOAD: u8                =  21; // 0x15
    pub const _LLOAD: u8                =  22; // 0x16
    pub const _FLOAD: u8                =  23; // 0x17
    pub const _DLOAD: u8                =  24; // 0x18
    pub const _ALOAD: u8                =  25; // 0x19
    pub const _ILOAD_0: u8              =  26; // 0x1a
    pub const _ILOAD_1: u8              =  27; // 0x1b
    pub const _ILOAD_2: u8              =  28; // 0x1c
    pub const _ILOAD_3: u8              =  29; // 0x1d
    pub const _LLOAD_0: u8              =  30; // 0x1e
    pub const _LLOAD_1: u8              =  31; // 0x1f
    pub const _LLOAD_2: u8              =  32; // 0x20
    pub const _LLOAD_3: u8              =  33; // 0x21
    pub const _FLOAD_0: u8              =  34; // 0x22
    pub const _FLOAD_1: u8              =  35; // 0x23
    pub const _FLOAD_2: u8              =  36; // 0x24
    pub const _FLOAD_3: u8              =  37; // 0x25
    pub const _DLOAD_0: u8              =  38; // 0x26
    pub const _DLOAD_1: u8              =  39; // 0x27
    pub const _DLOAD_2: u8              =  40; // 0x28
    pub const _DLOAD_3: u8              =  41; // 0x29
    pub const _ALOAD_0: u8              =  42; // 0x2a
    pub const _ALOAD_1: u8              =  43; // 0x2b
    pub const _ALOAD_2: u8              =  44; // 0x2c
    pub const _ALOAD_3: u8              =  45; // 0x2d
    pub const _IALOAD: u8               =  46; // 0x2e
    pub const _LALOAD: u8               =  47; // 0x2f
    pub const _FALOAD: u8               =  48; // 0x30
    pub const _DALOAD: u8               =  49; // 0x31
    pub const _AALOAD: u8               =  50; // 0x32
    pub const _BALOAD: u8               =  51; // 0x33
    pub const _CALOAD: u8               =  52; // 0x34
    pub const _SALOAD: u8               =  53; // 0x35
    pub const _ISTORE: u8               =  54; // 0x36
    pub const _LSTORE: u8               =  55; // 0x37
    pub const _FSTORE: u8               =  56; // 0x38
    pub const _DSTORE: u8               =  57; // 0x39
    pub const _ASTORE: u8               =  58; // 0x3a
    pub const _ISTORE_0: u8             =  59; // 0x3b
    pub const _ISTORE_1: u8             =  60; // 0x3c
    pub const _ISTORE_2: u8             =  61; // 0x3d
    pub const _ISTORE_3: u8             =  62; // 0x3e
    pub const _LSTORE_0: u8             =  63; // 0x3f
    pub const _LSTORE_1: u8             =  64; // 0x40
    pub const _LSTORE_2: u8             =  65; // 0x41
    pub const _LSTORE_3: u8             =  66; // 0x42
    pub const _FSTORE_0: u8             =  67; // 0x43
    pub const _FSTORE_1: u8             =  68; // 0x44
    pub const _FSTORE_2: u8             =  69; // 0x45
    pub const _FSTORE_3: u8             =  70; // 0x46
    pub const _DSTORE_0: u8             =  71; // 0x47
    pub const _DSTORE_1: u8             =  72; // 0x48
    pub const _DSTORE_2: u8             =  73; // 0x49
    pub const _DSTORE_3: u8             =  74; // 0x4a
    pub const _ASTORE_0: u8             =  75; // 0x4b
    pub const _ASTORE_1: u8             =  76; // 0x4c
    pub const _ASTORE_2: u8             =  77; // 0x4d
    pub const _ASTORE_3: u8             =  78; // 0x4e
    pub const _IASTORE: u8              =  79; // 0x4f
    pub const _LASTORE: u8              =  80; // 0x50
    pub const _FASTORE: u8              =  81; // 0x51
    pub const _DASTORE: u8              =  82; // 0x52
    pub const _AASTORE: u8              =  83; // 0x53
    pub const _BASTORE: u8              =  84; // 0x54
    pub const _CASTORE: u8              =  85; // 0x55
    pub const _SASTORE: u8              =  86; // 0x56
    pub const _POP: u8                  =  87; // 0x57
    pub const _POP2: u8                 =  88; // 0x58
    pub const _DUP: u8                  =  89; // 0x59
    pub const _DUP_X1: u8               =  90; // 0x5a
    pub const _DUP_X2: u8               =  91; // 0x5b
    pub const _DUP2: u8                 =  92; // 0x5c
    pub const _DUP2_X1: u8              =  93; // 0x5d
    pub const _DUP2_X2: u8              =  94; // 0x5e
    pub const _SWAP: u8                 =  95; // 0x5f
    pub const _IADD: u8                 =  96; // 0x60
    pub const _LADD: u8                 =  97; // 0x61
    pub const _FADD: u8                 =  98; // 0x62
    pub const _DADD: u8                 =  99; // 0x63
    pub const _ISUB: u8                 = 100; // 0x64
    pub const _LSUB: u8                 = 101; // 0x65
    pub const _FSUB: u8                 = 102; // 0x66
    pub const _DSUB: u8                 = 103; // 0x67
    pub const _IMUL: u8                 = 104; // 0x68
    pub const _LMUL: u8                 = 105; // 0x69
    pub const _FMUL: u8                 = 106; // 0x6a
    pub const _DMUL: u8                 = 107; // 0x6b
    pub const _IDIV: u8                 = 108; // 0x6c
    pub const _LDIV: u8                 = 109; // 0x6d
    pub const _FDIV: u8                 = 110; // 0x6e
    pub const _DDIV: u8                 = 111; // 0x6f
    pub const _IREM: u8                 = 112; // 0x70
    pub const _LREM: u8                 = 113; // 0x71
    pub const _FREM: u8                 = 114; // 0x72
    pub const _DREM: u8                 = 115; // 0x73
    pub const _INEG: u8                 = 116; // 0x74
    pub const _LNEG: u8                 = 117; // 0x75
    pub const _FNEG: u8                 = 118; // 0x76
    pub const _DNEG: u8                 = 119; // 0x77
    pub const _ISHL: u8                 = 120; // 0x78
    pub const _LSHL: u8                 = 121; // 0x79
    pub const _ISHR: u8                 = 122; // 0x7a
    pub const _LSHR: u8                 = 123; // 0x7b
    pub const _IUSHR: u8                = 124; // 0x7c
    pub const _LUSHR: u8                = 125; // 0x7d
    pub const _IAND: u8                 = 126; // 0x7e
    pub const _LAND: u8                 = 127; // 0x7f
    pub const _IOR: u8                  = 128; // 0x80
    pub const _LOR: u8                  = 129; // 0x81
    pub const _IXOR: u8                 = 130; // 0x82
    pub const _LXOR: u8                 = 131; // 0x83
    pub const _IINC: u8                 = 132; // 0x84
    pub const _I2L: u8                  = 133; // 0x85
    pub const _I2F: u8                  = 134; // 0x86
    pub const _I2D: u8                  = 135; // 0x87
    pub const _L2I: u8                  = 136; // 0x88
    pub const _L2F: u8                  = 137; // 0x89
    pub const _L2D: u8                  = 138; // 0x8a
    pub const _F2I: u8                  = 139; // 0x8b
    pub const _F2L: u8                  = 140; // 0x8c
    pub const _F2D: u8                  = 141; // 0x8d
    pub const _D2I: u8                  = 142; // 0x8e
    pub const _D2L: u8                  = 143; // 0x8f
    pub const _D2F: u8                  = 144; // 0x90
    pub const _I2B: u8                  = 145; // 0x91
    pub const _I2C: u8                  = 146; // 0x92
    pub const _I2S: u8                  = 147; // 0x93
    pub const _LCMP: u8                 = 148; // 0x94
    pub const _FCMPL: u8                = 149; // 0x95
    pub const _FCMPG: u8                = 150; // 0x96
    pub const _DCMPL: u8                = 151; // 0x97
    pub const _DCMPG: u8                = 152; // 0x98
    pub const _IFEQ: u8                 = 153; // 0x99
    pub const _IFNE: u8                 = 154; // 0x9a
    pub const _IFLT: u8                 = 155; // 0x9b
    pub const _IFGE: u8                 = 156; // 0x9c
    pub const _IFGT: u8                 = 157; // 0x9d
    pub const _IFLE: u8                 = 158; // 0x9e
    pub const _IF_ICMPEQ: u8            = 159; // 0x9f
    pub const _IF_ICMPNE: u8            = 160; // 0xa0
    pub const _IF_ICMPLT: u8            = 161; // 0xa1
    pub const _IF_ICMPGE: u8            = 162; // 0xa2
    pub const _IF_ICMPGT: u8            = 163; // 0xa3
    pub const _IF_ICMPLE: u8            = 164; // 0xa4
    pub const _IF_ACMPEQ: u8            = 165; // 0xa5
    pub const _IF_ACMPNE: u8            = 166; // 0xa6
    pub const _GOTO: u8                 = 167; // 0xa7
    pub const _JSR: u8                  = 168; // 0xa8
    pub const _RET: u8                  = 169; // 0xa9
    pub const _TABLESWITCH: u8          = 170; // 0xaa
    pub const _LOOKUPSWITCH: u8         = 171; // 0xab
    pub const _IRETURN: u8              = 172; // 0xac
    pub const _LRETURN: u8              = 173; // 0xad
    pub const _FRETURN: u8              = 174; // 0xae
    pub const _DRETURN: u8              = 175; // 0xaf
    pub const _ARETURN: u8              = 176; // 0xb0
    pub const _RETURN: u8               = 177; // 0xb1
    pub const _GETSTATIC: u8            = 178; // 0xb2
    pub const _PUTSTATIC: u8            = 179; // 0xb3
    pub const _GETFIELD: u8             = 180; // 0xb4
    pub const _PUTFIELD: u8             = 181; // 0xb5
    pub const _INVOKEVIRTUAL: u8        = 182; // 0xb6
    pub const _INVOKESPECIAL: u8        = 183; // 0xb7
    pub const _INVOKESTATIC: u8         = 184; // 0xb8
    pub const _INVOKEINTERFACE: u8      = 185; // 0xb9
    pub const _INVOKEDYNAMIC: u8        = 186; // 0xba
    pub const _NEW: u8                  = 187; // 0xbb
    pub const _NEWARRAY: u8             = 188; // 0xbc
    pub const _ANEWARRAY: u8            = 189; // 0xbd
    pub const _ARRAYLENGTH: u8          = 190; // 0xbe
    pub const _ATHROW: u8               = 191; // 0xbf
    pub const _CHECKCAST: u8            = 192; // 0xc0
    pub const _INSTANCEOF: u8           = 193; // 0xc1
    pub const _MONITORENTER: u8         = 194; // 0xc2
    pub const _MONITOREXIT: u8          = 195; // 0xc3
    pub const _WIDE: u8                 = 196; // 0xc4
    pub const _MULTIANEWARRAY: u8       = 197; // 0xc5
    pub const _IFNULL: u8               = 198; // 0xc6
    pub const _IFNONNULL: u8            = 199; // 0xc7
    pub const _GOTO_W: u8               = 200; // 0xc8
    pub const _JSR_W: u8                = 201; // 0xc9
    pub const _BREAKPOINT: u8           = 202; // 0xca

    pub const NUM_OF_JAVA_CODES: u8     = 203;
}
