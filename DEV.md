# Klover 开发文档

> 一个用 Rust + C 混合实现的 JVM，设计风格借鉴 HotSpot。
> 最后更新：阶段 4 对象指令基本完成，准备进入异常处理（`athrow`）。

---

## 一、项目结构

```
myjvm/
├── core/                          # C 层（内存与 GC 基础设施）
│   ├── gc/
│   │   ├── gc_heap.c/.h           # Java 堆 bump-pointer 分配（含 payload 清零）
│   │   ├── oop_storage.c/.h       # OopStorage（native 端持有 Java 对象引用）
│   │   ├── oop_closure.h          # GC 遍历回调接口
│   │   └── gc.c/.h                # gc_init 入口
│   ├── memory/
│   │   ├── virt_space.c/.h        # 虚拟内存保留/提交（mmap）
│   │   └── comp_space_defs.h      # 压缩指针常量（METASPACE_BASE / GCHEAP_BASE）
│   ├── metaspace/
│   │   └── metaspace.c/.h         # Metaspace chunk 分配（bump + free stack）
│   ├── obj_model/
│   │   ├── obj_desc.h             # ObjDesc（markword + payload）
│   │   ├── markword.h             # markword 位域布局
│   │   ├── oop_hierarchy.h        # objptr_t / nobjptr_t 类型别名
│   │   └── obj_layout.h           # ObjLayout 镜像（C 端 GC 用）
│   ├── tests/                     # C 层单元测试
│   └── utils/global_defs.h
├── rust/                          # Rust 层（JVM 核心）
│   ├── src/
│   │   ├── lib.rs                 # 模块注册
│   │   ├── class_parser/          # .class 文件解析
│   │   │   ├── class_file.rs      # ClassFile 结构
│   │   │   ├── class_reader.rs    # 字节流读取器
│   │   │   ├── cp_info.rs         # 常量池条目（解析态）
│   │   │   ├── field_info.rs      # 字段元信息（解析态）
│   │   │   ├── method_info.rs     # 方法元信息（解析态）
│   │   │   ├── attr_info.rs       # 属性解析（Code / ConstantValue）
│   │   │   └── parse_error.rs
│   │   ├── class_loader/          # 类加载与运行时常量池解析
│   │   │   ├── bootstrap_cld.rs   # Bootstrap ClassLoader（含 prim/array klass）
│   │   │   ├── cld.rs             # ClassLoaderData（用户 CLD，MVP 委托 bootstrap）
│   │   │   ├── cld_map.rs         # 全局 CLD 链表
│   │   │   ├── class_path.rs      # class 文件读取
│   │   │   ├── ms_api.rs          # ★ MSAllocator / MSBox / MSRef / 压缩指针
│   │   │   ├── resolve.rs         # ★ CP 引用解析（method/field/class）
│   │   │   └── load_error.rs
│   │   ├── gc_binding/            # ★ Rust↔C 的 GC/堆 FFI
│   │   │   ├── gc_binding.rs      # alloc_object / init_heap / klover_obj_layout_of
│   │   │   ├── oop_codec.rs       # encode_oop/decode_oop/klass_from_markword
│   │   │   └── obj_layout.rs      # ObjLayout 结构（#[repr(C)]）
│   │   ├── interpreter/           # 字节码解释器
│   │   │   ├── interpreter.rs     # Interpreter / 栈帧 / invoke_resolved
│   │   │   └── instructions.rs    # ★ 字节码分发表（188 条指令已注册）
│   │   ├── oops/                  # 运行时对象模型
│   │   │   ├── klass.rs           # Klass enum（Normal/Primitive/Array）
│   │   │   ├── normal_klass.rs    # ★ NormalKlass（含 ObjLayout / Fields）
│   │   │   ├── array_klass.rs     # ArrayKlass（含 element_size / element_is_ref）
│   │   │   ├── prim_klass.rs      # PrimKlass
│   │   │   ├── method.rs          # Method
│   │   │   ├── field.rs           # Field（acc_flags / name / desc / offs）
│   │   │   ├── desc.rs            # ★ FieldDesc（含 raw）/ MethodDesc / FieldElemType
│   │   │   ├── cp_entry.rs        # ★ 运行时常量池条目（含 lazy resolution）
│   │   │   ├── attr.rs            # CodeAttr / ExceptionTableEntry / ConstantValueAttr
│   │   │   ├── acc_flags.rs       # ACC_* 位标志
│   │   │   ├── oop_handle.rs      # OOPHandle / ObjDesc / ObjPtr / NObjPtr
│   │   │   ├── symbol_table.rs    # SymbolTable（引用计数 intern）
│   │   │   └── resolve_error.rs
│   │   └── runtime/arguments.rs   # 全局启动参数（bs_class_path）
│   └── tests/test_interpreter.rs  # ★ 端到端测试（63 个，全部通过）
├── test_data/classes/             # 测试用 Java 类
│   ├── ObjTest.java/.class        # 对象/字段/数组/类型检查测试
│   ├── Adder.java/.class          # 接口（invokeinterface 测试）
│   ├── IntAdder.java/.class       # 接口实现类
│   ├── Arith / ControlFlow / Wide / SimpleAddition  # 阶段 1-3 测试
│   ├── PrimKlass / HelloKlover    # 暂未使用（HelloKlover 需 native）
│   └── java/lang/Object.class     # 根类（<init> 为空）
├── java/java.base/                # 空（未来放 JDK 核心类）
└── CMakeLists.txt                 # 构建 klover-core 共享库
```

