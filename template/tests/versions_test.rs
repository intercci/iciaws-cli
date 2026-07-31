use iciaws_test_helper::{call_router, create_get, create_router, fmt_error, StatusValue};
use __PROJECT_NAME__::routes::add_routes;
use regex::Regex;

#[tokio::test]
async fn test_get_version() {
    let mut router = create_router("ici-users").await;
    add_routes(&mut router);
    let req = create_get("/version");
    let StatusValue { status, value } = call_router(&router, req).await.unwrap();
    assert_eq!(status, http::StatusCode::OK, "{}", fmt_error(&value));
    assert_eq!(value["status"].as_str(), Some("OK"));
    let vp = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    let ver = value["message"].as_str().unwrap();
    assert!(vp.is_match(ver));
}
