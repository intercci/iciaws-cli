#![allow(dead_code)]
use chrono::{Datelike, Duration, Utc};
use nanoid::nanoid;
use std::env;
use std::sync::LazyLock;

// static API_KEY: LazyLock<String> = LazyLock::new(|| {
//     env::var("API_KEY").expect("API_KEY environment variable must be set")
// });

pub static TABLENAME: LazyLock<String> =
    LazyLock::new(|| env::var("TABLE_NAME").unwrap_or_else(|_| "ici-packman".to_string()));

pub static SITE: LazyLock<String> =
    LazyLock::new(|| env::var("SITE").unwrap_or_else(|_| "ICI".to_string()));

pub static DOMAIN: LazyLock<String> =
    LazyLock::new(|| env::var("DOMAIN").unwrap_or_else(|_| "packman-dev.intercci.com".to_string()));

pub static COOKIE_DOMAIN: LazyLock<Option<String>> =
    LazyLock::new(|| env::var("COOKIE_DOMAIN").ok());

pub static FROM: LazyLock<String> =
    LazyLock::new(|| env::var("FROM").unwrap_or_else(|_| "admin@intercci.com".to_string()));

pub static INVITE_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    env::var("INVITE_TEMPLATE").unwrap_or_else(|_| "PackmanInviteTemplateDev".to_string())
});

pub static RESET_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    env::var("RESET_TEMPLATE").unwrap_or_else(|_| "PackmanResetTemplateDev".to_string())
});

pub static SUBMIT_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    env::var("SUBMIT_TEMPLATE").unwrap_or_else(|_| "PackmanSubmitTemplateDev".to_string())
});

pub static COMMENT_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    env::var("COMMENT_TEMPLATE").unwrap_or_else(|_| "PackmanCommentTemplateDev".to_string())
});

pub static IMAGE_BUCKET: LazyLock<String> =
    LazyLock::new(|| env::var("IMAGE_BUCKET").unwrap_or_else(|_| "packman-dev".to_string()));

pub static IMAGE_FOLDER: LazyLock<String> =
    LazyLock::new(|| env::var("IMAGE_FOLDER").unwrap_or_else(|_| "modules/images/".to_string()));

pub static DEFAULT_HOST_EMAIL: LazyLock<String> =
    LazyLock::new(|| env::var("HOST_EMAIL").unwrap_or_else(|_| "admin@intercci.com".to_string()));

pub const MODULE_INDEX: &'static str = "GSI1Index";

/// Return a timestamp in seconds.
pub fn timestamp_secs() -> i64 {
    let now = Utc::now();
    now.timestamp()
}

/// Return a timestamp in milliseconds.
pub fn timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Return a hex string of current timestamp in seconds.
///
/// # Example
///
/// ```rust
/// use crate::common::utils::timextamp_short;
/// assert!(timextamp_short().len() == 8);
/// ```
pub fn timextamp_short() -> String {
    format!("{:x}", timestamp_secs())
}

/// Return a hex string of current timestamp in milli-seconds.
///
/// # Example
///
/// ```rust
/// use crate::common::utils::timextamp_long;
/// assert!(timextamp_long().len() == 8);
/// ```
pub fn timextamp_long() -> String {
    format!("{:x}", timestamp_millis())
}

/// Generate a string with a prefix for DynamoDB primary key string.
///
/// # Arguments
///
/// * `prefix` - a prefix string like 'User'
///
/// # Returns
///
/// Returns a string with the prefix, a hash and a random timestamp in hex format.
///
/// # Example
///
/// ```rust
/// use crate::common::utils::gen_pk;
///
/// let pk = gen_pk("Client");
/// assert!(pk.starts_with("Client#"));
/// ```
pub fn gen_pk(prefix: &str) -> String {
    format!("{}#{}", prefix, timextamp_short())
}

/// Make a pk string in the pattern <prefix>#<id>
pub fn make_pk(prefix: &str, id: &str) -> String {
    format!("{}#{}", prefix, id)
}

/// Break a pk that contains # and returns the second part which should be the id
/// (Note this does not apply to the pk strings with more than one hash)
/// And if the pk_or_id does not contain a hash, it's regarded as the id itself.
///
/// #Example
///
/// ```rust
/// use crate::common::utils::extract_id;
/// assert_eq!(extract_id("User#1234"), "1234");
/// assert_eq!(extract_id("a123"), "a123");
/// ```
pub fn extract_id(pk_or_id: &str) -> String {
    if pk_or_id.contains("#") {
        let pk_and_id: Vec<&str> = pk_or_id.split('#').collect();
        pk_and_id[1].to_string()
    } else {
        pk_or_id.to_string()
    }
}

/// Generate a nano-id with n characters, suitable for invite code, etc. '86vQAx3DOm'
///
/// # Arguments
///
/// * `n` - how many characters to generate as the short ID string
///
/// # Returns
///
/// A short string with random letters and digits.
///
/// # Examples
///
/// ```rust
/// use crate::common::utils::gen_nanoid;
///
/// let sid = gen_nanoid(5);
/// assert_eq!(len(sid), 5);
/// ```
pub fn gen_nanoid(n: Option<usize>) -> String {
    if n.is_none() {
        nanoid!()
    } else {
        let x = n.unwrap();
        nanoid!(x)
    }
}

/// Returns a timestamp in seconds as an int64 by adding hours to current time.
///
/// # Arguments
///
/// * `hours` - number of hours to add to get a future timestamp
///
/// # Returns
///
/// Returns a `i64` number for timestamp as seconds.
///
/// # Examples
///
/// ```rust
/// use chrono::Utc;
/// use crate::common::utils::expires_in;
///
/// let now = Utc::now();
/// let ft = expires_in(1);
/// assert!(ft >= now.timestamp());
/// ```
pub fn expires_in(hours: i64) -> i64 {
    let now = Utc::now();
    let fut = now + Duration::hours(hours);
    fut.timestamp()
}

/// Returns the current year as a string like '2026'
///
pub fn this_year() -> String {
    let d = Utc::now();
    format!("{}", d.year())
}
