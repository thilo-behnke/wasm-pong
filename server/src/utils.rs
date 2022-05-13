pub mod http_utils {
    use std::collections::HashMap;
    use hyper::{Body, Request};

    pub fn get_query_params(req: &Request<Body>) -> HashMap<&str, &str> {
        let uri = req.uri();
        let query = uri.query();
        match query {
            None => HashMap::new(),
            Some(query) => {
                query.split("&").map(|s| s.split_at(query.find("=").unwrap())).collect()
            }
        }
    }
}
