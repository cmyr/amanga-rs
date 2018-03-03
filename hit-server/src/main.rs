extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate anagram_hit_manager as hit_manager;

use hyper::{Response, StatusCode};

use gotham::http::response::create_response;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};

use hit_manager::{DbAdapter, JoinedHit, HitStatus};

// heavily adapted from the gotham examples

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct HitQueryParams {
    #[serde(default)]
    newer_than: i32,
    status: Option<HitStatus>,
    #[serde(default)]
    pretty: bool,
}

const MAX_RESULTS: usize = 50;

fn get_hits_handler(state: State) -> (State, Response) {
    let res = {
        let params = HitQueryParams::borrow_from(&state);
        let db = DbAdapter::new();
        let result = db.get_hits(params.status, MAX_RESULTS, params.newer_than);
        let result = match params.pretty {
            true => serde_json::to_string_pretty(&result).unwrap(),
            false => serde_json::to_string(&result).unwrap(),
        };
        create_response(
            &state,
            StatusCode::Ok,
            Some((
                result.into_bytes(),
                mime::APPLICATION_JSON,
            )),
        )
    };

    (state, res)
}

fn router() -> Router {
    build_simple_router(|route| {
        route
            .get("/hits")
            .with_query_string_extractor::<HitQueryParams>()
            .to(get_hits_handler);
    })
}

/// Start a server and use a `Router` to dispatch requests
pub fn main() {
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn hit_query() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/hits")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

		let response: Vec<JoinedHit> = serde_json::from_reader(response.read_body().unwrap().as_slice()).unwrap();
        assert_eq!(response.len(), 50);

        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/hits?status=new&newer_than=50")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

		let response: Vec<JoinedHit> = serde_json::from_reader(response.read_body().unwrap().as_slice()).unwrap();
        assert_eq!(response.len(), 6);
    }
}
