use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use bsnext_guards::route_guard::RouteGuard;
use bsnext_input::route::{ListOrSingle, Route};
use bsnext_input::when_guard::{JsonGuard, JsonPropGuard, WhenBodyGuard};
use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{Method, Uri};
use serde_json::Value;
use std::ops::ControlFlow;
use tracing::trace;

pub struct NeedsJsonGuard<'a>(pub &'a ListOrSingle<WhenBodyGuard>);
impl RouteGuard for NeedsJsonGuard<'_> {
    #[tracing::instrument(skip_all, name = "NeedsJsonGuard.accept_req")]
    fn accept_req(&self, req: &Request, _outer_uri: &Uri) -> bool {
        let exec = match self.0 {
            ListOrSingle::WhenOne(WhenBodyGuard::Json { .. }) => true,
            ListOrSingle::WhenMany(items) => items
                .iter()
                .any(|item| matches!(item, WhenBodyGuard::Json { .. })),
            _ => false,
        };
        trace!(?exec);
        if !exec {
            return false;
        }
        let headers = req.headers();
        let method = req.method();
        let json = headers.get(CONTENT_TYPE).is_some_and(|header| {
            header
                .to_str()
                .ok()
                .map(|h| h.contains("application/json"))
                .unwrap_or(false)
        });
        trace!(?json, ?method, ?headers);
        json && method == Method::POST
    }

    fn accept_res<T>(&self, _res: &Response<T>, _outer_uri: &Uri) -> bool {
        true
    }
}

impl NeedsJsonGuard<'_> {
    pub fn match_body(&self, value: &Value) -> bool {
        let matches: Vec<(&'_ WhenBodyGuard, bool)> = match self.0 {
            ListOrSingle::WhenOne(one) => vec![(one, match_one_json(value, one))],
            ListOrSingle::WhenMany(many) => many
                .iter()
                .map(|guard| (guard, match_one_json(value, guard)))
                .collect(),
        };
        matches.iter().all(|(_item, result)| *result)
    }
}

fn match_one_json(value: &Value, when_body_guard: &WhenBodyGuard) -> bool {
    match when_body_guard {
        WhenBodyGuard::Json { json } => match json {
            JsonGuard::ArrayLast { items, last } => match value.pointer(items) {
                Some(Value::Array(arr)) => match arr.last() {
                    None => false,
                    Some(last_item) => last.iter().all(|prop_guard| match prop_guard {
                        JsonPropGuard::PathIs { path, is } => match last_item.pointer(path) {
                            Some(Value::String(val_string)) => val_string == is,
                            _ => false,
                        },
                        JsonPropGuard::PathHas { path, has } => match last_item.pointer(path) {
                            Some(Value::String(val_string)) => val_string.contains(has),
                            _ => false,
                        },
                    }),
                },
                _ => false,
            },
            JsonGuard::ArrayAny { items, any } => match value.pointer(items) {
                Some(Value::Array(arr)) if arr.is_empty() => false,
                Some(Value::Array(val)) => val
                    .iter()
                    .any(|val| any.iter().any(|prop_guard| match_prop(val, prop_guard))),
                _ => false,
            },
            JsonGuard::ArrayAll { items, all } => match value.pointer(items) {
                Some(Value::Array(arr)) if arr.is_empty() => false,
                Some(Value::Array(arr)) => arr
                    .iter()
                    .any(|one_val| all.iter().all(|guard| match_prop(one_val, guard))),
                _ => false,
            },
            JsonGuard::Path(pg) => match_prop(value, pg),
        },
        WhenBodyGuard::Never => false,
    }
}

pub async fn match_json_body(
    body: &mut Option<Body>,
    route: &Route,
) -> (Option<Body>, ControlFlow<()>) {
    use http_body_util::BodyExt;
    if let Some(inner_body) = body.take() {
        let collected = inner_body.collect();
        let bytes = match collected.await {
            Ok(collected) => collected.to_bytes(),
            Err(err) => {
                tracing::error!(?err, "could not collect bytes...");
                Bytes::new()
            }
        };

        trace!("did collect {} bytes", bytes.len());

        match serde_json::from_slice(bytes.iter().as_slice()) {
            Ok(value) => {
                let result = route
                    .when_body
                    .as_ref()
                    .map(|when_body| NeedsJsonGuard(when_body).match_body(&value));
                if result.is_some_and(|res| !res) {
                    trace!("ignoring, `when_body` was present, but didn't match the guards");
                    trace!("restoring body from clone");
                    (Some(Body::from(bytes)), ControlFlow::Break(()))
                } else {
                    if result.is_some() {
                        trace!("✅ when_body produced a valid match");
                    } else {
                        trace!("when_body didn't produce a value");
                    }
                    (Some(Body::from(bytes)), ControlFlow::Continue(()))
                }
            }
            Err(err) => {
                tracing::error!(?err, "could not deserialize into Value");
                (Some(Body::from(bytes)), ControlFlow::Continue(()))
            }
        }
    } else {
        trace!("could not .take() body");
        (None, ControlFlow::Continue(()))
    }
}

pub fn match_prop(value: &Value, prop_guard: &JsonPropGuard) -> bool {
    match prop_guard {
        JsonPropGuard::PathIs { path, is } => match value.pointer(path) {
            Some(Value::String(val_string)) => val_string == is,
            _ => false,
        },
        JsonPropGuard::PathHas { path, has } => match value.pointer(path) {
            Some(Value::String(val_string)) => val_string.contains(has),
            _ => false,
        },
    }
}
