#![allow(dead_code)]
use iciaws_router::types::DefaultKeys;
use iciaws_router::errors::{not_found_error, bad_request_error};
use crate::common::utils::{gen_pk};
use anyhow::Result;
use iciaws_dynamo::DynamoClient;
use iciaws_dynamo::builder::Updates;
use iciaws_dynamo::pagekey::last_evaluated_key_to_base64;
use chrono::{DateTime, Utc};
use aws_sdk_dynamodb::types::AttributeValue;
use iciaws_macros::TransDynamo;
use serde::{Deserialize, Serialize};
use serde_dynamo;
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, TransDynamo)]
pub struct User {
    pub pk: Option<String>,
    pub sk: Option<String>,
    pub email: String,
    pub name: String,
    //pub pwd: Option<String>, // not neccessary if using SSO
    pub role: String,
    //pub pwd: Option<String>, // not neccessary if using SSO
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
}

impl DefaultKeys for User {
    fn set_default_keys(&mut self, from_map: &serde_json::Value) -> Result<()> {
        self.pk = Some(
            from_map
                .get("uid")
                .and_then(serde_json::Value::as_str)
                .map_or_else(|| gen_pk("User"), |uid| format!("User#{uid}")),
        );
        self.sk = Some("#".to_string());
        Ok(())
    }
}

impl User {
    pub fn makepk(uid: Option<&str>) -> String {
        uid.map_or("User".to_string(), |u| format!("User#{u}"))
    }

    pub fn makesk(uid: Option<&str>) -> String {
        uid.map_or("#".to_string(), |u| format!("#{u}"))
    }

    pub fn uid(&self) -> String {
        self.pk
            .as_deref()
            .and_then(|pk| pk.strip_prefix("User#"))
            .or_else(|| self.sk.as_deref().and_then(|sk| sk.strip_prefix('#')))
            .unwrap_or_default()
            .to_owned()
    }

    pub fn iam(&self, role: &str) -> bool {
        self.role == role
    }

    fn user_scan_params() -> (&'static str, HashMap<String, String>, HashMap<String, AttributeValue>) {
        let ean = HashMap::from([("#pk".to_string(), "pk".to_string())]);
        let eav = HashMap::from([(":pkv".to_string(), AttributeValue::S("User#".to_string()))]);
        ("begins_with(#pk, :pkv)", ean, eav)
    }

    pub async fn query_users(
        dynamo: &DynamoClient,
        last_key_str: Option<String>,
    ) -> Result<(Vec<User>, Option<String>)> {
        let (filter, ean, eav) = Self::user_scan_params();
        let qo = dynamo.scan_with_filter2(filter, ean, eav, last_key_str, None).await?;
        let last = qo.last_evaluated_key.and_then(|v| last_evaluated_key_to_base64(v).ok());
        qo.items.map_or_else(
            || Err(not_found_error("users").into()),
            |items| Ok((users_from_dynamodb(items)?, last)),
        )
    }

    pub async fn get_user(dynamo: &DynamoClient, uid: &str) -> Result<Self> {
        let pk = Self::makepk(Some(uid));
        let sk = Self::makesk(None);
        let go = dynamo.get_by_pksk(&pk, &sk, None).await?;
        tracing::info!("get_user(uid={uid}, pk={pk}, sk={sk}) returned: {go:?}");
        go.item.map_or_else(
            || Err(not_found_error("user").into()),
            |item| item.try_into(),
        )
    }

    pub async fn create_user(dynamo: &DynamoClient, data: serde_json::Value) -> Result<String> {
        let user = Self::try_from(data)?;
        let sk = user.sk.clone().unwrap_or_default();
        tracing::info!(
            "create_user returned: {:?}",
            dynamo.put_over(user.try_into()?, None).await?
        );
        Ok(sk)
    }

    pub async fn delete_user(dynamo: &DynamoClient, uid: &str) -> Result<()> {
        dynamo.delete_by_pksk(&Self::makepk(Some(uid)), &Self::makesk(None), None).await?;
        Ok(())
    }

    pub async fn update_user(
        dynamo: &DynamoClient,
        uid: &str,
        data: serde_json::Value,
        fields: &[&str],
    ) -> Result<()> {
        let pk = Self::makepk(Some(uid));
        let sk = Self::makesk(None);
        let mut builder = Updates::builder().set_pksk(Some(&pk), Some(&sk));

        let body = data
            .as_object()
            .ok_or_else(|| bad_request_error("request body must be a JSON object"))?;

        let mut changed = false;
        for key in fields {
            if let Some(value) = body.get(*key).and_then(serde_json::Value::as_str) {
                builder =
                    builder.add_update_with_value(key, &serde_json::Value::String(value.to_string()));
                changed = true;
            }
        }

        if !changed {
            return Err(bad_request_error("No attributes to update").into());
        }

        let updates = builder
            .build()
            .map_err(|_| bad_request_error("failed to build update expression"))?;

        dynamo
            .update(
                updates.key.clone(),
                updates.uex(),
                updates.ean,
                updates.eav,
                None,
            )
            .await?;
        Ok(())
    }
}
