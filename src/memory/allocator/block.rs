
use core::alloc::{GlobalAlloc, Layout};
use crate::memory::allocator::{MayoAllocator, Locked};
use linked_list_allocator::Heap;
use core::ptr;
use core::ptr::{NonNull};
use core::mem::{size_of, align_of};

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

struct ListNode {
    next: Option<&'static mut ListNode>
}

pub struct FixedBlockAllocator {
    heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: Heap,
}

impl FixedBlockAllocator {
    /// Create a new `FixedBlockAllocator`
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: Heap::empty(),
        }
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

impl MayoAllocator for FixedBlockAllocator {
    fn new_for_fallback() -> Self {
        Self::new()
    }

    unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }
}

unsafe impl GlobalAlloc for Locked<FixedBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            if let Some(node) = allocator.heads[index].take() {
                allocator.heads[index] = node.next.take();
                node as *mut ListNode as *mut u8
            } else {
                let size = BLOCK_SIZES[index];
                let align = size;
                let layout = Layout::from_size_align(size, align).unwrap();
                allocator.fallback_alloc(layout)
            }
        } else {
            allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            let new_node = ListNode {
                next: allocator.heads[index].take(),
            };
            assert!(size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(align_of::<ListNode>() <= BLOCK_SIZES[index]);

            let new_node_ptr = ptr as *mut ListNode;
            new_node_ptr.write(new_node);

            allocator.heads[index] = Some(&mut *new_node_ptr);
        } else {
            let ptr = NonNull::new(ptr).unwrap();
            allocator.fallback_allocator.deallocate(ptr, layout);
        }
    }
}

