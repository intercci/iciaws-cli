use anyhow::Result;
use iciaws_router::{
    addons::AddonHolder,
    input::RouteHandlerInput,
    output::{ok_json, RouteHandlerOutput},
    types::RouteHandler,
};
use iciaws_macros::route;
use std::pin::Pin;
use std::future::Future;

const VERSION: &str = "0.0.1";

/// Handle `GET /version` — returns service name and version.
#[route("GET/version")]
pub async fn get_version(
    _input: RouteHandlerInput,
    _addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    let body = serde_json::json!({ "state": "Running", "version": VERSION });
    Ok(ok_json(serde_json::to_string(&body)?))
}
