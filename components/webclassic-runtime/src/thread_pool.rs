use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use webclassic_service::interrupt::{Interrupt, InterruptSource};

pub struct ThreadPool {
    workers: Vec<WorkerHandle>,
    sender: Option<SyncSender<Job>>,
    next_id: usize,
}

impl ThreadPool {
    pub fn new(worker_num: NonZero<usize>, queue_size: NonZero<usize>) -> ThreadPool {
        let (sender, jobs) = mpsc::sync_channel(queue_size.into());
        let jobs = Arc::new(Mutex::new(jobs));
        Self {
            workers: (0..worker_num.into())
                .map(|_| Worker::new(Arc::clone(&jobs)).spawn())
                .collect(),
            sender: Some(sender),
            next_id: 1,
        }
    }

    pub fn dispatch<F>(&mut self, procedure: F) -> bool
    where
        F: FnOnce(Interrupt) + Send + 'static,
    {
        if let Some(sender) = self.sender.as_mut() {
            let job = Job::new(self.next_id, Box::new(procedure));
            self.next_id = self.next_id.wrapping_add(1);

            match sender.try_send(job) {
                Ok(_) => true,
                Err(TrySendError::Full(job)) => {
                    if let Some(worker) = (self.workers.iter())
                        .min_by_key(|worker| worker.current_id.load(Ordering::Acquire))
                    {
                        worker.interrupt.trigger();
                    }
                    sender.send(job).is_ok()
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn shutdown(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            worker.interrupt.trigger()
        }
    }

    pub fn join(self) {
        for worker in self.workers {
            let _ = worker.join.join();
        }
    }
}

struct Job {
    id: usize,
    procedure: Box<dyn FnOnce(Interrupt) + Send>,
}

impl Job {
    fn new(id: usize, procedure: Box<dyn FnOnce(Interrupt) + Send>) -> Self {
        Self { id, procedure }
    }
}

struct WorkerHandle {
    join: JoinHandle<()>,
    interrupt: InterruptSource,
    current_id: Arc<AtomicUsize>,
}

impl WorkerHandle {
    fn new(join: JoinHandle<()>, interrupt: InterruptSource, current_id: Arc<AtomicUsize>) -> Self {
        Self {
            join,
            interrupt,
            current_id,
        }
    }
}

struct Worker {
    jobs: Arc<Mutex<Receiver<Job>>>,
    interrupt: InterruptSource,
    current_job_id: Arc<AtomicUsize>,
}

impl Worker {
    fn new(jobs: Arc<Mutex<Receiver<Job>>>) -> Self {
        let interrupt = InterruptSource::new();
        let current_job_id = Arc::new(AtomicUsize::new(0));
        Self {
            jobs,
            interrupt,
            current_job_id,
        }
    }

    fn spawn(self) -> WorkerHandle {
        let interrupt = self.interrupt.clone();
        let current_job_id = Arc::clone(&self.current_job_id);
        WorkerHandle::new(
            std::thread::spawn(move || {
                loop {
                    let Ok(job) = self.jobs.lock().unwrap().recv() else {
                        break;
                    };
                    self.interrupt.reset();

                    self.current_job_id.store(job.id, Ordering::Release);
                    (job.procedure)(self.interrupt.subscribe());
                }
            }),
            interrupt,
            current_job_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use super::*;

    #[test]
    fn dispatch_multiple() {
        let mut pool = ThreadPool::new(NonZero::new(4).unwrap(), NonZero::new(2).unwrap());
        let count = Arc::new(AtomicUsize::new(0));
        let total = 20;

        for _ in 0..total {
            let count = Arc::clone(&count);
            pool.dispatch(move |_interrupt| {
                count.fetch_add(1, Ordering::Relaxed);
            });
        }

        pool.shutdown();
        pool.join();

        assert_eq!(count.load(Ordering::Relaxed), total);
    }

    #[test]
    fn shutdown_and_join() {
        let mut pool = ThreadPool::new(NonZero::new(2).unwrap(), NonZero::new(2).unwrap());
        let done = Arc::new(AtomicBool::new(false));

        let done_clone = Arc::clone(&done);
        pool.dispatch(move |_interrupt| {
            done_clone.store(true, Ordering::Relaxed);
        });

        pool.shutdown();
        pool.join();

        assert!(done.load(Ordering::Relaxed));
    }

    #[test]
    fn interrupt_triggered_when_job_queue_is_full() {
        let mut pool = ThreadPool::new(NonZero::new(1).unwrap(), NonZero::new(1).unwrap());
        let interrupted = Arc::new(AtomicBool::new(false));

        let interrupted_clone = Arc::clone(&interrupted);
        pool.dispatch(move |interrupt| {
            while !interrupt.is_interrupted() {}
            interrupted_clone.store(true, Ordering::Relaxed);
        });

        let barrier = Arc::new(AtomicBool::new(false));
        let barrier_clone = Arc::clone(&barrier);
        pool.dispatch(move |_interrupt| {
            barrier_clone.store(true, Ordering::Relaxed);
        });

        pool.shutdown();
        pool.join();

        assert!(interrupted.load(Ordering::Relaxed));
        assert!(barrier.load(Ordering::Relaxed));
    }
}
