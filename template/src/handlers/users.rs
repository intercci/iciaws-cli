#![allow(dead_code)]
use crate::{
    models::user::User,
};
use anyhow::Result;
use iciaws_dynamo::DynamoClient;
use iciaws_router::{
    addons::AddonHolder,
    errors::{not_found_error, unauthorized_error},
    input::RouteHandlerInput,
    output::{RouteHandlerOutput, get_ok, query_page_ok, get_failed, item_created, item_updated, item_deleted},
    types::RouteHandler,
};
use iciaws_macros::route;
use lambda_http::tracing;
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
    let uid = User::create_user(dynamo, input.body.unwrap()).await?;
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
    let fields = ["name", "given_name", "family_name", "title", "role"];
    let _ = User::update_user(dynamo, &uid, input.body.unwrap(), &fields).await?;
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
