#[macro_use]
extern crate lazy_static;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use tokio_postgres::error::Error as TokioPostgresError;
use tokio_postgres::{Client, NoTls};

enum Handler {
    PostUsers,
    GetTokens,
    PostTokens,
    GetTokensCurrent,
    DeleteTokensCurrent,
    PostTokensCurrentRefresh,
    GetTokensCurrentValid,
    GetTokensId,
    DeleteTokensId,
}

struct Route {
    auth_required: bool,
    handler: Handler,
}

lazy_static! {
    static ref TOKEN_SECRET_REGEX: regex::Regex = regex::Regex::new("^[a-fA-F0-9]{64}$").unwrap();
    static ref TOKENS_ID_PATH_REGEX: regex::Regex = regex::Regex::new("^/tokens/[a-fA-F0-9]{32}$").unwrap();
}

fn get_route(request: &Request<Body>) -> Result<Route, Response<Body>> {
    println!("get_route");

    let orig_path = request.uri().path();

    let path = if orig_path.ends_with('/') {
        &orig_path[..orig_path.len() - 1]
    } else {
        orig_path
    };

    let mut path_found = true;

    match path {
        "/users" => match *request.method() {
            Method::POST => return Ok(Route { auth_required: false, handler: Handler::PostUsers }),
            _ => ()
        },
        "/tokens" => match *request.method() {
            Method::GET => return Ok(Route { auth_required: true, handler: Handler::GetTokens }),
            Method::POST => return Ok(Route { auth_required: false, handler: Handler::PostTokens }),
            _ => ()
        },
        "/tokens/current" => match *request.method() {
            Method::GET => return Ok(Route { auth_required: true, handler: Handler::GetTokensCurrent }),
            Method::DELETE => return Ok(Route { auth_required: true, handler: Handler::DeleteTokensCurrent }),
            _ => ()
        },
        "/tokens/current/refresh" => match *request.method() {
            Method::POST => return Ok(Route { auth_required: true, handler: Handler::PostTokensCurrentRefresh }),
            _ => ()
        },
        "/tokens/current/valid" => match *request.method() {
            Method::GET => return Ok(Route { auth_required: true, handler: Handler::GetTokensCurrentValid }),
            _ => ()
        },
        _ => path_found = false
    };

    if !path_found && TOKENS_ID_PATH_REGEX.is_match(path) {
        path_found = true;

        match *request.method() {
            Method::GET => return Ok(Route { auth_required: true, handler: Handler::GetTokensId }),
            Method::DELETE => return Ok(Route { auth_required: true, handler: Handler::DeleteTokensId }),
            _ => ()
        };
    }

    if path_found {
        return Err(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("method not allowed\n"))
                .unwrap())
    }

    Err(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("resource not found\n"))
        .unwrap())
}

async fn handle_anonymous_request(handler: Handler, request: Request<Body>, db: &Client) -> Response<Body> {
    match handler {
        Handler::PostUsers => post_users(request, db).await,
        Handler::PostTokens => post_tokens(request, db).await,
        _ => Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("service unavailable\n"))
                .unwrap()
    }
}

async fn post_users(request: Request<Body>, db: &Client) -> Response<Body> {
    Response::builder()
        .body(Body::from("POST Users!\n"))
        .unwrap()
}

async fn post_tokens(request: Request<Body>, db: &Client) -> Response<Body> {
    Response::builder()
        .body(Body::from("POST Tokens!\n"))
        .unwrap()
}

async fn handle_authenticated_request(handler: Handler, request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    match handler {
        Handler::GetTokens => get_tokens(request, db, user_id).await,
        Handler::GetTokensCurrent => get_tokens_current(request, db, user_id).await,
        Handler::DeleteTokensCurrent => delete_tokens_current(request, db, user_id).await,
        Handler::PostTokensCurrentRefresh => post_tokens_current_refresh(request, db, user_id).await,
        Handler::GetTokensCurrentValid => get_tokens_current_valid(request, db, user_id).await,
        Handler::GetTokensId => get_tokens_id(request, db, user_id).await,
        Handler::DeleteTokensId => delete_tokens_id(request, db, user_id).await,
        _ => Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("service unavailable\n"))
                .unwrap()
    }
}

async fn get_tokens(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("get_tokens {}\n", user_id)))
        .unwrap()
}

async fn get_tokens_current(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("get_tokens_current {}\n", user_id)))
        .unwrap()
}

