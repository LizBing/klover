use super::*;
use crate::tests::support::ensure_metaspace;

#[test]
fn handles_are_distinct_and_registered_by_strength() {
    ensure_metaspace();
    let arena = MsAllocator::new();
    let storage = OOPStorage::new(&arena);
    let first = storage.create_strong();
    let second = storage.create_strong();
    let weak = storage.create_weak();
    assert!(!std::ptr::eq(first.slot, second.slot));
    assert!(!std::ptr::eq(first.slot, weak.slot));
    let strongs = storage.strongs.lock();
    let weaks = storage.weaks.lock();
    assert_eq!(strongs.len(), 2);
    assert_eq!(weaks.len(), 1);
    assert_eq!(strongs[0].0.as_ptr().cast_const(), first.slot as *const _);
    assert_eq!(strongs[1].0.as_ptr().cast_const(), second.slot as *const _);
    assert_eq!(weaks[0].0.as_ptr().cast_const(), weak.slot as *const _);
}

#[test]
fn concurrent_handles_remain_distinct_and_registered() {
    ensure_metaspace();
    let arena = MsAllocator::new();
    let storage = OOPStorage::new(&arena);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    (0..100)
                        .map(|_| storage.create_strong())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let handles: Vec<_> = workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect();
        let addresses: std::collections::HashSet<_> = handles
            .iter()
            .map(|handle| handle.slot as *const _ as usize)
            .collect();
        assert_eq!(addresses.len(), 400);
        let registered = storage.strongs.lock();
        assert_eq!(registered.len(), 400);
        assert!(
            registered
                .iter()
                .all(|slot| addresses.contains(&(slot.0.as_ptr() as usize)))
        );
    });
}