---

## 二、核心设计决策

### 1. 双地址空间压缩指针

- **Metaspace**（`METASPACE_BASE = 1<<43`）：存放 Klass / Method / Field / CPEntry 等元数据
- **Java 堆**（`GCHEAP_BASE = 1<<44`）：存放 ObjDesc（Java 对象）
- 两者各自独立的 32 位 narrow ptr，8 字节对齐，覆盖 32GB
- **narrow ptr = 0 是 NULL 哨兵**：`gcheap_init` 跳过第一个 word，首次分配从 offset 8 开始

### 2. 引用表示

- **栈槽**（`StackSlot = i32`）：直接存 narrow ptr（4 字节）
- **对象体内引用字段**：narrow ptr（4 字节）
- **markword 里的 klass ptr**：以 `METASPACE_BASE` 为基准的 narrow ptr，编码在高 33 位
- **统一访问**：所有 metaspace 引用走 `MSRef`（`encode`/`decode`），`ms_comp_ptr_*` 不出 `ms_api.rs`

### 3. 对象内存布局

#### 普通对象

```
对象起始 (NObjPtr 解码后指向这里)
┌────────────────────────────────────┐  offset 0
│  markword (8B)                     │  含 narrow klass ptr
├────────────────────────────────────┤  offset 8  ← super 部分起点
│  super 部分                        │
│  ┌──────────────────────────────┐  │
│  │  oop 区 (ptr_count × 4B)     │  │  GC 扫描起点
│  │  ...                          │  │
│  ├─ padding 到 8B（若后跟 8B 字段）┤  │
│  │  8B / 4B / 2B / 1B 字段区     │  │
│  └──────────────────────────────┘  │  ← super.byte_size
├────────────────────────────────────┤  ← 本类部分起点 = super.byte_size
│  本类部分（同样布局）               │
└────────────────────────────────────┘  ← byte_size（8B 对齐，累计）
```

- `ObjLayout { super_layout, byte_size（累计）, ptr_count（本层）}`
- GC 遍历：每层 oop 区起点 = `super_layout.byte_size`（首层 = 8）
- 字段 offset **从对象头起点算**（含 markword），访问时 `base = obj_ptr`，不用 `payload.as_ptr()`

#### 数组对象

```
┌────────────────────────────────────┐  offset 0
│  markword (8B)                     │
├────────────────────────────────────┤  offset 8
│  length (i32, 4B)                  │
├─ padding 到 8B ─────────────────────┤  offset 12-15
├────────────────────────────────────┤  offset 16 (ARRAY_DATA_OFFSET)
│  element[0] / [1] / ...            │
└────────────────────────────────────┘
```

- `ArrayKlass` 不走 `ObjLayout`（`klover_obj_layout_of` 对数组返回 null）
- 常量：`ARRAY_HEADER_BYTES=16`, `ARRAY_LENGTH_OFFSET=8`, `ARRAY_DATA_OFFSET=16`

### 4. static 字段

- 独立存储（`NormalKlass.fields().static_storage`），**不进 ObjLayout**
- 在 `NormalKlass::build` 时立即分配 + 填 ConstantValue
- offset 相对 `static_storage` 起点

### 5. 字段布局构建（两阶段）

- **build 阶段**（`RawFields::build`）：解析字段元信息；static storage 立即分配 + 填 ConstantValue；instance 字段按 bucket 分组但**不算 offset**
- **set_super 阶段**（`Fields::finalize`）：叠加父类偏移，计算 instance offset + byte_size + ptr_count，填 ObjLayout

### 6. 方法调用

