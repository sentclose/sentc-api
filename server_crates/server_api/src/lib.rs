#![allow(clippy::too_many_arguments, clippy::manual_map, clippy::tabs_in_doc_comments, clippy::from_over_into)]

use rustgram::Router;

use crate::routes::routes;

mod group;
mod key_management;
mod routes;
mod user;
pub mod util;

pub use group::{
	group_controller as sentc_group_controller,
	group_entities as sentc_group_entities,
	group_key_rotation_controller as sentc_group_key_rotation_controller,
	group_light_controller as sentc_group_light_controller,
	group_service as sentc_group_service,
	group_user_controller as sentc_group_user_controller,
	group_user_service as sentc_group_user_service,
};
pub use key_management::{key_controller as sentc_key_controller, key_entity as sentc_key_entities};
pub use user::auth::auth_service as sentc_auth_service;
pub use user::light::{user_light_controller as sentc_user_light_controller, user_light_service as sentc_user_light_service};
pub use user::{user_controller as sentc_user_controller, user_entities as sentc_user_entities, user_service as sentc_user_service};

pub fn rest_routes(router: &mut Router)
{
	routes(router);
}
