//! # Monitoring a Hyper Server with Prometheus
//!
use std::convert::Infallible;

use prometheus::*;
use lazy_static::lazy_static;
use prometheus_static_metric::auto_flush_from;
use prometheus_static_metric::make_auto_flush_static_metric;

use hyper::Body;
use hyper::Response;
use hyper::StatusCode;

make_auto_flush_static_metric! {

    pub label_enum HistogramLabelsAPI {
        user,
        auth,
        data,
        unknown,
        unsupported,
    }

    pub label_enum HistogramMethodsAPI {
        post,
        get,
        put,
        delete,
        search,
        login,
        create_verify,
        consume_verify,
        create_otp,
        consume_otp,
        upload,
        unknown,
        unsupported,
    }

    pub struct LhrsHistogram: LocalHistogram {
        "resource" => HistogramLabelsAPI,
        "method" => HistogramMethodsAPI,
    }
}

lazy_static! {
    pub static ref HTTP_HISTO_VEC: HistogramVec =
        register_histogram_vec ! (
            "http_request_duration_seconds",
            "HTTP request latencies in seconds",
            & [
                "resource",
                "method",
            ] // it doesn't matter for the label order
        ).unwrap();
}

lazy_static! {
    // You can also use default flush duration which is 60 seconds.
    // pub static ref TLS_HTTP_HISTOGRAM: Lhrs = auto_flush_from!(HTTP_HISTO_VEC, Lhrs);
    pub static ref TLS_HTTP_HISTOGRAM: LhrsHistogram =
        auto_flush_from!(
            HTTP_HISTO_VEC,
            LhrsHistogram,
            std::time::Duration::from_secs(60));
}

// Counter for Requests

make_auto_flush_static_metric! {

    pub label_enum CounterLabelsAPI {
        user,
        auth,
        data,
        unknown,
    }

    pub label_enum CounterMethodsAPI {
        post,
        get,
        put,
        delete,
        search,
        login,
        create_verify,
        consume_verify,
        create_otp,
        consume_otp,
        upload,
        unknown,
    }

    pub struct LhrsIntCounter: LocalIntCounter {
        "resource" => CounterLabelsAPI,
        "method" => CounterMethodsAPI,
    }
}

lazy_static! {
    pub static ref HTTP_COUNTER_VEC: IntCounterVec =
        register_int_counter_vec ! (
            "http_requests_total",
            "Number of HTTP requests.",
            & [
                "resource",
                "method",
            ] // it doesn't matter for the label order
        ).unwrap();
}

lazy_static! {
    // You can also use default flush duration which is 60 seconds.
    // pub static ref TLS_HTTP_COUNTER: Lhrs = auto_flush_from!(HTTP_COUNTER_VEC, Lhrs);
    pub static ref TLS_HTTP_COUNTER: LhrsIntCounter = auto_flush_from!(
        HTTP_COUNTER_VEC,
        LhrsIntCounter,
        std::time::Duration::from_secs(60));
}

// Counter for Requests by Success/Failure/StatusCode

make_auto_flush_static_metric! {

    pub label_enum CounterLabelsAPIStatusCode {
        user,
        auth,
        data,
        unknown,
        unsupported,
    }

    pub label_enum CounterMethodsAPIStatusCode {
        post,
        get,
        put,
        delete,
        search,
        login,
        create_verify,
        consume_verify,
        create_otp,
        consume_otp,
        upload,
        unknown,
        unsupported,
    }

    pub label_enum CounterResultAPIStatusCode {
        http_200,
        http_201,
        http_202,
        http_203,
        http_20x,
        http_400,
        http_401,
        http_402,
        http_403,
        http_404,
        http_405,
        http_40x,
        http_500,
        http_501,
        http_502,
        http_503,
        http_504,
        http_50x,
        unsupported,
    }

    pub struct LhrsIntCounterStatusCode: LocalIntCounter {
        "resource" => CounterLabelsAPIStatusCode,
        "method" => CounterMethodsAPIStatusCode,
        "status_code" => CounterResultAPIStatusCode,
    }
}

lazy_static! {
    pub static ref HTTP_COUNTER_VEC_STATUS_CODE: IntCounterVec =
        register_int_counter_vec ! (
            "http_requests_total_by_status_code",
            "Number of HTTP requests.",
            & [
                "resource",
                "method",
                "status_code"
            ] // it doesn't matter for the label order
        ).unwrap();
}

lazy_static! {
    // You can also use default flush duration which is 60 seconds.
    // pub static ref TLS_HTTP_COUNTER_STATUS_CODE: LhrsIntCounterStatusCode = auto_flush_from!(HTTP_COUNTER_VEC, Lhrs);
    pub static ref TLS_HTTP_COUNTER_STATUS_CODE: LhrsIntCounterStatusCode = auto_flush_from!(
        HTTP_COUNTER_VEC_STATUS_CODE,
        LhrsIntCounterStatusCode,
        std::time::Duration::from_secs(60));
}

