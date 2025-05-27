use std::{error, pin::Pin, rc::Rc};

use actix_web::{HttpRequest, HttpResponse};

use crate::{authenticate::Authenticate, identify::Identify, register::{Register, RegisterService}, session};

#[derive(Clone)]
pub struct Builder {
    creds: Rc<cred_auth::handler::Handler>,
    sessions: Rc<session::SessionDB>
}

impl Builder {
    pub fn new(db_path: String) -> Result<Builder, ()>
    {
        //match cred_auth::handler::Handler::new(cred_auth::handler::Storage::File(db_path.clone())) {
        match (cred_auth::handler::Handler::new(cred_auth::handler::Storage::Memory), session::SessionDB::new()) {
            (Ok(creds), Ok(sessions)) => Ok(Builder{ creds: Rc::new(creds), sessions: Rc::new(sessions) }),
            _ => Err(())
        }
    }

    pub fn auth_middleware(&self) -> Authenticate {
        Authenticate::new(self.creds.clone(), self.sessions.clone())
    }

    pub fn register_middleware(&self) -> Register {
        Register::new(self.creds.clone())
    }

    pub fn register_service(&self) -> RegisterService {
        RegisterService::new(self.creds.clone())
    }

    pub fn identify_middleware(&self) -> Identify {
        Identify::new(self.creds.clone(), self.sessions.clone())
    }
}