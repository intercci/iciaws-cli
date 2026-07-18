use iciaws_router::router::Router;

use crate::handlers::{versions::GetVersionHandler, users::GetMyselfHandler,};

pub fn add_routes(router: &mut Router) {
    router.add_route(GetVersionHandler::get_key(), Box::new(GetVersionHandler));
    router.add_route(GetMyselfHandler::get_key(), Box::new(GetMyselfHandler));
}
