use std::{future::{ready, Ready}, rc::Rc};

use actix_web::{cookie::{time::Duration, Cookie}, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, web::Form, Error, HttpMessage};

use futures_util::future::LocalBoxFuture;

use crate::{credentials::Credentials, session::{self, SessionDB, SessionToken}};

pub struct Username {
    pub username: String
}

pub struct Authenticate
{
    creds: Rc<cred_auth::handler::Handler>,
    sessions: Rc<session::SessionDB>
}

impl Authenticate {
    pub(crate) fn new(creds: Rc<cred_auth::handler::Handler>, sessions: Rc<session::SessionDB>) -> Authenticate {
        Authenticate { creds: creds, sessions: sessions }
        /*match cred_auth::handler::Handler::new(cred_auth::handler::Storage::File(db_path)) {
            Ok(creds) =>  Ok(Authenticate { creds: Rc::new(creds) }),
            _ => Err(())
        }*/
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authenticate
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticateMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticateMiddleware::new(service, self.creds.clone(), self.sessions.clone())))
    }
}

pub struct AuthenticateMiddleware<S> {
    service: Rc<S>,
    creds: Rc<cred_auth::handler::Handler>,
    sessions: Rc<session::SessionDB>
}

impl<S> AuthenticateMiddleware<S> {
    fn new(service: S, creds: Rc<cred_auth::handler::Handler>, session: Rc<SessionDB>) -> AuthenticateMiddleware<S> {
        AuthenticateMiddleware {
            service: Rc::new(service),
            creds: creds,
            sessions: session
        }
    }
}

impl<S, B> Service<ServiceRequest> for AuthenticateMiddleware<S>
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
        let srv = self.service.clone();
        let c = self.creds.clone();
        let s = self.sessions.clone();

        Box::pin(async move {
            // Do we recognize the session token?
            println!("Testing cookie...");
            {
                if let Some(ses_cookie) = req.cookie("session_token") {
                    println!("session_token found: {}", ses_cookie.value());
                    if let Some(username) = s.get_user(SessionToken{ token: ses_cookie.value().to_string()}) {
                        println!("Recognize session cookie!");
                        req.extensions_mut().insert(Username{ username: username});
                        return Ok(srv.call(req).await?);
                    }
                }
            }

            println!("Testing credentials...");
            let credentials = req.extract::<Form<Credentials>>().await?;
            if let Ok(true) = c.verify(credentials.username.clone(), credentials.password.clone()) {
                if let Ok(token) = s.new_session(credentials.username.clone()) {
                    println!("Setting session_token:{}", token.token);
                    let mut cookie = Cookie::build("session_token", token.token).finish();
                    cookie.set_max_age(Duration::MINUTE*5);
                    let mut res = srv.call(req).await?;
                    res.response_mut().add_cookie(&cookie)?;
                    Ok(res)
                }
                else {
                    Err(actix_web::error::ErrorForbidden("Failed to generate token"))
                }
            } else {
                Err(actix_web::error::ErrorForbidden("Username or Password is incorrect"))
            }
        })
    }
}