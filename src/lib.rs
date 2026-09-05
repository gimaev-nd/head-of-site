pub mod fetch;
pub mod parse;

mod components;
mod routes;

use topcoat::router::{Router, RouterBuilderDiscoverExt};

pub fn router() -> Router {
    Router::builder().discover().build()
}