- `invoke_resolved`：帧建立/run_loop 的唯一实现点，static/instance 都走它
- **invokevirtual/invokeinterface**：MVP 用线性查找（沿继承链 `find_method`），不做 slot-based vtable
- **invokespecial**：用 `resolve_method_ref`（沿继承链向上找第一个匹配）
- 实参收集：`instance_arg_slot_count` = `arg_slot_count + 1`（含 this）

### 7. CP 引用解析

- **lazy + OnceLock 缓存**：`CPRefEntry.resolved` / `ClassCPEntry.resolved`
- **按 caller 的 ClassLoader 分发**：`load_class_by_caller(caller, name)`
  - `caller.cld == None` → `BootstrapCLD::find_class`
  - `caller.cld == Some` → `ClassLoaderData::load_class`（MVP 委托 bootstrap）

---

## 三、当前进度

### 已完成的阶段

| 阶段 | 内容 | 测试数 |
|------|------|--------|
| 阶段 1 | 最小闭环（常量加载 + 局部变量 + iadd + ireturn） | 4 |
| 阶段 2 | int/long 算术与位运算 | 17 |
| 阶段 3 | 浮点 + 类型转换 + 比较 + 控制流 + switch | 30 |
| 阶段 4 | 对象创建 + 字段访问 + 方法调用 + 数组 + 类型检查 | 12 |
| **总计** | **188 条指令已注册** | **63 个测试全部通过** |

### 阶段 4 已实现的指令

| 类别 | 指令 |
|------|------|
| 对象创建 | `new` (0xbb) |
| 字段访问 | `getfield`/`putfield`/`getstatic`/`putstatic` (0xb2-0xb5) |
| 方法调用 | `invokevirtual` (0xb6), `invokespecial` (0xb7), `invokestatic` (0xb8), `invokeinterface` (0xb9) |
| 数组 | `newarray` (0xbc), `anewarray` (0xbd), `arraylength` (0xbe), `*aload`/`*astore` (0x2e-0x35, 0x4f-0x56) |
| 类型检查 | `instanceof` (0xc1), `checkcast` (0xc0) |
| 栈操作 | `pop`/`pop2`/`dup`/`dup_x1`/`dup_x2`/`dup2`/`dup2_x1`/`dup2_x2`/`swap` (0x57-0x5f) |

### 测试覆盖的场景

- `ObjTest`：对象分配 + 构造器 + 字段读写 + 虚方法 + 静态方法 + 数组（int/ref/byte/char/long）+ instanceof + checkcast
- `IntAdder`/`Adder`：接口方法派发
- `Arith`/`ControlFlow`/`Wide`/`SimpleAddition`：纯算术 + 控制流

---

## 四、TODO 清单（按优先级）

### 🔴 阻塞下一阶段（athrow / 异常处理）

| TODO | 位置 | 说明 |
|------|------|------|
| `Flow::Throw` 当前是 `Throw(ObjPtr)` 裸指针 | `interpreter.rs:24` | 应改为 `Throw(u32)`（narrow ptr），与栈槽语义统一 |
| `ExceptionTableEntry.catch_type` 解析 0 会 panic | `attr.rs:21` | catch_type==0 是 catch all（finally）；需改成 `Option<MSRef<ClassCPEntry>>` |
| `athrow` (0xbf) 未实现 | dispatch 表 | 下一步要做的 |

### 🟡 异常处理相关（阶段 5）

| TODO | 位置 | 说明 |
|------|------|------|
| `pop_array_index` 越界 panic | `instructions.rs:1919` | 改成抛 `ArrayIndexOutOfBoundsException` |
| `checkcast` 失败 panic | `instructions.rs:2135` | 改成抛 `ClassCastException` |
| `getfield`/`putfield` 等的 null 检查 panic | 多处 `assert!(nptr != 0)` | 改成抛 `NullPointerException` |
| 栈底未捕获异常 | 待实现 | MVP：打印异常类名 + 栈轨迹，然后终止 |

### 🟡 类加载相关

| TODO | 位置 | 说明 |
|------|------|------|
| `ClassLoaderData::load_class` 是 bootstrap 委托 | `cld.rs:143` | 真正双亲委派需 native `ClassLoader.loadClass` |
| `<clinit>` 未实现 | 类加载流程 | static 字段访问/静态方法调用前应触发 `<clinit>` |
| `define_class` 拒绝无父类 | `cld.rs:107` | Object 类特判（super_index==0 合法） |

### 🟡 指令补全

