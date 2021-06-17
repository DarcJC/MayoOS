use crossbeam_queue::ArrayQueue;
use conquer_once::spin::OnceCell;
use crate::println;
use futures_util::{Stream, StreamExt};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::task::AtomicWaker;
use pc_keyboard::{Keyboard, layouts, ScancodeSet1, HandleControl, DecodedKey};
use crate::print;
use pc_keyboard::KeyCode::Enter;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input...");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

/// use `_private` field to prevent `ScancodeStream`
/// be constructed out of module
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        Self {
            _private: (),
        }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("SCANCODE_QUEUE isn't initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            },
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_keypress() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        super::init::push_key(character);
                    },
                    DecodedKey::RawKey(key) => {
                        // print!("{:?}", key)
                        if key == Enter {
                            super::init::push_key('\n');
                        }
                    },
                }
            }
        }
    }
}
