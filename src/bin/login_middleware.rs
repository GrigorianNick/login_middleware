use actix_session::{storage::{self, CookieSessionStore}, Session, SessionMiddleware};
use actix_web::{
    cookie::{Key, SameSite}, error::InternalError, get, http::StatusCode, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Responder
};
use login_middleware::builder::Builder;
use rusqlite::ffi::SQLITE_OK_LOAD_PERMANENTLY;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct User {
    id: i64,
    username: String,
    password: String,
}

impl User {
    fn authenticate(credentials: Credentials) -> Result<Self, HttpResponse> {
        // to do: figure out why I keep getting hacked      /s
        if &credentials.password != "hunter2" {
            return Err(HttpResponse::Unauthorized().json("Unauthorized"));
        }

        Ok(User {
            id: 42,
            username: credentials.username,
            password: credentials.password,
        })
    }
}

pub fn validate_session(session: &Session) -> Result<i64, HttpResponse> {
    let user_id: Option<i64> = session.get("user_id").unwrap_or(None);

    match user_id {
        Some(id) => {
            // keep the user's session alive
            session.renew();
            Ok(id)
        }
        None => Err(HttpResponse::Unauthorized().json("Unauthorized")),
    }
}

async fn login(
    credentials: web::Json<Credentials>,
    session: Session,
) -> Result<impl Responder, Error> {
    let credentials = credentials.into_inner();

    match User::authenticate(credentials) {
        Ok(user) => session.insert("user_id", user.id).unwrap(),
        Err(err) => return Err(InternalError::from_response("", err).into()),
    };

    Ok("Welcome!")
}

/// some protected resource
async fn secret(session: Session) -> Result<impl Responder, Error> {
    // only allow access to this resource if the user has an active session
    validate_session(&session).map_err(|err| InternalError::from_response("", err))?;

    Ok("secret revealed")
}

#[post("/login_user")]
async fn login_success(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().body("Hello"))
}

#[post("/register_user")]
async fn register_success(req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().body("Registration successful")
}

#[get("/login")]
async fn login_page() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponseBuilder::new(StatusCode::OK)
    .content_type("text/html; charset=utf-8").body(r#"<form action="/l/login_user" method="post" enctype="application/x-www-form-urlencoded">
  <label for="username">Username:</label><br>
  <input type="text" id="username" name="username" value="Bob@gmail.com"><br>
  <label for="password">Password</label><br>
  <input type="text" id="password" name="password" value="Hunter2"><br><br>
  <input type="submit" value="Submit">
</form> "#))
}

async fn login_page2() -> impl Responder {
    HttpResponseBuilder::new(StatusCode::OK)
    .content_type("text/html; charset=utf-8").body(r#"<form action="/l/login_user" method="post" enctype="application/x-www-form-urlencoded">
  <label for="username">Username:</label><br>
  <input type="text" id="username" name="username" value="Bob@gmail.com"><br>
  <label for="password">Password</label><br>
  <input type="text" id="password" name="password" value="Hunter2"><br><br>
  <input type="submit" value="Submit">
</form> "#)
}


#[get("/register")]
async fn register_page() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponseBuilder::new(StatusCode::OK)
    .content_type("text/html; charset=utf-8").body(r#"<form action="/r/register_user" method="post" enctype="application/x-www-form-urlencoded">
  <label for="username">Username:</label><br>
  <input type="text" id="username" name="username" value="Bob@gmail.com"><br>
  <label for="password">Password</label><br>
  <input type="text" id="password" name="password" value="Hunter2"><br><br>
  <input type="submit" value="Submit">
</form> "#))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // The signing key would usually be read from a configuration file/environment variables.
    let signing_key = Key::generate();


    HttpServer::new(move || {
        let builder = Builder::new(String::from("LoginTest.db")).expect("Failed to generate builder");

        let login_scope = web::scope("/l")
            .service(login_success)
            .wrap(builder.auth_middleware());

        let register_scope = web::scope("/r")
            .service(register_success)
            .wrap(builder.register_middleware());

        App::new()
        .service(
            web::resource("/test")
                .get(login_page2)
                .post(login_page2)
        )
        .service(login_scope)
        .service(register_scope)
        .service(login_page)
        .service(register_page)
        .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}