/// handle_showing_metrics
///
/// Prometheus prefers to scrape metrics on a timed frequency. This function
/// hosts all the collected metrics under the uri=`/metrics` with
/// a `GET` method.
///
/// # Examples
///
/// ```rust
/// use crate::monitoring::metrics::handle_showing_metrics;
/// handle_showing_metrics();
/// ```
pub fn handle_showing_metrics()
-> std::result::Result<Response<Body>, Infallible>
{
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder
        .encode(&prometheus::gather(), &mut buffer)
        .expect("Failed to encode metrics");

    let response = String::from_utf8(buffer.clone()).expect("Failed to convert bytes to string");
    buffer.clear();

    let processed_result = Ok(
        Response::new(
            Body::from(response)));
    return processed_result;
}

/// record_monitoring_metrics_api_before
///
/// This method records tracked metrics using
/// [Prometheus](https://docs.rs/prometheus/latest/prometheus/)
/// before the internal service handlers start processing the
/// request.
///
/// # Arguments
///
/// * `uri` - `str&` - url sub path without the hosting fqdn address
/// * `resource` - `str&` - HTTP resource (`user`, `data`, `auth`, etc.)
/// * `method` - `str&` - HTTP method used (`get`, `post`, `put`, `delete`, etc.)
///
/// # Returns
///
/// `None` - this is a fire and forget method.
///
/// # Examples
///
/// ```rust
/// use crate::monitoring::metrics::record_monitoring_metrics_api_before;
/// record_monitoring_metrics_api_before(
///     "/user",
///     "user",
///     "post");
/// ```
pub fn record_monitoring_metrics_api_before(
    uri: &str,
    resource: &str,
    method: &str)
{
    debug!("\
        metrics - before - uri={uri} \
        resource={resource} \
        method={method}");

    match (
            resource,
            method) {
        ("auth", "login") => {
            TLS_HTTP_COUNTER.auth.login.inc();
            TLS_HTTP_HISTOGRAM.auth.login.observe(1.0);
        },
        ("user", "post") => {
            TLS_HTTP_COUNTER.user.post.inc();
            TLS_HTTP_HISTOGRAM.user.post.observe(1.0);
        },
        ("user", "delete") => {
            TLS_HTTP_COUNTER.user.delete.inc();
            TLS_HTTP_HISTOGRAM.user.delete.observe(1.0);
        },
        ("user", "put") => {
            TLS_HTTP_COUNTER.user.put.inc();
            TLS_HTTP_HISTOGRAM.user.put.observe(1.0);
        },
        ("user", "get") => {
            TLS_HTTP_COUNTER.user.get.inc();
            TLS_HTTP_HISTOGRAM.user.get.observe(1.0);
        },
        ("user", "search") => {
            TLS_HTTP_COUNTER.user.search.inc();
            TLS_HTTP_HISTOGRAM.user.search.observe(1.0);
        },
        ("user", "create_otp") => {
            TLS_HTTP_COUNTER.user.create_otp.inc();
            TLS_HTTP_HISTOGRAM.user.create_otp.observe(1.0);
        },
        ("user", "consume_otp") => {
            TLS_HTTP_COUNTER.user.consume_otp.inc();
            TLS_HTTP_HISTOGRAM.user.consume_otp.observe(1.0);
        },
        ("user", "consume_verify") => {
            TLS_HTTP_COUNTER.user.consume_verify.inc();
            TLS_HTTP_HISTOGRAM.user.consume_verify.observe(1.0);
        },
        // end of user
        ("data", "post") => {
            TLS_HTTP_COUNTER.data.post.inc();
            TLS_HTTP_HISTOGRAM.data.post.observe(1.0);
        },
        ("data", "delete") => {
            TLS_HTTP_COUNTER.data.delete.inc();
            TLS_HTTP_HISTOGRAM.data.delete.observe(1.0);
        },
        ("data", "put") => {
            TLS_HTTP_COUNTER.data.put.inc();
            TLS_HTTP_HISTOGRAM.data.put.observe(1.0);
        },
        ("data", "get") => {
            TLS_HTTP_COUNTER.data.get.inc();
            TLS_HTTP_HISTOGRAM.data.get.observe(1.0);
        },
        ("data", "search") => {
            TLS_HTTP_COUNTER.data.search.inc();
            TLS_HTTP_HISTOGRAM.data.search.observe(1.0);
        },
        ("data", "upload") => {
            TLS_HTTP_COUNTER.data.upload.inc();
            TLS_HTTP_HISTOGRAM.data.upload.observe(1.0);
        },
        // end of data
        ("unknown", "get") => {
            TLS_HTTP_COUNTER.unknown.get.inc();
            TLS_HTTP_HISTOGRAM.unknown.get.observe(1.0);
        },
        ("unknown", "post") => {
            TLS_HTTP_COUNTER.unknown.post.inc();
            TLS_HTTP_HISTOGRAM.unknown.post.observe(1.0);
        },
        // end of unknown
        (_, _) => {
            warn!("\
                metrics - before - unsupported - uri={uri} \
                resource={resource} \
                method={method}");
        }
    }
}