async fn delete_tokens_current(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("delete_tokens_current {}\n", user_id)))
        .unwrap()
}

async fn post_tokens_current_refresh(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("post_tokens_current_refresh {}\n", user_id)))
        .unwrap()
}

async fn get_tokens_current_valid(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("get_tokens_current_valid {}\n", user_id)))
        .unwrap()
}

async fn get_tokens_id(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("get_tokens_id {}\n", user_id)))
        .unwrap()
}

async fn delete_tokens_id(request: Request<Body>, db: &Client, user_id: i32) -> Response<Body> {
    Response::builder()
        .body(Body::from(format!("delete_tokens_id {}\n", user_id)))
        .unwrap()
}


#[tokio::main]
async fn main() {
    println!("main");
    let pg_mgr = PostgresConnectionManager::new_from_stringlike(
        "postgresql://auth:auth@localhost:5432",
        NoTls,
    )
    .unwrap();

    let pool = match Pool::builder().build(pg_mgr).await {
        Ok(pool) => pool,
        Err(e) => panic!("bb8 error {:?}", e),
    };

    let make_svc = make_service_fn(move |_socket| {
        let pool = pool.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |request: Request<_>| {
                let pool = pool.clone();
                async move { process_request(request, pool).await }
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server until CTRL+C
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    } else {
        println!("\ngracefully shutdown");
    }
}

async fn process_request(
    request: Request<Body>,
    pool: Pool<PostgresConnectionManager<NoTls>>,
) -> Result<Response<Body>, Infallible> {
    println!("process_request");
    // do anything that doesn't need DB here before pool.run
    // (routing, token cookie presence/format)
    let route = get_route(&request);
    if route.is_err() {
        return Ok(route.err().unwrap());
    }
    let route = route.unwrap();

    let mut token_secret_option = None;
    if route.auth_required {
        match get_token_secret(&request) {
            Ok(secret) => token_secret_option = Some(secret),
            Err(response) => return Ok(response),
        }
    }

    let result = pool.run(move |db| {
        async move {
            let response = if let Some(token_secret) = token_secret_option {
                match get_user_id(token_secret, &db).await {
                    Ok(id) => handle_authenticated_request(route.handler, request, &db, id).await,
                    Err(e) => e
                }
            } else {
                handle_anonymous_request(route.handler, request, &db).await
            };

            println!("send response\n");
            Ok::<_, (TokioPostgresError, Client)>((response, db))
        }
    }).await;

    Ok(match result {
        Ok(response) => response,
        Err(e) => {
            println!("TokioPostgresError: {}", e);
            Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("service unavailable\n"))
                .unwrap()
        }
    })
}

async fn get_user_id(token_secret: String, db: &Client) -> Result<i32, Response<Body>> {
    println!("get_user_id");
    let rows = db.query(
        "SELECT identity_id FROM token_active WHERE secret = $1",
        &[&token_secret],
    ).await.unwrap();

    if rows.len() != 1 {
        return Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("invalid or expired token\n"))
            .unwrap());
    }

    let user_id: i32 = rows.get(0).unwrap().get("identity_id");

    Ok(user_id)
}

fn get_token_secret(request: &Request<Body>) -> Result<String, Response<Body>> {
    println!("get_token_secret");
    let mut token_secret_option = None;
    let mut found_token_cookie = false;

    for cookie_header in request.headers().get_all("cookie") {
        for cookie in cookie_header.to_str().unwrap().split(';') {
            let cookie_pair: Vec<&str> = cookie.split('=').collect();

            if cookie_pair.len() != 2 || cookie_pair.get(0).unwrap().trim() != "token" {
                continue;
            }

            let token_secret = cookie_pair.get(1).unwrap().trim();
            found_token_cookie = true;

            if !TOKEN_SECRET_REGEX.is_match(token_secret) {
                continue;
            }

            match token_secret_option {
                Some(first_token_secret) => {
                    if first_token_secret != token_secret {
                        return Err(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from("multiple token cookies\n"))
                            .unwrap());
                    }
                }
                None => token_secret_option = Some(token_secret),
            }
        }
    }

    match token_secret_option {
        Some(token_secret) => Ok(token_secret.to_string()),
        None => Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(if found_token_cookie {
                "token cookie invalid format\n"
            } else {
                "missing token cookie\n"
            }))
            .unwrap()),
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c().await
        .expect("failed to install CTRL+C signal handler");
}