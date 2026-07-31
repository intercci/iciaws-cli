use anyhow::Result;
use http::StatusCode;
use iciaws_router::{
    addons::AddonHolder,
    input::RouteHandlerInput,
    output::RouteHandlerOutput,
    types::RouteHandler,
};
use iciaws_macros::route;
use std::pin::Pin;
use std::future::Future;

const VERSION: &str = "0.0.1"; // this can be incremented using the tool upver upon deployment

/// Handle `GET /version` — returns service name and version.
#[route("GET/version")]
pub async fn get_version(
    _input: RouteHandlerInput,
    _addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    Ok(RouteHandlerOutput::message_output(StatusCode::OK, VERSION))
}
