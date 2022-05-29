pub mod http_utils {
    use std::borrow::BorrowMut;
    use std::collections::HashMap;
    use std::io::Read;
    use hyper::{Body, body, Request};
    use hyper::body::Buf;
    use serde::de::DeserializeOwned;
    use serde::Deserialize;

    pub fn get_query_params(req: &Request<Body>) -> HashMap<&str, &str> {
        let uri = req.uri();
        let query = uri.query();
        println!("uri={:?}, query={:?}", uri, query);
        match query {
            None => HashMap::new(),
            Some(query) => {
                query.split("&").map(|s| s.split_at(s.find("=").unwrap())).map(|(key, value)| (key, &value[1..])).collect()
            }
        }
    }

    pub async fn read_json_body<T>(req: &mut Request<Body>) -> T where T : DeserializeOwned {
        let mut body = req.body_mut();
        let bytes = body::to_bytes(body).await.unwrap();
        let body_str = std::str::from_utf8(&*bytes).unwrap();
        serde_json::from_str::<T>(body_str).unwrap()
    }
}

#[cfg(test)]
pub mod http_utils_tests {
    use rstest::rstest;
    use std::collections::HashMap;
    use hyper::{Body, Request, Uri};
    use hyper::http::uri::{Builder, Parts};
    use crate::utils::http_utils::get_query_params;
    use super::*;

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
        let uri = Builder::new().scheme("https").authority("behnke.rs").path_and_query(query_str).build().unwrap();
        let req = Request::get(uri).body(Body::empty()).unwrap();
        let res = get_query_params(&req);
        assert_eq!(res, expected)
    }
}
