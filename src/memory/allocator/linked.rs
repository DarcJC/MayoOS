use crate::memory::allocator::{align_up, Locked, MayoAllocator};
use core::mem::{align_of, size_of};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self {
            size,
            next: None,
        }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Adds the given memory region to the front of the list.
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, align_of::<ListNode>()), addr);
        assert!(size >= size_of::<ListNode>());

        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr);
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < size_of::<ListNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }

    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;

        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let res = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return res;
            } else {
                current = current.next.as_mut().unwrap();
            }
        }

        None
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// region is also capable of storing a `ListNode`.
    ///
    /// Returns the adjusted size and alignment as a (size, align) tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(size_of::<ListNode>());
        (size, layout.align())
    }
}

impl MayoAllocator for LinkedListAllocator {
    fn new_for_fallback() -> Self {
        Self::new()
    }

    /// Init allocator with params
    ///
    /// # Note
    /// This method should only be called once.
    unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size)
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size)
    }
}
