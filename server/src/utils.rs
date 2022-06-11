pub mod http_utils {
    use hyper::body::Buf;
    use hyper::{body, Body, Request};
    use serde::de::DeserializeOwned;
    use serde::Deserialize;
    use std::borrow::BorrowMut;
    use std::collections::HashMap;
    use std::io::Read;

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
        let mut body = req.body_mut();
        let bytes = body::to_bytes(body).await.unwrap();
        let body_str = std::str::from_utf8(&*bytes).unwrap();
        serde_json::from_str::<T>(body_str).unwrap()
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