/// record_monitoring_metrics_api_after
///
/// This method records tracked metrics using
/// [Prometheus](https://docs.rs/prometheus/latest/prometheus/)
/// after the internal service handlers processed the
/// request. This allows for tracking latency and status codes
/// for each resource and each method.
///
/// # Arguments
///
/// * `uri` - `str&` - url sub path without the hosting fqdn address
/// * `resource` - `str&` - HTTP resource (`user`, `data`, `auth`, etc.)
/// * `method` - `str&` - HTTP method used (`get`, `post`, `put`, `delete`, etc.)
/// * `processed_response` - `str&` - existing [`Response`](hyper::Response)
///   from the internal service handler
///
/// # Returns
///
/// Hyper [`Response`](hyper::Response) nested in a `Result`
///
/// `std::result::Result<Response<Body>, Infallible>`
///
/// # Examples
///
/// ```rust
/// use hyper::Body;
/// use hyper::Response;
/// use crate::monitoring::metrics::record_monitoring_metrics_api_after;
/// let mut processed_result: std::result::Result<Response<Body>, Infallible> = Ok(
///     Response::new(
///     Body::from(format!("test body message"))));
/// record_monitoring_metrics_api_after(
///     "/user",
///     "user",
///     "post",
///     processed_result);
/// ```
pub fn record_monitoring_metrics_api_after(
    uri: &str,
    resource: &str,
    method: &str,
    processed_response: std::result::Result<Response<Body>, Infallible>)
-> std::result::Result<Response<Body>, Infallible>
{
    if processed_response.is_ok() {
        let cloned_result = processed_response.unwrap();
        match (
                resource,
                method) {
            ("auth", "login") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.auth.login.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.auth.login.observe(1.0);
            },
            ("user", "post") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.post.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.post.observe(1.0);
            },
            ("user", "delete") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.delete.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.delete.observe(1.0);
            },
            ("user", "put") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.put.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.put.observe(1.0);
            },
            ("user", "get") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.get.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.get.observe(1.0);
            },
            ("user", "search") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.search.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.search.observe(1.0);
            },
            ("user", "create_otp") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.create_otp.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.create_otp.observe(1.0);
            },
            ("user", "consume_otp") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_otp.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.consume_otp.observe(1.0);
            },
            ("user", "consume_verify") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.user.consume_verify.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.user.consume_verify.observe(1.0);
            },
            // end of user
            ("data", "post") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.data.post.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.data.post.observe(1.0);
            },
            ("data", "delete") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.data.delete.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.data.delete.observe(1.0);
            },
            ("data", "put") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.data.put.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.data.put.observe(1.0);
            },
            ("data", "get") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.data.get.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.data.get.observe(1.0);
            },
            ("data", "search") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.data.search.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.data.search.observe(1.0);
            },
            ("data", "upload") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.data.upload.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.data.upload.observe(1.0);
            },
            // end of data
            ("unknown", "get") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.get.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.unknown.get.observe(1.0);
            },
            ("unknown", "post") => {
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.post.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.unknown.post.observe(1.0);
            },
            // end of unknown
            (_, _) => {
                warn!("\
                    metrics - after - unsupported - uri={uri} \
                    resource={resource} \
                    method={method}");
                match cloned_result.status() {
                    StatusCode::OK => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_200.inc();
                    },
                    StatusCode::CREATED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_201.inc();
                    },
                    StatusCode::BAD_REQUEST => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_400.inc();
                    },
                    StatusCode::UNAUTHORIZED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_401.inc();
                    },
                    StatusCode::FORBIDDEN => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_403.inc();
                    },
                    StatusCode::NOT_FOUND => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_404.inc();
                    },
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_500.inc();
                    },
                    StatusCode::NOT_IMPLEMENTED => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_501.inc();
                    },
                    StatusCode::BAD_GATEWAY => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_502.inc();
                    },
                    StatusCode::SERVICE_UNAVAILABLE => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_503.inc();
                    },
                    StatusCode::GATEWAY_TIMEOUT => {
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.http_504.inc();
                    },
                    _ => {
                        error!("\
                            unsupported metric \
                            resource={resource} \
                            method={method} \
                            result={:?} \
                            status_code={:?}",
                            cloned_result,
                            cloned_result.status());
                        TLS_HTTP_COUNTER_STATUS_CODE.unknown.unsupported.unsupported.inc();
                    },
                }
                TLS_HTTP_HISTOGRAM.unknown.unsupported.observe(1.0);
            }
        }
        return Ok(cloned_result);
    }

    return processed_response;
}
