use std::{future::{ready, Ready}, rc::Rc};

use actix_web::{cookie::Cookie, dev::{always_ready, forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, web::Form, Error, HttpMessage, HttpRequest, HttpResponse};
use futures_util::future::LocalBoxFuture;

use crate::credentials::{self, Credentials};

pub struct Register {
    creds: Rc<cred_auth::handler::Handler>
}

impl Register {
    pub fn new(creds: Rc<cred_auth::handler::Handler>) -> Register {
        Register{ creds: creds}
    }
}

impl<S, B> Transform<S, ServiceRequest> for Register
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RegisterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RegisterMiddleware::new(service, self.creds.clone())))
    }
}

pub struct RegisterMiddleware<S> {
    service: Rc<S>,
    creds: Rc<cred_auth::handler::Handler>
}

impl<S> RegisterMiddleware<S> {
    pub fn new(service: S, creds: Rc<cred_auth::handler::Handler>) -> Self {
        RegisterMiddleware {
            service: Rc::new(service),
            creds: creds
        }
    }
}

impl<S, B> Service<ServiceRequest> for RegisterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        println!("Calling RegisterMiddleware::call()");
        let s = self.service.clone();
        let c = self.creds.clone();

        Box::pin(async move {
            let credentials = req.extract::<Form<Credentials>>().await?;
            if let Ok(()) = c.register(credentials.username.clone(), credentials.password.clone()) {
                println!("Registration successful");
                Ok(s.call(req).await?)
            } else {
                println!("Registration failed");
                Err(actix_web::error::ErrorForbidden("Nope"))
            }
        })
    }
}

pub struct RegisterService {
    creds: Rc<cred_auth::handler::Handler>
}

impl RegisterService {
    pub fn new(creds: Rc<cred_auth::handler::Handler>) -> RegisterService {
        RegisterService { creds: creds }
    }
}

impl Service<ServiceRequest> for RegisterService
{
    type Response = HttpResponse;
    
    type Error = Error;
    
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    always_ready!();
    
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let c = self.creds.clone();

        Box::pin(async move {
            let credentials = req.extract::<Form<Credentials>>().await?;
            if let Ok(()) = c.register(credentials.username.clone(), credentials.password.clone())
            {
                //let response = ServiceResponse::new(req, actix_web::HttpResponse::Ok().finish());
                //Err(actix_web::error::ErrorForbidden("Register failed"))
                //Ok(actix_web::error::ErrorForbidden("Register failed"))
                //Ok(ServiceResponse::new::<HttpResponse>(req, actix_web::HttpResponse::Ok().finish()))
                Ok(actix_web::HttpResponse::Ok().finish())
            } else {
                Err(actix_web::error::ErrorForbidden("Register failed"))
            }
        })
    }
}