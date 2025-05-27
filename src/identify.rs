use std::{future::{ready, Ready}, rc::Rc};

use actix_web::{cookie::{time::Duration, Cookie}, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, web::Form, Error, HttpMessage};
use futures_util::future::LocalBoxFuture;

use crate::{authenticate::Username, credentials::Credentials, session::{SessionDB, SessionToken}};

pub struct Identify
{
    creds: Rc<cred_auth::handler::Handler>,
    sessions: Rc<SessionDB>
}

impl Identify {
    pub fn new(creds: Rc<cred_auth::handler::Handler>, sessions: Rc<SessionDB>) -> Identify {
        Identify { creds: creds, sessions: sessions }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Identify
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = IdentifyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(IdentifyMiddleware::new(service, self.creds.clone(), self.sessions.clone())))
    }
}

pub struct IdentifyMiddleware<S>
{
    service: Rc<S>,
    creds: Rc<cred_auth::handler::Handler>,
    sessions: Rc<SessionDB>
}

impl<S> IdentifyMiddleware<S>
{
    pub(crate) fn new(service: S, creds: Rc<cred_auth::handler::Handler>, sessions: Rc<SessionDB>) -> IdentifyMiddleware<S>
    {
      IdentifyMiddleware { service: Rc::new(service), creds: creds, sessions: sessions }  
    }
}

impl<S, B> Service<ServiceRequest> for IdentifyMiddleware<S>
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
            {
                if let Some(ses_cookie) = req.cookie("session_token") {
                    if let Some(username) = s.get_user(SessionToken{ token: ses_cookie.value().to_string()}) {
                        req.extensions_mut().insert(Username{ username: username});
                        return Ok(srv.call(req).await?);
                    }
                }
            }

            // No session token, see if we recognize the user and generate a session token for them
            let credentials = req.extract::<Form<Credentials>>().await?;
            if let Ok(true) = c.verify(credentials.username.clone(), credentials.password.clone()) {
                if let Ok(token) = s.new_session(credentials.username.clone()) {
                    let mut cookie = Cookie::build("session_token", token.token).finish();
                    cookie.set_max_age(Duration::MINUTE*5);
                    let mut res = srv.call(req).await?;
                    res.response_mut().add_cookie(&cookie)?;
                    return Ok(res);
                }
            }

            Ok(srv.call(req).await?)
        })
    }
}
