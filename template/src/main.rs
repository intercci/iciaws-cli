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
    let rs = router.route(event).await;
    let resp = rs.try_into()?;
    tracing::info!(" <---- function_handler returns: {:?}", resp);
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let tablename = TABLENAME.clone();
    let dynamo_client = get_dynamo_client(Some(tablename)).await;
    let s3_client = get_s3_client().await;
    let ses_client = get_ses_client().await;
    let sns_client = get_sns_client().await;

    let addon_map = AddonHolder::new();

    addon_map.put_addon("dynamo", dynamo_client);
    addon_map.put_addon("s3", s3_client);
    addon_map.put_addon("ses", ses_client);
    addon_map.put_addon("sns", sns_client);

    let mut router = Router::new(addon_map);
    add_routes(&mut router);

    let router_ref = &router;

    run(service_fn(move |event| async move {
        function_handler(event, router_ref).await
    }))
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use iciaws_router::addons::AddonHolder;
    use http::Request;
    use lambda_http::Body;

    #[tokio::test]
    async fn test_basic_handler() {
        let addon_map = AddonHolder::new();
        let mut router = Router::new(addon_map);
        add_routes(&mut router);
        
        use lambda_http::RequestExt;
        let request = Request::builder()
            .uri("/version")
            .method(http::Method::GET)
            .body(Body::Empty)
            .unwrap();
        
        let req = request.with_raw_http_path("/version");
        let result = router.route(req).await;
        let response: Response<Body> = result.try_into().unwrap();
        
        assert_eq!(response.status(), 200);
        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();
        assert!(body_string.contains("version"));
        assert!(body_string.contains("Running"));
    }
}
