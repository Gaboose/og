use futures::prelude::*;
use futures::Future;
use hyper::header::HeaderValue;
use hyper::service::Service;
use std::pin::Pin;

pub struct AllowAllOrigins<S> {
    upstream: S,
}

impl<S> AllowAllOrigins<S> {
    pub fn new(upstream: S) -> AllowAllOrigins<S> {
        AllowAllOrigins { upstream: upstream }
    }
}

impl<S, R, B> Service<R> for AllowAllOrigins<S>
where
    S: Service<R, Response = hyper::Response<B>>,
    S::Future: 'static,
    S::Future: std::marker::Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, request: R) -> Self::Future {
        let resp = self.upstream.call(request).map(|res| match res {
            Ok(mut r) => {
                r.headers_mut()
                    .insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
                Ok(r)
            }
            Err(e) => Err(e),
        });
        resp.boxed()
    }
}
