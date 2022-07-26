use rustgram::Request;

use crate::core::api_res::HttpErr;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
use crate::customer::customer_entities::CustomerRegisterData;

//TODO create new customer, delete customer, see how many active user per customer, valid customer data and valid customer token
pub(crate) mod customer_entities;

pub(crate) async fn register(mut req: Request) -> Result<String, HttpErr>
{
	let body = get_raw_body(&mut req).await?;

	let _register_data: CustomerRegisterData = bytes_to_json(&body)?;

	//check the email if it is a real email. send an email to complete the registration

	Ok(format!("done"))
}

pub(crate) async fn done_register(_req: Request) -> Result<String, HttpErr>
{
	//create the jwt keys when email was ok

	Ok(format!("done"))
}
