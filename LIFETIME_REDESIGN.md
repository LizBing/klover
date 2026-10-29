# Lifetime system redesign

Design for review, 2026-10-29. No implementation approved or included.

This design is based on the current source and working-tree edits. The earlier
METADATA_LIFETIMES.md proposal is excluded from the design basis at the user's
request. Confirmed requirement: support future class-loader unloading, but
implement actual unloading later. That clarification is not implementation approval.

## 1. The central decision

Let ClassLoaderData own its resources, but stop representing the persistent
metadata graph with Rust references. Use ordinary ownership inside each record,
typed IDs between records, and short Rust borrows when accessing them.

Three different questions need different answers:

| Question | Answer |
| --- | --- |
| Who releases this allocation and its resources? | A Rust owner: VM, CLD, class, frame, or temporary builder |
| What keeps metadata usable while something needs it? | Initially the VM retains every published CLD; explicit metadata roots prepare for later unloading |
| How long may this particular reference be used? | A Rust borrow of its owner or access guard |

An ID identifies something; it does not retain it. A root retains something;
it does not grant unrestricted mutation. A borrow permits access; it does not
decide Java reachability.

Keep normal Rust lifetimes for temporary views, parser inputs, and VM access.
Remove the recursive `'cld` / `'klass` / `'method` parameters from published
metadata records. This is a change to representation, not a replacement of
lifetime parameters with unchecked raw pointers.

## 2. Problems in the current code

These findings distinguish current defects from problems that would appear only
after adding collection. Paths refer to this repository.

| Finding | Evidence | Consequence |
| --- | --- | --- |
| Unsynchronized initialization is exposed as thread safe | `class_loader/class_loader_data.rs:13,23`: unsafe Sync plus std::cell::OnceCell | Safe callers can concurrently initialize/read a cell that is not synchronized. Declaring Sync does not add synchronization. |
| One lifetime represents both storage and a short borrow | `engines/invocation.rs:12-24`: `&'metadata NormalKlass<'metadata>` | Local classes cannot be borrowed for short invocations once the inner type is invariant. Test compilation currently fails. |
| CLD contains a reference into itself | `class_loader/class_loader_data.rs:8-24`: storage borrows msa | Two-step initialization couples the value to a long self-borrow and makes ordinary owned construction/return difficult. Safe Rust restrictions are not, by themselves, a dangling-pointer defect. |
| Publishing leaks resource ownership | `runtime/ms_api.rs:268-279`, `class_loader/bs_cld.rs:89`, `code/code.rs:29`, `oops/tmp_constant_pool.rs:22` | Releasing chunks does not run destructors of leaked classes, instructions, or constant records. Heap allocations and symbol ownership can survive arena reclamation. |
| Symbol clone/drop accounting is inconsistent | `oops/symbol_table.rs:104-115` | Derived Clone copies the pointer without incrementing ref_cnt; each Drop decrements. Counts can reach zero while handles exist or wrap. Symbols currently leak, masking the future use-after-free risk. |
| Duplicate class publication replaces the old entry | `class_loader/bs_cld.rs:89-92` | `insert` installs the new class before reporting a duplicate. Existing users and future lookups can observe different metadata for the same class. |
| Resolution context is supplied on each call | `oops/constant_entries.rs:92-123` | A single cached result has no structural association with its referring class/loader. Filling this cache under different caller-supplied loaders could return the wrong class's method. The function is currently an empty draft. |
| Frame completion does not release the frame | `engines/exec_dispatcher.rs:26-28` | Return reports completion without popping the frame or terminating the thread. Future frame roots would retain loaders indefinitely unless lifecycle handling changes. |
| Heap pointer copies are not registered roots | `Invocation::args`, `Slot::Ref`, `MethodReturn`, `InvocationBrief::args`, `ExecErrorKind::ArgumentTypeMismatch` | Once GC exists, copied object addresses in arguments, results, or errors could become dangling or stale. Metadata protection does not protect these objects. |
| OOP handles do not deregister on drop | `class_loader/oop_storage.rs:16-18,42-60` | Once populated and traced, temporary strong slots could retain objects until the entire storage dies. Storage lifetime currently protects the slot, not an implemented object-liveness protocol. |
| Metaspace initialization has an early-error resource gap | `runtime/metaspace.rs:67-75` | If initial expansion fails after reservation, the raw VirtSpace is returned from without destruction. The destructor-owning Metaspace has not yet been constructed. |

