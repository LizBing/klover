# Class parser API

The parser accepts Java 8 class files (major version 52). Its pipeline is
`decode -> validate -> ClassImage`. The validated `ClassImage` owns its data and
has no borrow of the input bytes.

## Attributes

Known Java 8 attributes are decoded into owner-specific structs:

- `image.attributes()` returns `ClassAttributes`.
- `field.attributes` is `FieldAttributes`.
- `method.attributes` is `MethodAttributes`; its `code` field holds `CodeData`.
- `code.attributes` is `CodeAttributes`.

These cover all 23 predefined Java 8 attributes, including both annotation
visibilities, parameter and type annotations, annotation defaults, bootstrap
methods, stack maps, signatures, enclosing/inner classes, and debug tables.
`Option<Vec<T>>` distinguishes an absent attribute from a present empty one.
Line-number and local-variable tables retain separate occurrences, in file
order. Unknown attributes retain their lossless `JavaText` name and owned body
bytes, including repeated occurrences. Names outside their predefined owner
location are also preserved as opaque attributes.

The original pool remains available through `image.constant(index)` and
`image.constant_pool()`. Lookup returns `None` for index zero, indices outside
the pool, and the unusable second slots of long/double constants. Opaque
attribute bodies may contain indices into this original pool; interpret them
while the image remains available. Attribute order across different known
kinds and byte-for-byte round trips are not part of this API.

## Constant identity and runtime construction

One per-class builder creates canonical symbolic entries. Every consumer of a
particular original CP index receives a clone of the same `Arc`. This applies
across methods, field constants, annotations, exception handlers, stack maps,
and bootstrap arguments. Distinct CP indices are not merged by equal contents.

`ClassRef` is shared by class instructions, class literals, member owners,
exception references, stack-map object types, and class metadata. A class
literal uses `LoadableConstant::Class(Arc<ClassRef>)`. A method-handle literal
uses `LoadableConstant::MethodHandle(Arc<MethodHandleConstant>)`; bootstrap
records hold the same inner handle. Member references retain their field,
method, or interface-method kind. Bootstrap-table rows have their own shared
records; `InvokeDynamicConstant.bootstrap` points directly to its record.

A runtime builder can use a temporary map such as
`HashMap<*const ClassMemberRef, &'cld RuntimeMethodRef>`, keyed by
`Arc::as_ptr()`. Use the inner `ClassRef` or `MethodHandleConstant` Arc as the
key when that runtime object is shared across different parsed wrappers. Keep
the image alive until all instructions and attributes have been converted,
then discard the temporary maps. Runtime objects must own or intern their
symbolic data and outlive all runtime references to them.

The original `Constant` objects are a different representation; their addresses
are not keys for the published canonical Arcs. The parser does not allocate
metaspace objects or resolve classes, methods, handles, or strings at runtime.
In particular, shared invokedynamic descriptions do not imply shared mutable
call-site linkage state: that state belongs to an instruction occurrence.

## Validation boundary

Recognized attributes are checked for framing, exact body consumption,
multiplicity, CP kinds, constant/descriptor compatibility, and local structural
constraints. Code metadata uses original bytecode offsets. Ranges and frame
offsets are checked against instruction boundaries; stack-map uninitialized
values must identify `new`. Frame encoding, local-width changes, and stack/local
capacities are checked, using the method descriptor for initial local widths.

This does not implement a complete bytecode verifier. Stack-map types are not
checked against instruction data flow or the class hierarchy. Generic
signatures are preserved as text rather than parsed and type-checked; indices
into generic parameter/bound lists and generic type paths are not semantically
validated. Annotation values are structurally decoded without loading their
annotation classes. Recursive annotation parsing has a depth limit of 128
parser steps to avoid exhausting the host stack on hostile input.

Exposing metadata does not add execution support for exceptions, field
initialization, invokedynamic, reflection, or debugging. Runtime construction
chooses which metadata it needs to retain.

The format reference is [JVMS 8, section 4.7](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.7).
