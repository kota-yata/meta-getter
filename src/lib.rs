pub struct ThreadPool;

impl ThreadPool {
  pub fn new(size: usize) -> ThreadPool {
      ThreadPool
  }
  pub fn execute<F>(&self, f: F)
  where F: FnOnce() + Send + 'static,
  {}
}

// impl Drop for ThreadPool {
//   fn drop(&mut self) {
//     for worker in &mut self.workers {
//       println!("Shutting down worker {}", worker.id);
//       worker.thread.join().unwrap();
//     }
//   }
// }
