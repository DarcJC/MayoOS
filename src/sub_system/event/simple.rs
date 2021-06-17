use core::sync::atomic::{AtomicU64, Ordering};
use alloc::collections::{BinaryHeap, LinkedList};
use core::future::Future;
use alloc::boxed::Box;
use spin::RwLock;
use core::task::{Context, Poll};
use core::pin::Pin;
use futures_util::future::UnsafeFutureObj;
use futures_util::TryFuture;
use lazy_static::lazy_static;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub struct BusId(u64);

impl BusId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn new_unchecked(id: u64) -> Self {
        Self(id)
    }
}

pub struct EventBus {
    pub id: BusId,
    hooks: RwLock<LinkedList<Pin<Box<dyn Future<Output=()>>>>>,
}

impl EventBus {
    pub fn register(&self, hook: impl Future<Output=()> + 'static) {
        self.hooks.write().push_back(Box::pin(hook));
    }

    pub fn new() -> Self {
        Self {
            id: BusId::new(),
            hooks: RwLock::new(LinkedList::new()),
        }
    }

    async fn run(&self) {
        let hooks = self.hooks.read();
        async {
            for future in hooks.iter() {
                unsafe {
                    let m = &mut * (future as * const Pin<Box<dyn Future<Output=()>>> as *mut Pin<Box<dyn Future<Output=()>>>);
                    m.await;
                }
            }
        }.await;
    }
}


// static KEYBOARD_EVT: RwLock<EventBus> = RwLock::new(EventBus::new());

// lazy_static! {
//     static ref EVENT_BUSES: RwLock<LinkedList<EventBus>> = RwLock::new(LinkedList::new());
// }
//
// pub fn create_event_bus() -> BusId {
//     let new_bus = EventBus::new();
//     let id = new_bus.id;
//     EVENT_BUSES.write().push_back(new_bus);
//
//     id
// }
//
// /// If give id exist in buses list, return given id
// ///
// /// else return a new bus id
// pub fn get_or_create_event_bus(id: u64) -> BusId {
//     if let Some(res) = EVENT_BUSES.read().iter().find(|evt| {
//         evt.id == id
//     }) {
//         res.id
//     } else {
//         create_event_bus()
//     }
// }
//
// impl BusId {
//     pub fn fire(&self) {
//     }
// }
