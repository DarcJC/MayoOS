use futures_util::task::AtomicWaker;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, StreamExt};
use core::pin::Pin;
use core::task::{Context, Poll};
use crate::{println, exit_qemu_by_port, QemuExitCode};
use alloc::string::String;
use crate::print;
use crate::driver::fw_cfg::{get_files, read_file_by_name};

static KEY_QUEUE: OnceCell<ArrayQueue<char>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn push_key(c: char) {
    if let Ok(queue) = KEY_QUEUE.try_get() {
        if let Err(_) = queue.push(c) {
            println!("WARNING: scancode queue full; dropping keyboard input...");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: key queue uninitialized");
    }
}

struct KeyStream {
    _private: (),
}

impl KeyStream {
    pub fn new() -> Self {
        KEY_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("KeyStream::new should only be called once");
        Self {_private: ()}
    }
}


impl Stream for KeyStream {
    type Item = char;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let queue = KEY_QUEUE.try_get().expect("KEY_QUEUE isn't initialized");

        // fast path
        if let Ok(key) = queue.pop() {
            return Poll::Ready(Some(key));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(key) => {
                WAKER.take();
                Poll::Ready(Some(key))
            },
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn mayo_init() {
    let mut keys = KeyStream::new();
    let mut str_cache = String::new();

    while let Some(key) = keys.next().await {
        if key == '\n' {
            print!("\n");
            commands(&str_cache);
            str_cache.clear();
        } else {
            str_cache.push(key);
            print!("{}", key);
        }
    }
}

fn commands(command: &str) {
    let mut iter = command.split_ascii_whitespace();
    let label = iter.next().unwrap_or("");
    match label {
        "ls" => {
            let files = get_files();
            files.iter().for_each(|f| {
                println!("|- {}", f.get_name());
            });
        },
        "cat" => {
            let filename = iter.next().unwrap_or("opt/test");
            if let Some(data) = read_file_by_name(filename) {
                if let Ok(s) = String::from_utf8(data.clone()) {
                    println!("{}", s);
                } else {
                    println!("{:#?}", data);
                    println!("Failed to load hex data to string...");
                }
            } else {
                println!("Target file {} doesn't exist", filename);
            }
        },
        "shutdown" => {
            exit_qemu_by_port(QemuExitCode::Success);
        },
        _ => try_or_fail(command),
    }
}

fn try_or_fail(command: &str) {
    println!("Command \"{}\" isn't found", command);
}
