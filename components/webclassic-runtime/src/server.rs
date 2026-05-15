use std::net::TcpListener;
use std::num::NonZero;
use std::sync::Arc;

use webclassic_service::service::Service;

use crate::thread_pool::ThreadPool;

pub struct ServerOptions<S> {
    service: Arc<S>,
    max_connections: NonZero<usize>,
    max_pending: NonZero<usize>,
}

impl<S> ServerOptions<S> {
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
            max_connections: 32.try_into().unwrap(),
            max_pending: 128.try_into().unwrap(),
        }
    }

    pub fn max_connections(mut self, max_connections: NonZero<usize>) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn max_pending(mut self, max_pending: NonZero<usize>) -> Self {
        self.max_pending = max_pending;
        self
    }
}

impl<S> ServerOptions<S>
where
    S: Service + Send + Sync + 'static,
{
    pub fn serve(self, listener: TcpListener) {
        let mut executor = ThreadPool::new(self.max_connections, self.max_pending);
        for connection in listener.incoming() {
            let service = Arc::clone(&self.service);
            executor.dispatch(move |interrupt| {
                if let Ok(stream) = connection {
                    let _ = service.run(&stream, &stream, &interrupt);
                };
            });
        }
        executor.shutdown();
        executor.join();
    }
}
