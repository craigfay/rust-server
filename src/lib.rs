use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Create a new thread pool
    ///
    /// The size parameter represents the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }
}

impl ThreadPool {
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {

        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();

    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = std::thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job", id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate", id);
                        break;
                    }
                }
                
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl <F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)();
    }
}
