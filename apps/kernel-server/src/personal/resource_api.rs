//! Private, versioned resource projection for Personal daemon clients.
//!
//! This is deliberately not a public contract or a durable generic Resource
//! aggregate. It exposes the six fixed product families as daemon observations
//! and makes missing authority backends explicit rather than fabricating rows.

use std::collections::VecDeque;

use serde_json::{Value, json};

const PROJECTION_VERSION: &str = "personal-resource-projection/1";
const MAX_WATCH_EVENTS: usize = 128;
const RESOURCE_FAMILIES: [&str; 6] = ["memory", "skill", "tool", "context", "task", "runtime"];

/// A response ready for the loopback HTTP transport.
pub(crate) struct ResourceApiResponse {
    pub status: u16,
    pub body: String,
    pub content_type: &'static str,
}

/// Process-lifetime delivery state for private resource observations.
pub(crate) struct ResourceApi {
    next_watch_sequence: u64,
    watch_events: VecDeque<(u64, String, Value)>,
}

impl ResourceApi {
    pub(crate) fn new() -> Self {
        let mut api = Self {
            next_watch_sequence: 1,
            watch_events: VecDeque::new(),
        };
        for family in RESOURCE_FAMILIES {
            api.publish(family, "projection.initialized", family_projection(family));
        }
        api
    }

    pub(crate) fn handle(&self, method_path: &str) -> ResourceApiResponse {
        let (path, query) = method_path
            .split_once('?')
            .map_or((method_path, ""), |(path, query)| (path, query));
        let family = match query_parameter(query, "family") {
            Some(family) if RESOURCE_FAMILIES.contains(&family) => family,
            Some(_) => {
                return error(
                    400,
                    "RESOURCE_PROJECTION_FAMILY_INVALID",
                    "resource family must be one of memory, skill, tool, context, task, or runtime",
                );
            }
            None => "task",
        };
        if let Some(version) = query_parameter(query, "version")
            && version != "1"
        {
            return error(
                400,
                "RESOURCE_PROJECTION_VERSION_UNSUPPORTED",
                "private resource projection version 1 is required",
            );
        }
        if path == "GET /resource/v1/projection" {
            return json_response(
                200,
                snapshot(family, self.next_watch_sequence.saturating_sub(1)),
            );
        }
        if path == "GET /resource/v1/watch" {
            return self.watch(family, query);
        }
        error(
            404,
            "RESOURCE_PROJECTION_ROUTE_NOT_FOUND",
            "no private resource projection route matched",
        )
    }

    fn watch(&self, family: &str, query: &str) -> ResourceApiResponse {
        let requested_sequence = query_parameter(query, "resume_from")
            .map(str::parse::<u64>)
            .transpose()
            .ok()
            .flatten();
        let oldest_sequence = self
            .watch_events
            .iter()
            .find(|(_, event_family, _)| event_family == family)
            .map(|(sequence, _, _)| *sequence)
            .unwrap_or(self.next_watch_sequence);
        if requested_sequence.is_some_and(|sequence| sequence.saturating_add(1) < oldest_sequence) {
            return error(
                409,
                "RESOURCE_WATCH_RESUME_STALE",
                "requested resource projection cursor is no longer retained for this family",
            );
        }
        let mut frames = vec![format!(
            "event: snapshot\ndata: {}\n\n",
            snapshot(family, self.next_watch_sequence.saturating_sub(1))
        )];
        for (sequence, event_family, event) in
            self.watch_events
                .iter()
                .filter(|(sequence, event_family, _)| {
                    event_family == family
                        && requested_sequence.is_none_or(|resume| *sequence > resume)
                })
        {
            frames.push(format!(
                "id: {sequence}\nevent: delta\ndata: {}\n\n",
                json!({"kind":"delta", "family": event_family, "sequence": sequence, "event": event})
            ));
        }
        ResourceApiResponse {
            status: 200,
            body: frames.concat(),
            content_type: "text/event-stream",
        }
    }

    fn publish(&mut self, family: &str, kind: &str, body: Value) {
        self.watch_events.push_back((
            self.next_watch_sequence,
            family.to_owned(),
            json!({"kind": kind, "body": body}),
        ));
        self.next_watch_sequence = self.next_watch_sequence.saturating_add(1);
        if self.watch_events.len() > MAX_WATCH_EVENTS {
            self.watch_events.pop_front();
        }
    }
}

fn snapshot(family: &str, latest_sequence: u64) -> Value {
    json!({
        "kind": "snapshot",
        "projection_version": PROJECTION_VERSION,
        "family": family,
        "latest_sequence": latest_sequence,
        "projection": family_projection(family),
    })
}

fn family_projection(family: &str) -> Value {
    let (availability, authority_source) = match family {
        "task" => ("available", "daemon-task-application-service"),
        "runtime" => ("available", "personal-daemon-runtime"),
        _ => ("not-backed", "authority-service-not-yet-implemented"),
    };
    json!({
        "family": family,
        "availability": availability,
        "authority_source": authority_source,
        "resources": [],
        "authority_side_effects": false,
    })
}

fn query_parameter<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix(&format!("{name}=")))
        .and_then(|value| value.split_whitespace().next())
}

fn json_response(status: u16, body: Value) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: body.to_string(),
        content_type: "application/json",
    }
}

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    json_response(
        status,
        json!({"status":"error", "code": code, "message": message}),
    )
}
