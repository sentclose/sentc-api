use rustgram::Request;

use crate::core::api_err::HttpErr;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::customer::customer_entities::{CustomerAppJwtRegisterOutput, CustomerAppRegisterOutput, CustomerRegisterData};
use crate::user::jwt::create_jwt_keys;

//TODO create new customer, delete customer, see how many active user per customer, valid customer data and valid customer token
pub(crate) mod customer_entities;

pub(crate) async fn create(req: Request) -> Result<String, HttpErr>
{
	let body = get_raw_body(req).await?;

	let _register_data: CustomerRegisterData = bytes_to_json(&body)?;

	//check the email if it is a real email. send an email to complete the registration

	Ok(format!("done"))
}

pub(crate) async fn done_create(_req: Request) -> Result<String, HttpErr>
{
	//create the jwt keys when email was ok

	Ok(format!("done"))
}

pub(crate) async fn create_app(_req: Request) -> Result<String, HttpErr>
{
	//1. create the first jwt keys
	let (jwt_sign_key, jwt_verify_key, alg) = create_jwt_keys()?;

	//2. create an new app (with new secret_token and public_token)
	//3. save both tokens hashed in the db
	let customer_id = "abc".to_string();
	let app_id = "dfg".to_string();

	let _customer_app_data = CustomerAppRegisterOutput {
		customer_id: customer_id.to_string(),
		app_id: app_id.to_string(),
		secret_token: "".to_string(),
		public_token: "".to_string(),
		jwt_data: CustomerAppJwtRegisterOutput {
			customer_id,
			app_id,
			jwt_verify_key,
			jwt_sign_key,
			jwt_alg: alg.to_string(),
		},
	};

	Ok(format!("done"))
}
