use asm::{prefetchw, CONSUME};

use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize,
ATOMIC_USIZE_INIT, Ordering, fence};

// TODO: investigate some sort of transaction-management structure
// similar to flat-combining

const IN_ARR: usize = 3;

static WRITE_NUM: AtomicUsize = ATOMIC_USIZE_INIT;

enum NodeInd {
    Next = 0,
    Deleted = 1,
    Dummy = 2,
}

struct Node {
    ptrs: [AtomicPtr<Node>; 3],
    write_tag: AtomicUsize,
    val: usize,
    locked: AtomicBool,
}

impl Node {
    pub fn validate_cell_bulk(&self, reader: usize) {
        if self.locked.load(Ordering::Relaxed) {
            false
        } else {
            self.write_tag.load(Ordering::Relaxed) != reader
        }
    }

    #[inline(always)]
    pub fn validate_cell(&self, reader: usize) {
        let rval = self.validate_cell_bulk(reader);
        if rval {
            fence(Ordering::Acquire);
        }
        rval
    }
}

fn traverse_to(head: &AtomicPtr<Node>, val: usize) -> (*mut Node, *mut Node) {
    unsafe {
        let cur_ptr = head.load(Consume);
        let prev_ptr = ptr::null_mut();
        loop {
            let cur_node = &*cur_ptr;
            if cur_node.val()
        }
        (prev_ptr, cur_ptr)
    }
}

enum IndexOptions {
    AddFrom(*mut Node),
    DeleteFrom(*mut Node),
}

// This looks really weird over an enum, but it removes pointless branching
struct WriteOp {
    node: *mut Node,
    write_val: *mut Node,
    option: NodeInd,
}

impl WriteOp {
    fn commit(self, write_version: usize) {
        unsafe {
            let ptr = &(*self.node).ptrs.get_unchecked(self.option as usize);
            ptr.store(self.write_val, Ordering::Relaxed);
        }
    }
}

struct Transaction {
    read_set: Vec<ReadID>,
    write_set: Vec<WriteOp>,
}

struct WriteGuard<'a> {
    write_set: &'a Vec<WriteOp>,
}

impl<'a> Drop for WriteGuard<'a> {
    fn drop(&mut self) {
        fence(Ordering::Release);
        unsafe {
            for val in self.write_set {
                (*val.node).locked.store(false, Ordering::Relaxed);
            }
        }
    }
}

impl Transaction {
    fn acquire_writes(&self) -> Result<WriteGuard, ()> {
        unsafe {
            // These prefetches really help the CAS latency
            // since the reads-for-ownership which at least on x86
            // the CAS blocks on can all happen concurrently
            // hopefully leading to the later cas instructions have a much lower latency
            for node in self.write_set.iter() {
                prefetchw(&(*node.node).locked);
            }
            let mut i = 0;
            let len = self.write_set.len();
            while i < len {
                let node = &*self.write_set.get_unchecked(i).node;
                if node.locked.load(Ordering::Relaxed) {
                    break;
                }
                if node.locked
                    .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                    .is_err() {
                    break;
                }
                i += 1;
            }
            if i != len {
                for node in self.write_set.iter().take(i) {
                    (*node.node).locked.store(false, Ordering::Relaxed);
                }
                Err(())
            } else {
                fence(Ordering::Acquire);
                Ok(WriteGuard { write_set: &self.write_set })
            }
        }
    }

    fn validate_reads(&self) {
        true
    }

    fn commit(self) -> Result<(), ()> {
        let write_guard = try!(self.acquire_writes());
        prefetchw(&WRITE_NUM);
        if !self.validate_reads() {
            return Err(());
        }
        self.write_set.drain(..).map(|x| x.commit());

    }
}
