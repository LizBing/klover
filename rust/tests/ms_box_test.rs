use std::sync::Once;

use rust::class_loader::ms_box::{MSAllocator, MSBox};

// ── one-time metaspace init ─────────────────────────────────────────

fn ms_init_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let ok = unsafe { rust::class_loader::ms_box::ms_init() };
        assert!(ok, "ms_init: metaspace initialisation failed");
    });
}

// ── helper types ───────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C, align(64))]
struct OverAligned {
    data: [u8; 64],
}

// ── basic allocation ───────────────────────────────────────────────

#[test]
fn test_alloc_basic() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let b = MSBox::new(&allocator, Point { x: 10, y: 20 });
    assert_eq!(b.x, 10);
    assert_eq!(b.y, 20);
}

#[test]
fn test_alloc_int() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let b = MSBox::new(&allocator, 42u32);
    assert_eq!(*b, 42);
}

#[test]
fn test_alloc_empty_tuple() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let b = MSBox::new(&allocator, ());
    // Deref yields &(), which is a ZST — benign, no-op
    let _ = *b;
}

// ── multiple allocations ───────────────────────────────────────────

#[test]
fn test_alloc_multiple_distinct_addresses() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let a = MSBox::new(&allocator, 1u64);
    let b = MSBox::new(&allocator, 2u64);
    let c = MSBox::new(&allocator, 3u64);

    let pa = &*a as *const u64;
    let pb = &*b as *const u64;
    let pc = &*c as *const u64;

    // Allocations should be in distinct memory locations.
    assert_ne!(pa, pb);
    assert_ne!(pb, pc);
    assert_ne!(pa, pc);

    // Values are correct.
    assert_eq!(*a, 1);
    assert_eq!(*b, 2);
    assert_eq!(*c, 3);
}

#[test]
fn test_alloc_many_small() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let boxes: Vec<MSBox<u32>> = (0..1000).map(|i| MSBox::new(&allocator, i)).collect();

    for (i, b) in boxes.iter().enumerate() {
        assert_eq!(**b, i as u32);
    }
}

// ── DerefMut ───────────────────────────────────────────────────────

#[test]
fn test_deref_mut() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let mut b = MSBox::new(&allocator, Point { x: 0, y: 0 });
    b.x = 100;
    b.y = 200;

    assert_eq!(b.x, 100);
    assert_eq!(b.y, 200);
}

// ── large allocation (sized-chunk path) ────────────────────────────

#[test]
fn test_alloc_large_object() {
    ms_init_once();
    let allocator = MSAllocator::new();

    // 5 KB > BUMP_THRESHOLD (4 KB) → triggers sized-chunk path.
    let mut b = MSBox::new(&allocator, [0u8; 5 * 1024]);

    // Fill with a pattern and verify.
    for (i, byte) in b.iter_mut().enumerate() {
        *byte = (i & 0xff) as u8;
    }
    for (i, &byte) in b.iter().enumerate() {
        assert_eq!(byte, (i & 0xff) as u8);
    }
}

#[test]
fn test_alloc_mixed_small_and_large() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let small = MSBox::new(&allocator, 7u64);
    let large = MSBox::new(&allocator, [0xffu8; 5000]);
    let another = MSBox::new(&allocator, 42i32);

    assert_eq!(*small, 7);
    assert_eq!(large[0], 0xff);
    assert_eq!(large[4999], 0xff);
    assert_eq!(*another, 42);
}

// ── chunk overflow (beyond 8 KB small-chunk capacity) ──────────────

#[test]
fn test_chunk_overflow_forces_new_chunk() {
    ms_init_once();
    let allocator = MSAllocator::new();

    // Each allocation: 256 bytes → chunk fits ~32 before overflow.
    // Allocating 64 forces at least one chunk switch.
    let boxes: Vec<MSBox<[u8; 256]>> = (0..64)
        .map(|i| {
            let mut arr = [0u8; 256];
            arr[0] = i as u8;
            MSBox::new(&allocator, arr)
        })
        .collect();

    for (i, b) in boxes.iter().enumerate() {
        assert_eq!(b[0], i as u8);
    }
}

// ── alignment ──────────────────────────────────────────────────────

#[test]
fn test_alloc_overaligned() {
    ms_init_once();
    let allocator = MSAllocator::new();

    let b = MSBox::new(&allocator, OverAligned { data: [0xAA; 64] });

    let addr = &*b as *const OverAligned as usize;
    assert_eq!(addr % 64, 0, "address must be 64-byte aligned");

    assert_eq!(b.data[0], 0xAA);
    assert_eq!(b.data[63], 0xAA);
}

// ── drop (allocator destruction) ───────────────────────────────────

#[test]
fn test_drop_allocator_does_not_crash() {
    ms_init_once();

    {
        let allocator = MSAllocator::new();
        let _a = MSBox::new(&allocator, 1u32);
        let _b = MSBox::new(&allocator, [0u8; 6000]);
        let _c = MSBox::new(&allocator, 3.14f64);
    }
    // Allocator dropped here — all chunks freed via ms_free_chunk.
}

#[test]
fn test_drop_empty_allocator_does_not_crash() {
    ms_init_once();

    {
        let _allocator = MSAllocator::new();
    }
    // No allocations, just drop.
}

// ── thread safety ──────────────────────────────────────────────────

#[test]
fn test_concurrent_allocations() {
    ms_init_once();
    use std::sync::Arc;

    let allocator = Arc::new(MSAllocator::new());
    let mut handles = Vec::new();

    for tid in 0..8 {
        let a = Arc::clone(&allocator);
        handles.push(std::thread::spawn(move || {
            let mut boxes = Vec::new();
            for i in 0..100 {
                boxes.push(MSBox::new(&a, (tid, i)));
            }
            boxes
        }));
    }

    for h in handles {
        let boxes = h.join().unwrap();
        for (tid, i) in boxes.iter().map(|b| **b) {
            assert!(tid < 8, "thread id out of range");
            assert!(i < 100, "loop index out of range");
        }
    }
}

// ── Send + Sync compile-time checks ────────────────────────────────

#[test]
fn test_msbox_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<MSBox<i32>>();
}

#[test]
fn test_msbox_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<MSBox<i32>>();
}

// ── Default impl ───────────────────────────────────────────────────

#[test]
fn test_allocator_default() {
    ms_init_once();
    let allocator = MSAllocator::default();
    let b = MSBox::new(&allocator, "hello");
    assert_eq!(*b, "hello");
}
