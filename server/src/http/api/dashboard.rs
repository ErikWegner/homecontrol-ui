use axum::{http::header, http::StatusCode, response::IntoResponse};

pub async fn dashboard_handler() -> impl IntoResponse {
    let mut path = std::env::current_dir().unwrap();
    path.push("public/dashboard.json");
    match std::fs::read_to_string(path) {
        Ok(body) => ([(header::CONTENT_TYPE, "application/json")], body).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use axum::{
        body::Body,
        http::{self, header, Request},
        routing::get,
        Router,
    };
    use http_body_util::BodyExt; // for `collect`
    use tower::ServiceExt;

    use crate::http::api::dashboard::dashboard_handler; // for `call`, `oneshot`, and `ready`

    #[tokio::test]
    async fn test_dashboard_endpoint() {
        // Build a router that uses the dashboard handler directly (no auth)
        let app = Router::new().route("/dashboard", get(dashboard_handler));

        // Create GET request
        let req = Request::builder()
            .uri("/dashboard")
            .body(Body::empty())
            .unwrap();

        // Execute the request
        let resp = app.oneshot(req).await.unwrap();

        // Check status code
        assert_eq!(resp.status(), http::StatusCode::OK);

        // Check content‑type header
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(ct.to_str().unwrap(), "application/json");

        // Read expected body from file
        let expected_body =
            fs::read_to_string("public/dashboard.json").expect("Failed to read dashboard.json");

        // Extract response body bytes
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_str, expected_body);
    }
}
