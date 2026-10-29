# Metadata lifetime design proposal

Status: proposed, not approved for implementation. Prepared 2026-10-28.

This proposal replaces the earlier suggestion of one VM-wide Rust metadata
lifetime. The existing MethodRef lookup draft remains incomplete and unverified.
This document does not approve that draft or the implementation stages below.

## Ownership and validity

Each class belongs to its defining ClassLoaderData (CLD). Its methods, code,
runtime constants, and other metadata are reclaimed together with that CLD.
Loading a class through a delegating loader does not transfer its ownership.
Bootstrap metadata remains alive for the VM's lifetime. These rules agree with
[JLS 8 section 12.7](https://docs.oracle.com/javase/specs/jls/se8/html/jls-12.html#jls-12.7),
which ties unloading eligibility to the defining loader's collectability.

There are three separate responsibilities:

| Responsibility | Proposed mechanism |
| --- | --- |
| Own allocations and release resources | One owner per CLD, including its arena and cleanup records |
| Decide whether a CLD is still needed | Runtime tracing of heap objects, VM roots, and CLD dependencies |
| Bound a Rust reference to valid storage | A short borrow from an explicit metadata root guard |

A Rust lifetime cannot by itself represent the changing reachability of loaders
in a running JVM. Durable metadata links should therefore use opaque handles;
Rust references should describe individual protected accesses.

## Liveness graph

The proposed collector treats the following as starting points or tracing rules:

- The bootstrap CLD is a permanent root.
- Explicit native metadata roots and active or suspended frames retain their CLDs.
- A reachable Java ClassLoader object retains its associated CLD.
- A reachable class mirror retains the CLD of the class it represents.
- A live Java instance retains its class's defining CLD through its class metadata.
- A live CLD retains CLDs referenced by its metadata, including superclasses,
  interfaces, delegated class-table entries, and resolved constant-pool targets.
- A live CLD traces its loader object and its strong heap references, including
  static reference fields and runtime constants. Weak references remain weak.

Heap marking and CLD marking must reach a common fixed point: marking a heap
object may discover a CLD, and marking a CLD may discover heap objects or other
CLDs. Only live heap objects contribute class edges. Enumerating every allocated
object or every registered CLD must not make them all roots.

In particular, the CLD registry is an ownership/enumeration structure, not an
unconditional GC root. Strong OOP slots belonging to an unloadable CLD are traced
when that CLD becomes live. Otherwise a CLD, its mirrors, and its loader could
keep themselves alive forever.

For a cached method reference from CLD A to a method defined in CLD B:

```text
external root -> A -> B
```

A live A retains B. B need not retain A. If A and B reference one another but
neither is reachable from an external root, both remain collectible. Internal
dependencies are tracing edges, not permanent root guards or Arc ownership.
OpenJDK 8's [CLD implementation](https://github.com/openjdk/jdk8u/blob/master/hotspot/src/share/vm/classfile/classLoaderData.cpp)
provides a useful reference for recording loader dependencies and releasing
CLD-owned resources; the Rust API below is a separate proposal.

## Handles and protected access

Proposed persistent handles are CldId, KlassId, and MethodId. They have private
construction, carry enough identity to reject reused slots, and provide no safe
Deref implementation. Copying an ID does not keep its target alive.

A registry entry's generation changes before its slot can be reused. A stale
handle must be rejected before reading freed arena memory. Generation exhaustion
must retire a slot rather than silently alias an old identity.

An explicit root guard registers a temporary external root for a CLD. A borrowed
metadata view cannot outlive that guard. For example, the intended API shape is:

```rust
// Illustrative API, not implemented definitions.
fn root_method(&self, id: MethodId) -> Result<RootedMethod, StaleMetadata>;

impl RootedMethod {
    fn method(&self) -> &Method;
}
```

Root acquisition validates identity and registers the root as one operation with
respect to unloading. The CLD's metadata remains at stable addresses while the
guard exists, including across safepoints. Ordinary metadata records are immutable
after publication; caches and linking state use explicit synchronization.

This replaces lifetime parameters representing the complete runtime lifetime of
published Klass, Method, and MethodRef records. It does not remove ordinary Rust
borrows from buffers, views, or scoped allocation APIs. Arena slices also need
owner-aware wrappers; removing a generic lifetime must not manufacture static
references.

Raw metadata addresses, including compressed class pointers in existing C object
headers, remain private to the trusted runtime/collector boundary. Decoding an
address alone does not establish liveness. Safe Rust access requires a guard;
collector access requires its exclusive collection protocol. Durable native
metadata references must participate in rooting as well.

## Frames and method resolution

Frames retain an owned metadata root for their executing method's defining CLD.
That root remains registered while a frame is suspended between budget slices.
Removing the frame releases it. Frame fields should hold handles/root guards,
and obtain short metadata borrows during execution, avoiding a frame that owns a
guard while storing a reference borrowed from that same guard.

A MethodRef belongs to a particular referring class/CLD. Its resolution context
is established during metadata construction, so callers cannot resolve the same
cache through different loaders. Resolution uses that referring class's defining
loader, as specified by
[JVMS 8 section 5.4.3.1](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-5.html#jvms-5.4.3.1).

The proposed cache holds OnceLock<ResolvedMethod>, where ResolvedMethod contains
the target MethodId and declaring KlassId. The symbolic owner may differ from
the declaring class when lookup finds a superclass method.

On successful resolution:

1. Keep the source CLD rooted throughout loading and lookup.
2. Acquire protection for the target CLD before it can be reclaimed.
3. Register the source-to-target dependency before publishing the cached target.
4. Publish or use the concurrent winner; ensure its dependency is already visible.
5. Return a rooted result when the method must survive independently of the source.

No unprotected gap may exist between lookup, dependency registration, cache
publication, and frame installation. A failed registration must not publish a
usable target. Conservatively retained dependencies can remain until the source
CLD is reclaimed. Method lookup semantics and error caching are separate decisions;
this proposal does not expand the previously approved lookup subset.

## Reclamation protocol

Start with stop-the-world CLD reclamation: participating threads stop at agreed
safepoints before metadata is freed. This is a proposal, not an existing capability
of the VM. Concurrent unloading requires a separate synchronization design.

Root registration, dependency publication, and unloading share a defined
synchronization boundary. Short critical sections cannot reach a safepoint or
call Java. The collector must not wait for a lock held by a parked thread.
Loading and Java execution happen outside those critical sections. Native threads
must register their participation before accessing metadata.

At collection:

1. Stop participating threads and establish a consistent root snapshot.
2. Trace heap and metadata edges to a fixed point, respecting weak-reference and
   eventual finalization semantics before deciding unload eligibility.
3. Mark unreachable CLDs as unloading and reject new root acquisition for them.
4. Remove their externally discoverable entries and invalidate their identities.
5. Release owned non-arena resources, unregister OOP slots, and return arena chunks.
6. Resume threads after all metadata-visible state is consistent.

A CLD's own strong pointer to its Java loader is conditional on CLD liveness;
it must not turn the loader into an unconditional root. Reachable loader objects,
including reachable empty loaders, still prevent their CLDs from unloading.

## Resource cleanup

MsBox::leak currently publishes references while skipping destructors. Dropping
MsAllocator returns chunks but does not destroy all published values. That is
insufficient for unloadable metadata containing Box, Vec, HashMap, or SymbolHandle.

The proposed CLD owner must track destruction of owned resources, either through
typed ownership records or registered cleanup actions. Each owned value is cleaned
up exactly once before its arena storage is recycled. Non-owning metadata links
are never recursively destroyed. Cleanup must not dereference another dead CLD;
unreachable dependency cycles may be destroyed in any order.

The current external-arena API can remain useful for construction and component
tests. Publishing unloadable runtime metadata needs an ownership layer that
records cleanup and avoids a Rust struct borrowing another of its own fields.

## Trade-offs and staged validation

This design allows runtime CLD reclamation and collectible dependency cycles,
with short Rust borrows at use sites. Its costs are handle validation, explicit
roots, dependency tracking, and a small audited unsafe boundary around arena
access. Plain Arc links cannot collect cycles; a VM-wide reference lifetime
would prevent the intended reclamation.

Possible implementation sequence, not yet approved:

1. Prove handle identity, guarded borrowing, root registration, cleanup, and
   dependency tracing in an isolated metadata subsystem. Use an explicit test
   collection entry point; do not expose production unloading before safepoints.
2. Migrate class tables, hierarchy links, constants, frames, and MethodRef caches
   to the agreed API. Preserve the focused method-lookup scope.
3. Integrate heap tracing, native roots, safepoints, and actual CLD reclamation.

Required tests include suspended-frame retention; A retaining B while A is live;
collection of unreachable dependency cycles; reachable loaders, mirrors, and
instances preserving their CLDs; unconditional bootstrap retention; conditional
OOP-slot tracing; stale-handle rejection after address/slot reuse; exactly-once
cleanup; and root/publication races against unloading. Compiler checks must
reject references escaping a guard or dropping a guard while its borrow is used.

The first decision for review is the ownership model: CLD-owned storage, runtime
tracing for liveness, and guarded Rust borrows for access. Handle encoding, exact
synchronization primitives, and migration patches remain follow-up design work.
