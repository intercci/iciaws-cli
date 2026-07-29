use iciaws_router::{addons::AddonHolder, router::Router};
use lambda_http::{Body, Error, Request, Response, run, service_fn, tracing};
mod common;
mod handlers;
mod models;
mod routes;
use common::utils::TABLENAME;
use iciaws_dynamo::get_dynamo_client;
use routes::add_routes;
use iciaws_s3::get_s3_client;
use iciaws_ses::get_ses_client;
use iciaws_sns::get_sns_client;

async fn function_handler(event: Request, router: &Router) -> Result<Response<Body>, Error> {
    let resp: Response<Body> = router.route(event).await.try_into()?;
    tracing::info!(" <---- function_handler returns: {resp:?}");
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let addon_map = AddonHolder::new();
    addon_map.put_addon("dynamo", get_dynamo_client(Some(TABLENAME.clone())).await);
    addon_map.put_addon("s3", get_s3_client().await);
    addon_map.put_addon("ses", get_ses_client().await);
    addon_map.put_addon("sns", get_sns_client().await);

    let mut router = Router::new(addon_map);
    add_routes(&mut router);

    // Borrow router so the move closure below can share it across invocations.
    let router = &router;

    run(service_fn(move |event| async move {
        function_handler(event, router).await
    }))
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use lambda_http::{Body, RequestExt};

    fn test_router() -> Router {
        let mut router = Router::new(AddonHolder::new());
        add_routes(&mut router);
        router
    }

    async fn route_get(router: &Router, path: &str) -> Response<Body> {
        let request = Request::builder()
            .uri(path)
            .method(http::Method::GET)
            .body(Body::Empty)
            .unwrap()
            .with_raw_http_path(path);
        router.route(request).await.try_into().unwrap()
    }

    #[tokio::test]
    async fn test_version_endpoint_returns_200() {
        let router = test_router();
        let response = route_get(&router, "/version").await;

        assert_eq!(response.status(), 200);
        let body = String::from_utf8(response.body().to_vec()).unwrap();
        assert!(body.contains("version"), "response body: {body}");
        assert!(body.contains("Running"), "response body: {body}");
    }
}
