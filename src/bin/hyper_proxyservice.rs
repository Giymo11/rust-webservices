use futures::future;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use hyper_reverse_proxy::ProxyError;
use std::convert::Infallible;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let in_addr = SocketAddr::from(([127, 0, 0, 1], 3030));

    #[derive(Debug, Copy, Clone)]
    struct ProxyAddr<'a> {
        out_addr: SocketAddr,
        prefix: &'a str,
    }

    let out_addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new("Hello, World".into()))
    }

    fn debug_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let body_str = format!("{:?}", req);
        let response = Response::new(Body::from(body_str));
        Ok(response)
    }

    let make_service = make_service_fn(move |_| {
        async move {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, hyper::Error>(service_fn(move |mut req| async move {
                let proxy1 = ProxyAddr {
                    out_addr: SocketAddr::from(([127, 0, 0, 1], 3031)),
                    prefix: "/hello",
                };

                let proxy2 = ProxyAddr {
                    out_addr: SocketAddr::from(([127, 0, 0, 1], 3032)),
                    prefix: "/proto2",
                };

                let proxies = vec![proxy1, proxy2];
                let path = req.uri().path();

                let proxy = proxies
                    .into_iter()
                    .find(|proxy| path.starts_with(proxy.prefix));

                if proxy.is_some() {
                    let uri_string = format!(
                        "http://{}{}",
                        proxy.unwrap().out_addr,
                        req.uri().path_and_query().map(|x| x.as_str()).unwrap_or("")
                    );
                    let uri = uri_string.parse().unwrap();
                    *req.uri_mut() = uri;
                    println!("requesting {:?}", req);
                    Client::new().request(req).await
                } else {
                    debug_request(req)
                }
            }))
        }
    });

    let server = Server::bind(&in_addr).serve(make_service);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
