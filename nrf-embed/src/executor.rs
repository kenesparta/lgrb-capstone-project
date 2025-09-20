use crate::future::NewFuture;
use cortex_m::asm;
use heapless::mpmc::Queue;
use rtt_target::rprintln;

pub fn wake_task(task_id: usize) {
    rprintln!("Waking task {}", task_id);
    if TASK_ID_READY.enqueue(task_id).is_err() {
        panic!("Queue is full, can't add task {}", task_id);
    }
}

static TASK_ID_READY: Queue<usize, 32> = Queue::new();

pub fn run_tasks(tasks: &mut [&mut dyn NewFuture<Output = ()>; 3]) -> ! {
    for task_id in 0..tasks.len() {
        TASK_ID_READY.enqueue(task_id).ok();
    }
    loop {
        while let Some(task_id) = TASK_ID_READY.dequeue() {
            if task_id >= tasks.len() {
                rprintln!("Task {} is not valid", task_id);
                continue;
            }

            rprintln!("Task {} is ready", task_id);
            tasks[task_id].poll(task_id);
        }
        rprintln!("Exiting");
        asm::wfi();
    }
}