All Rust paths above are under `rust/src/`.

The immediate compiler chain is:

```text
OnceCell<OOPStorage<'cld>> permits interior mutation
  -> ClassLoaderData<'cld> is invariant in 'cld
  -> NormalKlass<'cld>, which refers to it, is also invariant
  -> &'metadata NormalKlass<'metadata> ties a short outer borrow
     to an inner lifetime Rust cannot shorten
  -> local Invocation construction fails
```

Invariant means Rust cannot substitute a shorter lifetime inside that type.
This follows the rules for interior mutability described in the
[Rustonomicon](https://doc.rust-lang.org/nomicon/subtyping.html).
Replacing OnceCell with OnceLock would fix this cell's synchronization issue,
but would not remove the self-reference or this kind of lifetime coupling.
[OnceCell documentation](https://doc.rust-lang.org/std/cell/struct.OnceCell.html)
identifies OnceLock as its synchronized counterpart.

As a limited repair, a borrowed Invocation could distinguish `'borrow` from
`'storage`: `&'borrow NormalKlass<'storage>`. That is useful for a temporary
bridge, but it is not the proposed permanent representation of frames or
cross-loader metadata links. Renaming lifetime parameters does not establish
ownership or loader liveness.

## 3. Ownership from the VM down

```text
VM metadata service
  owns CLD registry
    owns ClassLoaderData
      owns metaspace arena
      owns class allocation records
        each owns one Klass record at a stable metaspace address
          owns methods, code, descriptors, runtime constants, and tables
      owns heap-reference slots used by its metadata
      owns loader-local lookup and dependency information

JavaThread owns frames
  each frame owns an executing-method root and its execution state

VM heap owns Java objects
  roots and object edges determine their eventual collection
```

The CLD registry owns allocations; later it must not automatically make every
entry a GC root. Owning storage and deciding reachability are separate jobs.
The bootstrap CLD uses the same interface with a permanent-retention policy.
Eliminate `Option<&ClassLoaderData>` as the bootstrap identity convention;
represent bootstrap with an ordinary, distinguished CldId.

For the first implementation, keep Klass records in metaspace because C object
headers already encode their addresses relative to METASPACE_BASE. Let the
class own subordinate data through `Box<[Method]>`, `Box<[Instruction]>`, and
other ordinary owned containers. Methods own their descriptors and code.
Hierarchy links and constant-pool references use IDs.

This costs more allocations and potentially worse locality than placing every
record in an arena. It buys much simpler destruction and removes most lifetime
parameters without introducing a generic unsafe arena graph. Measure before
replacing these boxes with arena-owned arrays. Moving Klass itself to a normal
Box would require changing the compressed-class-pointer layout, so it is not
part of this proposal.

The private class allocation owner records the address and responsibility to
destroy that Klass. It does not contain a Rust reference to the sibling arena
field. CLD teardown destroys these records and their owned children before
returning arena chunks. Raw pointers remain inside this small storage layer.
The public API cannot directly free a published class or manufacture its owner.

Avoid arbitrary cleanup callbacks: class destructors release their own resources
and never follow another class's IDs. This gives one destruction path per owned
value and allows unreachable loader cycles to be torn down in any order later.

## 4. IDs, roots, and access

Illustrative shapes, not compiled API definitions:

```rust
struct CldId { slot: u32, generation: u64 }
struct KlassId { cld: CldId, index: u32 }
struct MethodId { klass: KlassId, index: u32 }

struct Klass {
    defining_loader: CldId,
    super_class: Option<KlassId>,
    interfaces: Box<[KlassId]>,
    methods: Box<[Method]>,
    constants: Box<[RuntimeConstant]>,
    // Names, flags, and other owned data.
}

struct RootedMethod<'vm> {
    root: MetadataRoot<'vm>,
    id: MethodId,
}

impl VmMetadata {
    fn root_method(&self, id: MethodId)
        -> Result<RootedMethod<'_>, InvalidMetadata>;
}

impl RootedMethod<'_> {
    fn method(&self) -> &Method;
}
```

Fields and constructors are private. IDs have no Deref. Validate CLD generation,
class publication, and member bounds before touching metadata. Within a live
CLD, published class/member indices are append-only and are never repurposed.
If individual deletion or redefinition is added, that assumption needs revisiting.
Retire an exhausted generation rather than wrapping it. Handles belong to one
VM; preserve today's single-VM restriction, and require VM identity checking or
type branding before allowing independently created VMs to exchange IDs.

A root is an explicit registration retaining the target's defining CLD. Root
acquisition validates and registers under the same synchronization boundary.
Cloning a root registers another retention; dropping it releases that retention.
Copying an ID does neither. An immutable root cannot be retargeted while borrowed.
The root may live through a budget pause; a reference returned from it cannot
outlive it. The remaining `'vm` bounds the VM service, not the Java class graph.

Initially, the registry retains every published CLD even when its root count is
zero. Root accounting establishes the future contract but must not trigger
reclamation yet: a zero count says nothing about live Java objects or dependencies.

Published metadata is immutable except for explicitly synchronized caches and
state. Stable addresses and roots do not permit mutation through an existing
shared reference. Safe class/field/method accessors must preserve that rule.

## 5. Construction, loading, and constant resolution

Build metadata privately with ordinary owning builders. Initialize all fallible
parts before publication. Temporary roots protect already published dependencies.
Failed construction drops initialized resources; a partially initialized array
must track its initialized prefix, or use Vec during construction instead.
Arena space consumed by a failed class can wait for CLD teardown, but resource
destructors and temporary roots must still run.

Publish through a per-loader/name loading entry with loading, published, and
failed states. A competing load must not replace an existing definition. Do not
hold the registry's global lock while reading files, recursively loading classes,
waiting for another load, or running Java. Record the loading owner so recursive
requests do not blindly wait on themselves. Full loader constraints, circularity
checks, and Java errors remain separately scoped semantic work.

Give each referring class one runtime constant pool. Instructions store a
constant index rather than an independently owned MethodRef or a borrowed pointer.
That class fixes the resolution context; callers do not supply a loader afresh.
Java class identity includes the defining loader, and resolution uses the
referring class's defining loader; see
[JVMS 8, chapter 5](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-5.html#jvms-5.4.3.1).

A resolved method entry stores the declaring MethodId and any distinct symbolic
owner identity needed by resolution semantics. An inherited method may belong to
a different class from the one named in the symbolic reference.

Resolution follows this order:

1. Retain the source class while loading and lookup run.
2. Retain the candidate target while deciding the result.
3. Record the source-CLD to target-CLD dependency.
4. Publish the resolved IDs, or use the already published winner.
5. Acquire/transfer a target root for independent use, such as a new frame.

Dependencies are graph edges, never long-lived root guards stored in metadata.
Otherwise a cache could make a loader permanently live. Conservative extra edges
from losing resolutions are safe but may retain loaders longer; document that cost.
Do not run recursive resolution inside a one-time initializer that can wait on
itself. OnceLock can publish an already computed result. Resolution failures need
an explicit policy compatible with the supported JVM subset, not accidental retry
behavior. This lifetime design does not approve broader method-lookup semantics.

## 6. Frames, instructions, and escaped values

An Invocation owns a RootedMethod and argument storage. Derive the declaring class
from MethodId; avoid independently writable owner/method fields that can disagree.
An interpreter frame owns that invocation, locals, operand stack, and program
counter. It does not store references borrowed from its own root.

Access code briefly from the root during execution. Keep instructions as compact
values with indices into class/code-owned constant and switch tables. This avoids
cloning large switch tables and avoids returning arena-lived instruction views
from `next_inst`. A frame can split a borrow of immutable code from mutable
execution state. Resolving a frame's root once need not imply a registry lookup
or reference-count update for every bytecode.

Budget exhaustion retains the frame and its root. A return transfers the result,
pops exactly one frame, and delivers to the caller. Top-level return terminates
the thread and releases its frame. Error, cancellation, and thread destruction
also need explicit cleanup paths; unwinding drops guards exactly once.

Java object arguments require a separate root policy. Before GC is enabled,
define these ownership transitions:

| Location | Future GC contract |
| --- | --- |
| Installed frame locals and operand stack | Enumerate live reference slots; allow the collector to update them |
| Pending invocation not yet installed | Temporary registered argument roots |
| Return in transit between frames | Temporary root, or a bounded transfer during which collection cannot occur |
| Reference result returned to native caller | Owned object-root handle, not a bare NObjPtr |
| Diagnostic error | Owned text and primitive snapshots; object arguments become non-dereferenceable diagnostic tokens |

The diagnostic rule also covers `ArgumentTypeMismatch.actual`, not just the
invocation summary. An error intentionally retaining a Java exception instead
needs an explicit object root. Preserve all references during transfer before
removing their previous roots; a C++/Rust local variable is not automatically
visible to a Java collector.

## 7. Heap handles and the C boundary

Separate externally registered strong roots, weak handles, and CLD-owned heap
slots. A CLD-owned slot becomes strong when its owning CLD is traced; it is not
an unconditional VM root in the eventual collector.

Let slot storage own stable boxed slots, independent of the metaspace allocator.
A scoped/owned external handle registers and deregisters its slot. A metadata
record stores a HeapSlotId into its CLD-owned storage. This removes OOPStorage's
borrow of a sibling MsAllocator. Slot reuse needs stale-ID protection just as
metadata reuse does. Weak handles require a synchronized upgrade to a strong
root; reading a non-null weak slot is not enough.

NObjPtr and compressed Klass addresses remain representation details. Decoding
an address does not prove that the object or metadata is alive. A Rust reference
into a Java object must be limited to an access period during which collection
cannot invalidate it, and must obey Rust aliasing rules for mutable fields.
If a moving collector is added, roots must be updated and raw object references
must not cross its safepoints. An AtomicU32 slot alone solves neither problem.

C receives borrowed class addresses only through a documented runtime/collector
protocol. A live Java object must keep its defining CLD live before its class
header is decoded for normal access. During collection all metadata needed to
scan heap objects remains allocated until tracing and relevant heap sweeping
are complete. Dead objects left in heap storage must never be scanned later
through a freed/reused class pointer.

## 8. Symbols and global resources

Symbols contain immutable text and are not a Java loader graph. Prefer
Arc<Symbol> with a synchronized weak interner over the current custom counter.
Interning upgrades or creates the canonical live symbol under a bucket lock;
remove expired weak entries and their string keys so the interner itself does
not grow forever. Preserve equality/hash consistency and canonical identity
while handles remain alive. Do not apply strong Arc edges to the CLD graph:
mutually referencing loaders would leak.

Keep parser-owned buffers and temporary ClassImage data as ordinary owned Rust
values. Do not retain input-buffer borrows in published metadata.

Wrap native reservations in an owner immediately after successful acquisition,
before subsequent initialization can fail. Commit that owner only after success.
Apply this to VirtSpace initialization as well as class construction.

Current globals and native heap initialization have process-wide lifetimes;
Rust static destructors do not provide a VM shutdown protocol. Keep the existing
one-successful-initialization contract. A future explicit shutdown must stop
execution, release external/frame roots, destroy metadata and slots while their
services remain valid, then release backing memory. It must not race outstanding
guards. Restarting or embedding multiple VMs is a separate design decision.

## 9. Future unloading contract — deferred implementation

Start with stop-the-world collection: participating execution pauses at known
safe points. This requires a thread-participation/safepoint protocol that the
current VM does not have. Metadata root registration, dependency publication,
and unload eligibility must be serialized against that protocol. No thread may
park at a safepoint while holding a lock the collector needs. Foreign/native
threads accessing metadata participate too.

Trace heap and CLD edges together until neither discovers more reachable data:

- Bootstrap, active/suspended frames, and explicit external roots retain CLDs.
- A reachable loader, represented class mirror, or instance retains its relevant CLD.
- A live CLD traces its loader object, metadata heap slots, and dependent CLDs.
- Registry membership alone does not retain a CLD.

Thus rooted A with an edge to B retains B. An unreachable A/B cycle can disappear
together. Do not root every class mirror or every loader-owned strong slot
globally; doing so can make the graph retain itself forever.

Only after joint reachability and supported weak-reference/finalization rules
are settled may the collector reject new roots for dead CLDs, remove lookup
entries, invalidate IDs, clear dependent external weak entries, destroy owned
resources, and recycle chunks. All heap scanning that needs dead class layouts
must finish first. Reclamation remains disabled until these requirements hold.
Java 8 permits unloading only when the defining loader can be reclaimed and
forbids unloading bootstrap classes; see
[JLS 8 §12.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-12.html#jls-12.7).

Concurrent unloading, class redefinition, JIT code, JNI IDs, and reflection will
add lifetime obligations. They must use explicit roots or traced dependencies
when introduced; they are not covered by merely having a generational ID.

## 10. Trade-offs and implementation stages

| Alternative | Benefit | Reason not to use it as the main design |
| --- | --- | --- |
| Keep external arenas and split borrow/storage lifetimes | Smallest immediate compiler repair; useful in component tests | Does not give CLD-owned storage or an independently unloadable runtime graph |
| Give everything a VM-long reference | Simple while metadata is permanent | Becomes an API promise that obstructs reclamation during that VM lifetime |
| Pin the current self-referential CLD | Can establish address stability with a correct pinned API | Still needs initialization, destruction, synchronization, and liveness rules; Pin is not a GC root |
| Arc for every metadata relationship | Familiar shared ownership | Strong dependency cycles retain loaders; Weak-only links can lose required targets |
| IDs plus ownership and rooted access | Separates local borrowing from runtime relationships | Adds validation, root accounting, and a small audited unsafe storage boundary |

The recommendation is the last row, with ordinary owned child containers to
minimize its unsafe surface. The Pin distinction follows the
[Rust pinning contract](https://doc.rust-lang.org/std/pin/): it concerns location
stability during a value's lifetime, not Java reachability.

Proposed stages, each subject to implementation approval:

1. Establish a compiling baseline and resolve the unsynchronized cell and symbol
   accounting defects. Do not silently expand the unfinished resolution subset.
2. Introduce CLD-owned storage, typed IDs, root guards, fully owned metadata,
   and transactional publication. Keep all published CLDs retained by the VM.
3. Migrate class tables, hierarchy links, runtime constants, invocations, frames,
   and diagnostics. Retain existing interpreter behavior except explicitly
   approved frame-lifecycle fixes. Remove obsolete metadata lifetime parameters.
4. Specify and implement heap roots and safepoints before enabling GC. Add joint
   heap/CLD tracing and unloading only as a separately approved feature.

Acceptance checks should cover ordinary local construction/drop; references that
cannot escape a root; exactly-once cleanup, including failed builds; concurrent
publication without replacement; independent resolution contexts; frame retention
across budget pauses; cleanup on return/error/cancel; and correct symbol cloning.
Existing numeric/interpreter and ownership checks must continue to pass. Later
collection tests must cover loader cycles, live instances/mirrors/loaders, weak
handles, address and slot reuse, and publication/root acquisition versus unloading.

## 11. Validation of this review

No runtime or test source was changed. On the reviewed working tree:

- `cargo check --manifest-path rust/Cargo.toml --locked --offline --target-dir /private/tmp/myjvm-lifetime-review-target`
  failed with E0308 because MethodRef::resolve has an empty body.
- The same command with `--tests` failed with 13 errors, including that E0308,
  E0597 class-borrow failures, and the explicit NormalKlass invariance diagnostic
  in `tests/support.rs`.

These are compilation findings, not failing runtime tests. No test pass or
memory-safety proof is claimed. Future-collector hazards are source-review
findings because the current native heap does not collect objects.
