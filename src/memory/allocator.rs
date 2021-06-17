use x86_64::{VirtAddr, structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, mapper::MapToError}};
// use linked_list_allocator::LockedHeap;
// use crate::memory::allocator::bump::BumpAllocator;
// use crate::memory::allocator::linked::LinkedListAllocator;
use crate::memory::allocator::block::FixedBlockAllocator;


pub const HEAP_START: usize = 0x_6000_0000_0000;
pub const HEAP_SIZE: usize = 1000 * 1024; // 100 KiB

#[global_allocator]
// static ALLOCATOR: LockedHeap = LockedHeap::empty();
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new( BumpAllocator::new());
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
static ALLOCATOR: Locked<FixedBlockAllocator> = Locked::new(FixedBlockAllocator::new());

pub trait MayoAllocator {
    fn new_for_fallback() -> Self;
    unsafe fn init(&mut self, heap_start: usize, heap_size: usize);
}

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start = Page::containing_address(heap_start);
        let heap_end = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start, heap_end)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    };

    Ok(())
}

pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2")
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub mod bump;
pub mod linked;
pub mod block;
