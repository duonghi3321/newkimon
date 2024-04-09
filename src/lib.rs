//!
//! Kantan is a library for writing responses to servers.
//!
//!  * You can spin up a `Server` within a test.
//!  * Create requests that will run against that.
//!  * Retrieve what they happen to return.
//!  * Assert that the response works how you expect.
//!
//! It icludes built in suppot with Serde, Cookies,
//! and other common crates for working with the web.
//!
//! ## Getting Started
//!
//! In essence; create your Axum application, create a `Server`,
//! and then make requests against it.
//!
//! ```rust
//! # ::tokio_test::block_on(async {
//! use ::axum::Router;
//! use ::axum::extract::Json;
//! use ::axum::routing::put;
//! use ::axum_test::Server;
//! use ::serde_json::json;
//! use ::serde_json::Value;
//!
//! async fn put_user(Json(user): Json<Value>) -> () {
//!     // todo
//! }
//!
//! let my_app = Router::new()
//!     .route("/users", put(put_user))
//!     .into_make_service();
//!
//! let server = Server::new(my_app)
//!     .unwrap();
//!
//! let response = server.put("/users")
//!     .json(&json!({
//!         "username": "Terrance Pencilworth",
//!     }))
//!     .await;
//! # })
//! ```
//!
//! ## Features
//!
//! ### Auto Cookie Saving 🍪
//!
//! When you build a `Server`, you can turn on a feature to automatically save cookies
//! across requests. This is used for automatically saving things like session cookies.
//!
//! ```rust
//! # ::tokio_test::block_on(async {
//! use ::axum::Router;
//! use ::axum_test::Server;
//! use ::axum_test::ServerConfig;
//!
//! let my_app = Router::new()
//!     .into_make_service();
//!
//! let config = ServerConfig {
//!     save_cookies: true,
//!     ..ServerConfig::default()
//! };
//! let server = Server::new_with_config(my_app, config)
//!     .unwrap();
//! # })
//! ```
//!
//! Then when you make a request, any cookies that are returned will be reused
//! by the next request. This is on a per server basis (it doesn't save across servers).
//!
//! You can turn this on or off per request, using `Request::do_save_cookies'
//! and Request::do_not_save_cookies'.
//!
//! ### Content Type 📇
//!
//! When performing a request, it will start with no content type at all.
//!
//! You can set a default type for all `Request` objects to use,
//! by setting the `default_content_type` in the `ServerConfig`.
//! When creating the `Server` instance, using `new_with_config`.
//!
//! ```rust
//! # ::tokio_test::block_on(async {
//! use ::axum::Router;
//! use ::axum_test::Server;
//! use ::axum_test::ServerConfig;
//!
//! let my_app = Router::new()
//!     .into_make_service();
//!
//! let config = ServerConfig {
//!     default_content_type: Some("application/json".to_string()),
//!     ..ServerConfig::default()
//! };
//!
//! let server = Server::new_with_config(my_app, config)
//!     .unwrap();
//! # })
//! ```
//!
//! If there is no default, then a `Request` will try to guess the content type.
//! Such as setting `application/json` when calling `Request::json`,
//! and `text/plain` when calling `Request::text`.
//! This will never override any default content type provided.
//!
//! Finally on each `Request`, one can set the content type to use.
//! By calling `Request::content_type` on it.
//!
//! ```rust
//! # ::tokio_test::block_on(async {
//! use ::axum::Router;
//! use ::axum::extract::Json;
//! use ::axum::routing::put;
//! use ::kantan::Server;
//! use ::serde_json::json;
//! use ::serde_json::Value;
//!
//! async fn put_user(Json(user): Json<Value>) -> () {
//!     // todo
//! }
//!
//! let my_app = Router::new()
//!     .route("/users", put(put_user))
//!     .into_make_service();
//!
//! let server = Server::new(my_app)
//!     .unwrap();
//!
//! let response = server.put("/users")
//!     .content_type(&"application/json")
//!     .json(&json!({
//!         "username": "Terrance Pencilworth",
//!     }))
//!     .await;
//! # })
//! ```
//!
//! ### Fail Fast
//!
//! This library is written to panic quickly. For example by default a response will presume to
//! succeed and will panic if they don't (which you can change).
//! Functions to retreive cookies and headers will by default panic if they aren't found.
//!
//! This behaviour is unorthodox for Rust, however it is intentional to aid with writing tests.
//! Where you want the test to fail as quickly, and skip on writing error handling code.
//!

mod server;
pub use self::server::*;

mod request;
pub use self::request::*;

mod response;
pub use self::response::*;

pub use ::hyper::http;

#[cfg(test)]
mod test_get {
    use super::*;

    use ::axum::routing::get;
    use ::axum::Router;
    use ::axum_test::TestServer;

