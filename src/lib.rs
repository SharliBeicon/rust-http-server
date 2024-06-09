use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Creates a new ThreadPool.
    ///
    /// Size is the number of threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if size is 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(sender) = self.sender.as_ref() {
            if let Err(err) = sender.send(job) {
                println!("Error: {err}");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {0}", worker.id);

            if let Some(thread) = worker.thread.take() {
                match thread.join() {
                    Ok(_) => println!("Worker {0} stopped", worker.id),
                    Err(_) => println!("Something went wrong stopping worker {0}", worker.id),
                }
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            match receiver
                .lock()
                .unwrap_or_else(|err| panic!("Error getting info from receiver: {err}"))
                .recv()
            {
                Ok(job) => {
                    println!("Worker {id} get a job. Starting...");
                    job();
                }
                Err(_) => {
                    println!("Session finished, terminating worker...");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