| TODO | 位置 | 说明 |
|------|------|------|
| `ldc`/`ldc_w` 只支持 int/float | `instructions.rs:505,517` | 需加 String / Class / MethodType / MethodHandle 支持 |
| `ldc2_w` 只支持 long/double | `instructions.rs:529` | 已完整 |
| `multianewarray` (0xc5) 跳过 | dispatch 表 | 多维数组创建，MVP 不需要 |
| `areturn` (0xb0) 未注册 | dispatch 表 | 引用返回（`ReturnValue::Ref(u32)` 已就绪，只差 handler 注册） |
| `monitorenter`/`monitorexit` (0xc2/0xc3) | dispatch 表 | 多线程同步，远期 |
| `jsr`/`ret`/`jsr_w` | dispatch 表 | 已废弃，不需要 |

### 🟢 接口/类型系统增强

| TODO | 位置 | 说明 |
|------|------|------|
| default method 不支持 | `invoke_dispatched` 注释 | 需遍历 implements 列表查找接口默认方法 |
| 接口类型的 instanceof/checkcast 不支持 | `is_instance_of` | 需遍历 implements 列表 |
| 数组对象的 instanceof/checkcast 不支持 | `is_instance_of` | 数组类型关系判断 |

### 🟢 GC 相关

| TODO | 位置 | 说明 |
|------|------|------|
| `gcheap_alloc` 是纯 bump 分配，无回收 | `core/gc/gc_heap.c` | 需实现标记-清除/标记-复制 |
| 解释器栈无法区分引用槽 | `StackSlot = i32` | GC 需 oop map 告知哪些槽是引用 |
| C 端 GC 数组扫描未实现 | `obj_layout.h` | `klover_obj_layout_of` 返回 null 时需走数组专用路径 |
| static 引用字段不参与 GC 扫描 | `Fields` | 需补 static oop map |

### 🟢 Native / 类库

| TODO | 位置 | 说明 |
|------|------|------|
| `HelloKlover` 无法运行 | test_data | 需 `System.out.println` 的 native 实现或 intrinsic |
| String 池未实现 | - | `ldc String` 需要 intern String 池 |
| `java/java.base/` 空 | 目录 | 需要最小 JDK 核心类库 |

### 🟢 代码质量

| TODO | 位置 | 说明 |
|------|------|------|
| 24 个 warning（主要是 edition 2024 unsafe block） | 全局 | `unsafe fn` 内部裸指针操作需包 `unsafe {}` 块 |
| 测试需 `--test-threads=1` | - | 并行测试时终端输出/状态竞争（测试本身不依赖顺序） |

---

## 五、下次开工指引

### 立即下一步：`athrow`（异常抛出）

讨论中已确定的设计（尚未实现）：

1. **`Flow::Throw` 改为 `Throw(u32)`**（narrow ptr）
2. **`ExceptionTableEntry.catch_type` 改为 `Option<MSRef<ClassCPEntry>>`**（None = catch all）
3. **`athrow` handler**：
   - pop 异常对象引用
   - 在当前帧的 exception_table 查找 handler（throw_pc = athrow 指令的 bci）
   - 找到：清栈、push 异常、跳 handler_pc、`Flow::Continue`
   - 找不到：`Flow::Throw(nptr)`，让 `invoke_resolved` 处理
4. **`invoke_resolved` 栈展开**：
   - 收到 `Flow::Throw` 后，在调用者帧的 exception_table 继续查找
   - 调用者帧的 pc 已是 invoke 之后的位置（JVM 规范要求的）
   - 找到：清栈、push 异常、改 pc、继续 run_loop
   - 找不到：继续向上 `Flow::Throw`
5. **栈底未捕获**：panic + 打印异常类名

### 中期目标：让 `HelloKlover` 跑起来

需要：
- String 池（`ldc String` 支持）
- `System.out.println` 的 intrinsic 或 native
- `java.lang.String` / `System` / `PrintStream` 的最小实现

### 远期：GC

- 标记-清除（最简单）
- 需要先解决"解释器栈的 oop map"问题

---

## 六、构建与测试

```bash
# 构建 C 层
cd build && cmake --build .

# 构建 Rust 层
cd rust && cargo build

# 跑全部测试（注意：必须 --test-threads=1，否则终端卡住）
cd rust && cargo test --test test_interpreter -- --test-threads=1

# 单个测试
cd rust && cargo test --test test_interpreter <test_name>

# 重新编译测试用 Java 类
cd test_data/classes && javac ObjTest.java
```

### 编译测试 Java 类的注意事项

- 测试用 `.java` 改动后必须重新 `javac`
- `checkcast` 测试需要对象先以 `Object` 类型存在再 cast（否则 javac 优化掉 checkcast）
- `invokeinterface` 测试需要接口和实现类分开文件
