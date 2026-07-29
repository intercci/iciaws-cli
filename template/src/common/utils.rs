#![allow(dead_code)]
use chrono::{Datelike, Duration, Utc};
use nanoid::nanoid;
use std::env;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Environment variables (LazyLock statics)
// ---------------------------------------------------------------------------

// static API_KEY: LazyLock<String> = LazyLock::new(|| {
//     env::var("API_KEY").expect("API_KEY environment variable must be set")
// });

pub static TABLENAME: LazyLock<String> =
    LazyLock::new(|| env::var("TABLE_NAME").unwrap_or_else(|_| "ici-users".to_string()));

pub static SITE: LazyLock<String> =
    LazyLock::new(|| env::var("SITE").unwrap_or_else(|_| "ICI".to_string()));

pub static DOMAIN: LazyLock<String> =
    LazyLock::new(|| env::var("DOMAIN").unwrap_or_else(|_| "dev.intercci.com".to_string()));

pub static COOKIE_DOMAIN: LazyLock<Option<String>> =
    LazyLock::new(|| env::var("COOKIE_DOMAIN").ok());

pub static FROM: LazyLock<String> =
    LazyLock::new(|| env::var("FROM").unwrap_or_else(|_| "admin@intercci.com".to_string()));

pub static RESET_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    env::var("RESET_TEMPLATE").unwrap_or_else(|_| "ResetTemplateDev".to_string())
});

pub static IMAGE_BUCKET: LazyLock<String> =
    LazyLock::new(|| env::var("IMAGE_BUCKET").unwrap_or_else(|_| "images".to_string()));

pub static IMAGE_FOLDER: LazyLock<String> =
    LazyLock::new(|| env::var("IMAGE_FOLDER").unwrap_or_else(|_| "modules/images/".to_string()));

pub static DEFAULT_HOST_EMAIL: LazyLock<String> =
    LazyLock::new(|| env::var("HOST_EMAIL").unwrap_or_else(|_| "admin@intercci.com".to_string()));

pub const MODULE_INDEX: &'static str = "GSI1Index";

// ---------------------------------------------------------------------------
// Timestamp helpers
// ---------------------------------------------------------------------------

/// Return a timestamp in seconds.
pub fn timestamp_secs() -> i64 {
    Utc::now().timestamp()
}

/// Return a timestamp in milliseconds.
pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn timextamp_short() -> String {
    format!("{:x}", Utc::now().timestamp())
}

pub fn timextamp_long() -> String {
    format!("{:x}", Utc::now().timestamp_millis())
}

// ---------------------------------------------------------------------------
// DynamoDB key helpers
// ---------------------------------------------------------------------------

pub fn gen_pk(prefix: &str) -> String {
    format!("{prefix}#{}", timextamp_short())
}

/// Make a pk string in the pattern `<prefix>#<id>`.
pub fn make_pk(prefix: &str, id: &str) -> String {
    format!("{}#{}", prefix, id)
}

/// Extract the id portion after `#` from a pk string.
///
/// If the input contains no `#`, it is returned as-is.
///
/// # Example
/// ```
/// assert_eq!(extract_id("User#1234"), "1234");
/// assert_eq!(extract_id("a123"), "a123");
/// ```
pub fn extract_id(pk_or_id: &str) -> String {
    match pk_or_id.split_once('#') {
        Some((_, id)) => id.to_string(),
        None => pk_or_id.to_string(),
    }
}

// ---------------------------------------------------------------------------
// ID generation
// ---------------------------------------------------------------------------

/// Generate a nano-id with `n` characters.
///
/// # Example
/// ```
/// let sid = gen_nanoid(5);
/// assert_eq!(sid.len(), 5);
/// ```
pub fn gen_nanoid(n: usize) -> String {
    nanoid!(n)
}

// ---------------------------------------------------------------------------
// Date / expiry helpers
// ---------------------------------------------------------------------------

pub fn expires_in(hours: i64) -> i64 {
    (Utc::now() + Duration::hours(hours)).timestamp()
}

pub fn this_year() -> String {
    Utc::now().year().to_string()
}
