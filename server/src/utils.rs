pub mod http_utils {
    use hyper::{body, Body, Request, Response, StatusCode};
    use serde::de::DeserializeOwned;
    use std::collections::HashMap;
    use std::convert::Infallible;

    pub fn get_query_params(req: &Request<Body>) -> HashMap<&str, &str> {
        let uri = req.uri();
        let query = uri.query();
        println!("uri={:?}, query={:?}", uri, query);
        match query {
            None => HashMap::new(),
            Some(query) => query
                .split("&")
                .map(|s| s.split_at(s.find("=").unwrap()))
                .map(|(key, value)| (key, &value[1..]))
                .collect(),
        }
    }

    pub async fn read_json_body<T>(req: &mut Request<Body>) -> T
    where
        T: DeserializeOwned,
    {
        let body_str = read_json_body_raw(req).await;
        serde_json::from_str::<T>(&body_str).unwrap()
    }

    pub async fn read_json_body_raw(req: &mut Request<Body>) -> String
    {
        let body = req.body_mut();
        let bytes = body::to_bytes(body).await.unwrap();
        std::str::from_utf8(&*bytes).unwrap().to_owned()
    }

    pub fn build_success_res(value: &str) -> Result<Response<Body>, Infallible> {
        let json = format!("{{\"data\": {}}}", value);
        let mut res = Response::new(Body::from(json));
        let headers = res.headers_mut();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        Ok(res)
    }

    pub fn build_error_res(error: &str, status: StatusCode) -> Result<Response<Body>, Infallible> {
        let json = format!("{{\"error\": \"{}\"}}", error);
        let mut res = Response::new(Body::from(json));
        *res.status_mut() = status;
        return Ok(res);
    }
}

#[cfg(test)]
pub mod http_utils_tests {
    use super::*;
    use crate::utils::http_utils::get_query_params;
    use hyper::http::uri::{Builder, Parts};
    use hyper::{Body, Request, Uri};
    use rstest::rstest;
    use std::collections::HashMap;

    #[rstest]
    #[case(
        "?test=abc",
        HashMap::from([("test", "abc")])
    )]
    #[case(
        "?test=abc&help=123",
        HashMap::from([("test", "abc"), ("help", "123")])
    )]
    #[case(
        "show?topic=status&key=abc",
        HashMap::from([("topic", "status"), ("key", "abc")])
    )]
    fn get_query_params_tests(#[case] query_str: &str, #[case] expected: HashMap<&str, &str>) {
        let uri = Builder::new()
            .scheme("https")
            .authority("behnke.rs")
            .path_and_query(query_str)
            .build()
            .unwrap();
        let req = Request::get(uri).body(Body::empty()).unwrap();
        let res = get_query_params(&req);
        assert_eq!(res, expected)
    }
}

pub mod time_utils {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        return since_the_epoch.as_millis();
    }
}

pub mod json_utils {
    pub fn unescape(json: &str) -> String {
        return json.replace("\\\"", "\"")
    }
}
