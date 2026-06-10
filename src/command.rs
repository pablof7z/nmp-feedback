use std::ffi::CString;

use nmp_ffi::NmpApp;
use serde::{Deserialize, Serialize};

use crate::config::{text_note_kind, FeedbackConfig};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FeedbackCommandOutcome {
    pub ok: bool,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl FeedbackCommandOutcome {
    #[must_use]
    pub fn accepted(status: impl Into<String>) -> Self {
        Self {
            ok: true,
            status: status.into(),
            error: None,
        }
    }

    #[must_use]
    pub fn rejected(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            status: "rejected".to_string(),
            error: Some(error.into()),
        }
    }

    #[must_use]
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            serde_json::json!({
                "ok": false,
                "status": "rejected",
                "error": "feedback outcome encoding failed"
            })
        })
    }
}

#[must_use]
pub fn fetch_feedback(app: *mut NmpApp, config: &FeedbackConfig) -> FeedbackCommandOutcome {
    if app.is_null() {
        return FeedbackCommandOutcome::rejected("feedback runtime unavailable");
    }
    // SAFETY: app is non-null and owned by the host NMP runtime.
    unsafe { &*app }.push_interest(config.interest());
    FeedbackCommandOutcome::accepted("subscribed")
}

#[must_use]
pub fn publish_feedback(
    app: *mut NmpApp,
    config: &FeedbackConfig,
    category: &str,
    content: &str,
    parent_event_id: Option<&str>,
    reply_to_pubkey: Option<&str>,
) -> FeedbackCommandOutcome {
    let content = content.trim();
    if content.is_empty() {
        return FeedbackCommandOutcome::rejected("empty feedback");
    }
    if app.is_null() {
        return FeedbackCommandOutcome::rejected("feedback runtime unavailable");
    }
    let tags = config.tags(category, parent_event_id, reply_to_pubkey);
    let body = serde_json::json!({
        "PublishRaw": {
            "kind": text_note_kind(),
            "tags": tags,
            "content": content,
            "target": { "Explicit": { "relays": [&config.relay_url] } },
        }
    });
    dispatch_nmp_publish(app, body)
}

fn dispatch_nmp_publish(app: *mut NmpApp, body: serde_json::Value) -> FeedbackCommandOutcome {
    let Ok(namespace) = CString::new("nmp.publish") else {
        return FeedbackCommandOutcome::rejected("invalid publish namespace");
    };
    let Ok(body) = CString::new(body.to_string()) else {
        return FeedbackCommandOutcome::rejected("invalid publish body");
    };
    let raw = nmp_ffi::nmp_app_dispatch_action(app, namespace.as_ptr(), body.as_ptr());
    if raw.is_null() {
        return FeedbackCommandOutcome::rejected("publish dispatch failed");
    }
    nmp_ffi::nmp_app_free_string(raw);
    FeedbackCommandOutcome::accepted("queued")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_publish_is_rejected_before_dispatch() {
        let config = FeedbackConfig::new("31933:abc:app");
        let out = publish_feedback(std::ptr::null_mut(), &config, "bug", "  ", None, None);
        assert_eq!(out.ok, false);
        assert_eq!(out.error.as_deref(), Some("empty feedback"));
    }

    #[test]
    fn null_runtime_is_rejected_not_faked() {
        let config = FeedbackConfig::new("31933:abc:app");
        let fetch = fetch_feedback(std::ptr::null_mut(), &config);
        let publish = publish_feedback(
            std::ptr::null_mut(),
            &config,
            "bug",
            "real text",
            None,
            None,
        );
        assert_eq!(fetch.error.as_deref(), Some("feedback runtime unavailable"));
        assert_eq!(
            publish.error.as_deref(),
            Some("feedback runtime unavailable")
        );
    }
}
