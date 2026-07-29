#![allow(dead_code)]
use crate::models::user::User;
use anyhow::Result;
use iciaws_dynamo::DynamoClient;
use iciaws_router::{
    addons::AddonHolder,
    errors::bad_request_error,
    input::RouteHandlerInput,
    output::{
        get_failed, get_ok, item_created, item_deleted, item_updated, query_ok, query_page_ok,
        RouteHandlerOutput,
    },
    types::RouteHandler,
};
use iciaws_macros::route;
use tracing::instrument;
use std::pin::Pin;
use std::future::Future;

#[instrument(skip(addons))]
#[route("GET/users")]
pub async fn query_users(
    input: RouteHandlerInput,
    addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    let last: Option<String> = input.get_query_value("last").ok();

    let dynamo = addons.get::<&DynamoClient>("dynamo")?;
    let rs = User::query_users(dynamo, last).await;
    match rs {
        Ok((users, last)) => query_page_ok("users", users, last),
        Err(_) => query_ok("users", vec![] as Vec<User>),
    }
}

#[instrument(skip(addons))]
#[route("GET/users/{uid}")]
pub async fn get_user(
    input: RouteHandlerInput,
    addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    let uid = input.get_path_value("uid")?;
    // let me = input.get_claim("uid")?; // me == uid or me is host or admin
    let dynamo = addons.get::<&DynamoClient>("dynamo")?;
    let usr = User::get_user(dynamo, &uid).await;
    match usr {
        Ok(user) => get_ok("user", user),
        Err(_) => get_failed("user"),
    }
}

#[instrument(skip(addons))]
#[route("POST/users")]
pub async fn create_user(
    input: RouteHandlerInput,
    addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    let dynamo = addons.get::<&DynamoClient>("dynamo")?;
    let body = input.body.ok_or_else(|| bad_request_error("request body required"))?;
    let uid = User::create_user(dynamo, body).await?;
    item_created("user", "uid", &uid)
}

#[instrument(skip(addons))]
#[route("PUT/users/{uid}")]
pub async fn update_user(
    input: RouteHandlerInput,
    addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    let uid = input.get_path_value("uid")?;
    let dynamo = addons.get::<&DynamoClient>("dynamo")?;
    let body = input.body.ok_or_else(|| bad_request_error("request body required"))?;
    let fields = ["name", "given_name", "family_name", "title", "role"];
    User::update_user(dynamo, &uid, body, &fields).await?;
    item_updated("user")
}

#[instrument(skip(addons))]
#[route("DELETE/users/{uid}")]
pub async fn delete_user(
    input: RouteHandlerInput,
    addons: &AddonHolder,
) -> Result<RouteHandlerOutput> {
    let uid = input.get_path_value("uid")?;
    let dynamo = addons.get::<&DynamoClient>("dynamo")?;
    let _ = User::delete_user(dynamo, &uid).await?;
    item_deleted("user")
}
