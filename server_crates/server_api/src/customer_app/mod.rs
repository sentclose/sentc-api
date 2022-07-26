pub(crate) mod app_entities;

use rustgram::Request;

use crate::core::api_res::{echo, JRes};
use crate::core::input_helper::get_raw_body;
use crate::customer_app::app_entities::{AppJwtRegisterOutput, AppRegisterOutput};
use crate::user::jwt::create_jwt_keys;

pub(crate) async fn create_app(mut req: Request) -> JRes<AppRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	//1. create the first jwt keys
	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	//2. create an new app (with new secret_token and public_token)
	//3. save both tokens hashed in the db
	let customer_id = "abc".to_string();
	let app_id = "dfg".to_string();

	let customer_app_data = AppRegisterOutput {
		customer_id: customer_id.to_string(),
		app_id: app_id.to_string(),
		secret_token: "".to_string(),
		public_token: "".to_string(),
		jwt_data: AppJwtRegisterOutput {
			customer_id,
			app_id,
			jwt_verify_key,
			jwt_sign_key,
			jwt_alg: alg.to_string(),
		},
	};

	echo(customer_app_data)
}
