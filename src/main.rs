use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs::File;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    fn handle_client(mut stream: TcpStream) {
        let mut buffer = vec![0; 512];
        stream.read(&mut buffer).unwrap();
        let get = b"GET / HTTP/1.1\r\n";

        let start = &buffer[..get.len()];

        let (header, filename) = if start == get {
            ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
        };

        let mut file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let response = format!("{}{}", header, contents);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    let pool = Pool::new(4);

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                pool.execute(|| {
                    handle_client(stream);
                });
            }
            Err(e) => { /* connection failed */ }
        }
    }
}

struct Pool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: u32,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(job_receiver: Arc<Mutex<mpsc::Receiver<Job>>>, id: u32) -> Worker {
        let handle = thread::spawn(move || {
            loop {
                let job = job_receiver.lock().unwrap().recv().unwrap();

                println!("Worker {} got a job.", id);

                job.job.call_box();
            }
        });

        Worker {
            id: id,
            handle: Some(handle),
        }
    }
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

struct Job {
    job: Box<FnBox + Send + 'static>,
}

impl Pool {
    fn new(size: usize) -> Pool {
        assert!(size > 1);

        let (job_sender, job_receiver) = mpsc::channel::<Job>();

        let job_receiver = Arc::new(Mutex::new(job_receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            let worker = Worker::new(job_receiver.clone(), i as u32);

            workers.push(worker);
        }

        Pool {
            workers: workers,
            sender: job_sender,
        }
    }

    fn execute<F: FnOnce() + Send + 'static>(&self, job: F) {
        let job = Job {
            job: Box::new(job),
        };

        self.sender.send(job).unwrap();
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Dropping worker {}", worker.id);
            worker.handle.take().unwrap().join().unwrap();
        }
    }
}