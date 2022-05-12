use std::convert::Infallible;
use hyper::{Body, Request, Response, Server};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};

pub struct HttpServer {
    addr: [u8; 4],
    port: u16
}
impl HttpServer {
    pub fn new(addr: [u8; 4], port: u16) -> HttpServer {
        HttpServer {addr, port}
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>  {
        let make_svc = make_service_fn(|socket: &AddrStream| async {
            Ok::<_, Infallible>(service_fn(handle_request))
        });

        let host = (self.addr, self.port).into();
        let server = Server::bind(&host).serve(make_svc);
        println!("Listening on http://{}", host);
        let graceful = server.with_graceful_shutdown(shutdown_signal());
        graceful.await?;
        Ok(())
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("hello".into()))
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
