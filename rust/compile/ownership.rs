#![allow(dead_code, unused_imports, unexpected_cfgs)]

// Include production modules at the crate root so their crate:: paths and
// privacy are unchanged. No public test API or copies of domain types needed.
include!("../src/lib.rs");

#[link(name = "klover-core", kind = "static")]
unsafe extern "C" {}

use class_loader::class_loader_data::ClassLoaderData;
use runtime::ms_api::{MsAllocator, MsBox};

#[cfg(not(any(
    klover_case = "leak",
    klover_case = "share",
    klover_case = "mutate",
    klover_case = "handle"
)))]
fn main() {
    let arena = MsAllocator::new();
    let mut owner = MsBox::new(&arena, 1);
    let borrowed = owner.share();
    println!("{borrowed}");
    *owner = 2;
    let published = owner.leak();
    println!("{published}");
    let cld = ClassLoaderData::new();
    let storage = cld.oop_storage();
    let handle = storage.create_strong();
    println!("{handle:?}");
    drop(cld);
    drop(arena);
}

#[cfg(klover_case = "leak")]
fn main() {
    let arena = MsAllocator::new();
    let published = MsBox::new(&arena, 42).leak();
    drop(arena); // Must be E0505: cannot move a borrowed allocator.
    println!("{published}");
}

#[cfg(klover_case = "share")]
fn main() {
    let arena = MsAllocator::new();
    let owner = MsBox::new(&arena, String::from("metadata"));
    let borrowed = owner.share();
    drop(owner); // Must be E0505: cannot move a borrowed owner.
    println!("{borrowed}");
}

#[cfg(klover_case = "mutate")]
fn main() {
    let arena = MsAllocator::new();
    let mut owner = MsBox::new(&arena, 1);
    let borrowed = owner.share();
    *owner = 2; // Must be E0502: mutable borrow conflicts with shared borrow.
    println!("{borrowed}");
}

#[cfg(klover_case = "handle")]
fn main() {
    let cld = ClassLoaderData::new();
    let storage = cld.oop_storage();
    let handle = storage.create_strong();
    drop(cld); // Must be E0505: the registry owner must outlive its handle.
    println!("{handle:?}");
}