    async fn get_ping() -> &'static str {
        "pong!"
    }

    #[tokio::test]
    async fn it_sound_get() {
        // Build an application with a route.
        let app = Router::new()
            .route("/ping", get(get_ping))
            .into_make_service();

        // Run the server.
        let test_server = TestServer::new(app).expect("Should create test server");
        let server_address = test_server.server_address();

        // Get the request.
        let server = Server::new(server_address).expect("Should create server");
        let text = server.get(&"/ping").await.text();

        assert_eq!(text, "pong!");
    }
}

#[cfg(test)]
mod test_content_type {
    use super::*;

    use ::axum::http::header::CONTENT_TYPE;
    use ::axum::http::HeaderMap;
    use ::axum::routing::get;
    use ::axum::Router;
    use ::axum_test::TestServer;

    async fn get_content_type(headers: HeaderMap) -> String {
        headers
            .get(CONTENT_TYPE)
            .map(|h| h.to_str().unwrap().to_string())
            .unwrap_or_else(|| "".to_string())
    }

    #[tokio::test]
    async fn it_should_not_set_a_content_type_by_default() {
        // Build an application with a route.
        let app = Router::new()
            .route("/content_type", get(get_content_type))
            .into_make_service();

        // Run the server.
        let test_server = TestServer::new(app).expect("Should create test server");
        let server_address = test_server.server_address();

        // Get the request.
        let server = Server::new(server_address).expect("Should create server");
        let text = server.get(&"/content_type").await.text();

        assert_eq!(text, "");
    }

    #[tokio::test]
    async fn it_should_set_content_type_when_present() {
        // Build an application with a route.
        let app = Router::new()
            .route("/content_type", get(get_content_type))
            .into_make_service();

        // Run the server.
        let test_server = TestServer::new(app).expect("Should create test server");
        let server_address = test_server.server_address();

        // Get the request.
        let server = Server::new(server_address).expect("Should create server");
        let response = server
            .get(&"/content_type")
            .content_type(&"application/json")
            .await;

        assert_eq!(response.status_code(), ::hyper::StatusCode::OK);
        let text = response.text();
        assert_eq!(text, "application/json");
    }
}

#[cfg(test)]
mod test_cookies {
    use super::*;

    use ::axum::extract::RawBody;
    use ::axum::routing::get;
    use ::axum::routing::put;
    use ::axum::Router;
    use ::axum_extra::extract::cookie::Cookie as AxumCookie;
    use ::axum_extra::extract::cookie::CookieJar;
    use ::axum_test::TestServer;
    use ::hyper::body::to_bytes;

    const TEST_COOKIE_NAME: &'static str = &"test-cookie";

    async fn get_cookie(cookies: CookieJar) -> (CookieJar, String) {
        let cookie = cookies.get(&TEST_COOKIE_NAME);
        let cookie_value = cookie
            .map(|c| c.value().to_string())
            .unwrap_or_else(|| "cookie-not-found".to_string());

        (cookies, cookie_value)
    }

    async fn put_cookie(
        mut cookies: CookieJar,
        RawBody(body): RawBody,
    ) -> (CookieJar, &'static str) {
        let body_bytes = to_bytes(body)
            .await
            .expect("Should turn the body into bytes");
        let body_text: String = String::from_utf8_lossy(&body_bytes).to_string();
        let cookie = AxumCookie::new(TEST_COOKIE_NAME, body_text);
        cookies = cookies.add(cookie);

        (cookies, &"done")
    }

    #[tokio::test]
    async fn it_should_not_pass_cookies_created_back_up_to_server_by_default() {
        // Build an application with a route.
        let app = Router::new()
            .route("/cookie", put(put_cookie))
            .route("/cookie", get(get_cookie))
            .into_make_service();

        // Run the server.
        let test_server = TestServer::new(app).expect("Should create test server");
        let server_address = test_server.server_address();

        // Get the request.
        let server = Server::new(server_address).expect("Should create server");
        server.put(&"/cookie").text(&"new-cookie").await;

        // Check it comes back.
        let response_text = server.get(&"/cookie").await.text();

        assert_eq!(response_text, "cookie-not-found");
    }

    #[tokio::test]
    async fn it_should_pass_cookies_created_back_up_to_server_when_turned_on_for_request() {
        // Build an application with a route.
        let app = Router::new()
            .route("/cookie", put(put_cookie))
            .route("/cookie", get(get_cookie))
            .into_make_service();

        // Run the server.
        let test_server = TestServer::new(app).expect("Should create test server");
        let server_address = test_server.server_address();

        // Create a cookie.
        let server = Server::new(server_address).expect("Should create server");
        server
            .put(&"/cookie")
            .text(&"cookie-found!")
            .do_save_cookies()
            .await;

        // Check it comes back.
        let response_text = server.get(&"/cookie").await.text();

        assert_eq!(response_text, "cookie-found!");
    }
}
