use std::thread;
use std::sync::mpsc; // gives us channel stuff
use std::sync::Arc;
use std::sync::Mutex;

enum Message {
    NewJob(Job),
    Shutdown,
}

pub struct ThreadPool {
	// check to see about <_>
	workers: Vec<Worker>,
	sender: mpsc::Sender<Message>,
}


impl Drop for ThreadPool {
    fn drop(&mut self) {
        // do what we need to shut down our threads
        
        // we want to wait until all the `workers` have finished
        // their work, and then we'll go ahead and bail out

        // each worker has a thread
        // we should wait for each worker's thread to finish

        // thread.join() blocks until the thread finishes running
        // thread.join().unwrap();

        for worker in &mut self.workers {
            eprintln!("Sending shutdown message to worker {}", worker.id);
            self.sender.send(Message::Shutdown).unwrap();
        }

        for worker in &mut self.workers {
            eprintln!("Shutting down worker {}", worker.id);
            

            // worker.thread.join().unwrap();

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }

            // worker.thread.take().unwrap().join().unwrap();
        }



        // ok, we're done waiting for workers to finish
        // bye!  
    }
}


struct Worker {
	id: usize,
	thread: Option<thread::JoinHandle<()>>,
}

trait FnBox {
	fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
	fn call_box(self: Box<F>) {
		(*self)()
	}
}

type Job = Box<dyn FnBox + Send + 'static>;

// 1: Define a Worker struct that has an id and holds a JoinHandle<T>
// 2: Change ThreadPool to holder Workers instead of JoinHandle<T>
// 3: Define a Worker::new(id) that will create a new worker with a given id
// 4: create Workers in ThreadPool::new(), each one given an id that is our index in the loop
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
    	// panic if try to instantiate with negative
    	// number of threads
    	assert!(size > 0);

    	let (sender, receiver) = mpsc::channel();

    	// now let's get an Arc wrapped Mutex around the receiver
    	let receiver = Arc::new(Mutex::new(receiver));

    	// with_capacity() allocates memory initially
    	// instead of growing the vector
    	let mut workers = Vec::with_capacity(size);

    	for id in 0..size {
    		// create workers and store them in a vector
    		workers.push(Worker::new(id, Arc::clone(&receiver)));
    	}

        ThreadPool {
        	workers,
        	sender,
        }
    }

    // where
    // F: FnOnce() -> T + Send + 'static
    // thread::spawn returns a JoinHandle<T>
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    	let job = Box::new(f);

    	self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Worker {
	fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
		// still we don't know how to deal with this closure!

		let thread = thread::spawn(move || {
			loop {
				// use mutex to ensure that receiver is never being messed with
				// by more than one thread at a time.

				// we need to pull one `Message` off of the queue (`receiver`)
                let message = receiver.lock().unwrap().recv().unwrap();
				// let job = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        eprintln!("Worker {} got a job.", id);
                        // we have to actually execute the job
                        job.call_box();
                    }
                    Message::Shutdown => {
                        eprintln!("Worker {} shutting down", id);
                        break;
                    }
                };
	
			}
            // join() happens when we get here
            // what baout thread::exit()			
		});

        let thread = Some(thread);

		Worker {
			id,
			thread,
		}
	}
}



