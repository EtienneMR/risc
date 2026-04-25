//! @core/http — synchronous HTTP client built on ureq.
//! All functions return {status, body, headers}; non-2xx responses are not raised as errors.
//! http.get(url, headers?) and http.delete(url, headers?) send bodyless requests.
//! http.post(url, body, headers?) and http.put(url, body, headers?) send string bodies.
//! http.request(method, url, body, headers?) is the low-level escape hatch for other verbs.

use std::rc::Rc;

use crate::{
    error::NativeError,
    value::{CallContext, Signal, Table, TableKey, Value},
};

use super::helpers::{define_in, get_string};

pub fn create() -> Value {
    let t = Table::new();
    define_in(&t, "http.get", http_get);
    define_in(&t, "http.post", http_post);
    define_in(&t, "http.put", http_put);
    define_in(&t, "http.delete", http_delete);
    define_in(&t, "http.request", http_request);
    Value::Table(t)
}

fn http_err(ctx: &CallContext, e: impl std::fmt::Display) -> Signal {
    ctx.error(NativeError::new("http error", e.to_string()))
}

fn apply_headers(mut req: ureq::Request, ctx: &CallContext, index: usize) -> ureq::Request {
    if let Value::Table(t) = ctx.get(index, "headers") {
        for (k, v) in t.entries() {
            if let TableKey::String(name) = k {
                req = req.set(&name, &v.to_string_ref());
            }
        }
    }
    req
}

fn finish(ctx: &CallContext, resp: ureq::Response) -> Result<Value, Signal> {
    let status = resp.status() as f64;

    // Collect header name→value pairs before consuming resp.
    let header_pairs: Vec<(String, String)> = resp
        .headers_names()
        .into_iter()
        .filter_map(|name| resp.header(&name).map(|v| (name, v.to_owned())))
        .collect();

    let body = resp
        .into_string()
        .map_err(|e| http_err(ctx, format!("reading body: {e}")))?;

    let mut headers = Table::new();
    for (name, val) in header_pairs {
        headers.set(name.as_str(), Value::String(Rc::from(val.as_str())));
    }

    let mut t = Table::new();
    t.set("status", Value::Number(status));
    t.set("body", Value::String(Rc::from(body.as_str())));
    t.set("headers", Value::Table(headers));
    Ok(Value::Table(t))
}

fn call_no_body(ctx: &CallContext, req: ureq::Request) -> Result<Value, Signal> {
    let resp = match req.call() {
        Ok(r) | Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(http_err(ctx, e)),
    };
    finish(ctx, resp)
}

fn call_with_body(ctx: &CallContext, req: ureq::Request, body: &str) -> Result<Value, Signal> {
    let resp = match req.send_string(body) {
        Ok(r) | Err(ureq::Error::Status(_, r)) => r,
        Err(e) => return Err(http_err(ctx, e)),
    };
    finish(ctx, resp)
}

fn http_get(ctx: CallContext) -> Result<Value, Signal> {
    let url = get_string(&ctx, 0, "url", "http.get")?;
    let req = apply_headers(ureq::get(url.as_ref()), &ctx, 1);
    call_no_body(&ctx, req)
}

fn http_delete(ctx: CallContext) -> Result<Value, Signal> {
    let url = get_string(&ctx, 0, "url", "http.delete")?;
    let req = apply_headers(ureq::delete(url.as_ref()), &ctx, 1);
    call_no_body(&ctx, req)
}

fn http_post(ctx: CallContext) -> Result<Value, Signal> {
    let url = get_string(&ctx, 0, "url", "http.post")?;
    let body = ctx.get(1, "body").to_string_ref();
    let req = apply_headers(ureq::post(url.as_ref()), &ctx, 2);
    call_with_body(&ctx, req, &body)
}

fn http_put(ctx: CallContext) -> Result<Value, Signal> {
    let url = get_string(&ctx, 0, "url", "http.put")?;
    let body = ctx.get(1, "body").to_string_ref();
    let req = apply_headers(ureq::put(url.as_ref()), &ctx, 2);
    call_with_body(&ctx, req, &body)
}

fn http_request(ctx: CallContext) -> Result<Value, Signal> {
    let method = get_string(&ctx, 0, "method", "http.request")?;
    let url = get_string(&ctx, 1, "url", "http.request")?;
    let body = ctx.get(2, "body").to_string_ref();
    let req = apply_headers(ureq::request(method.as_ref(), url.as_ref()), &ctx, 3);
    call_with_body(&ctx, req, &body)
}
