#![allow(dead_code)]
use iciaws_router::types::DefaultKeys;
use crate::common::utils::{gen_pk};
use anyhow::Result;
use iciaws_dynamo::DynamoClient;
use aws_sdk_dynamodb::types::AttributeValue;
use iciaws_macros::TransDynamo;
use serde::{Deserialize, Serialize};
use serde_dynamo;
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, TransDynamo)]
pub struct User {
    pub pk: Option<String>, // User#<UID>
    pub sk: Option<String>, // #
    pub email: String,
    pub name: String,
    //pub pwd: Option<String>, // not neccessary if using SSO
    pub role: String, // owner, staff, admin, editor, client, etc
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>, //e.g. active, pending, disabled, etc
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
}

impl DefaultKeys for User {
    fn set_default_keys(&mut self, from_map: &serde_json::Value) -> Result<()> {
        if let Some(uid) = from_map.get("uid") {
            let pks = uid.as_str().unwrap().to_string();
            self.pk = Some(format!("User#{}", pks));
        } else {
            self.pk = Some(gen_pk("User"));
            // use timexstamp_long() to generate a uid
        }
        self.sk = Some("#".to_string());
        Ok(())
    }
}

impl User {
    pub fn makepk(uid: Option<&str>) -> String {
        if uid.is_none() {
            "User".to_string()
        } else {
            format!("User#{}", uid.unwrap())
        }
    }

    pub fn makesk(uid: Option<&str>) -> String {
        if uid.is_none() {
            "#".to_string()
        } else {
            format!("#{}", uid.unwrap())
        }
    }

    pub fn uid(&self) -> String {
        let mut uid = self.pk.as_ref().map(|pk| pk.strip_prefix("User#"));
        if uid.is_none() {
            uid = self.sk.as_deref().map(|sk| sk.strip_prefix('#'))
        }
        uid.unwrap_or_default().to_owned()
    }

    pub fn iam(&self, role: &str) -> bool {
        self.role == role
    }

    pub async fn query_users(
        dynamo: &DynamoClient,
        last_key_str: Option<String>,
    ) -> Result<Vec<User>> {
        let pfx = "User#";
        let filter = "begins_with(#pk, :pkv)";
        let ean = HashMap::from([
            ("#pk".to_string(), "pk".to_string())
        ]);
        let eav = HashMap::from([
            (":pkv".to_string(), AttributeValue::S(pfx.to_string()))
        ]);
        let qo = dynamo.scan_with_filter2(filter, ean, eav, last_key_str, None).await?;
        let last = qo.last_evaluated_key.and_then(|v| last_evaluated_key_to_base64(v).ok());
        match qo.items {
            Some(items) => Ok((users_from_dynamodb(items)?, last)),
            None => Err(not_found_error("users").into())
        }
    }

    pub async fn get_user(dynamo: &DynamoClient, uid: &str) -> Result<Self> {
        let pk = Self::makepk(Some(uid));
        let sk = Self::makesk(None);
        let go = dynamo.get_by_pksk(&pk, &sk, None).await?;
        dbg!("get_user(uid={},pk={},sk={}) returned: {:?}", uid, &pk, &sk, &go);
        match go.item {
            Some(item) => item.try_into(),
            None => Err(not_found_error("user").into())
        }
    }

    pub async fn create_user(dynamo: &DynamoClient, data: serde_json::Value) -> Result<String> {
        let app = Self::try_from(data)?;
        let sk = String::from(app.sk.as_deref().unwrap_or_default());
        let item = app.try_into()?;
        let _po = dynamo.put_over(item, None).await?;
        tracing::info!("create_user returned: {:?}", _po);
        Ok(sk)
    }

    pub async fn delete_user(dynamo: &DynamoClient, uid: &str) -> Result<()> {
        let pk = Self::makepk(Some(uid));
        let sk = Self::makesk(None);
        let _ = dynamo.delete_by_pksk(&pk, &sk, None).await?;
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
        let body = data.as_object().unwrap();
        let mut changed = false;
        for key in fields {
            if body.contains_key(*key) {
                let vs = body.get(*key).and_then(|v| v.as_str()).unwrap();
                builder =
                    builder.add_update_with_value(key, &serde_json::Value::String(vs.to_string()));
                changed = true;
            }
        }
        if !changed {
            return Err(bad_request_error("No attributes to update").into());
        }
        let updates = builder.build().unwrap();
        let _uo = dynamo
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
