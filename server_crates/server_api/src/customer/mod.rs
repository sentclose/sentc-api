use rand::RngCore;
use rustgram::Request;
use sentc_crypto_common::server_default::ServerSuccessOutput;
use sentc_crypto_common::user::{DoneLoginServerInput, PrepareLoginSaltServerOutput, PrepareLoginServerInput};
use sentc_crypto_common::CustomerId;
use server_api_common::customer::{
	CustomerDoneLoginOutput,
	CustomerDoneRegistrationInput,
	CustomerEmailData,
	CustomerRegisterData,
	CustomerRegisterOutput,
};

use crate::core::api_res::{echo, echo_success, ApiErrorCodes, AppRes, HttpErr, JRes};
use crate::core::email;
#[cfg(feature = "send_mail")]
use crate::core::email::send_mail::send_mail_registration;
use crate::core::input_helper::{bytes_to_json, get_raw_body};
#[cfg(feature = "send_mail")]
use crate::customer::customer_entities::RegisterEmailStatus;
use crate::customer_app::app_util::get_app_data_from_req;
use crate::user;
use crate::user::jwt::get_jwt_data_from_param;

mod customer_entities;
pub(crate) mod customer_model;
pub mod customer_util;

pub(crate) async fn register(mut req: Request) -> JRes<CustomerRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	let register_data: CustomerRegisterData = bytes_to_json(&body)?;

	let email = register_data.email.as_str();

	let app_data = get_app_data_from_req(&req)?;

	let email_check = email::check_email(email);

	if email_check == false {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid".to_string(),
			None,
		));
	}

	let registered_user_account = user::user_service::register(app_data, register_data.register_data).await?;
	let customer_id = registered_user_account.user_id;

	//send the normal token via email
	let validate_token = generate_email_validate_token()?;

	customer_model::register_customer(
		register_data.email.to_string(),
		customer_id.to_string(),
		validate_token.to_string(),
	)
	.await?;

	#[cfg(feature = "send_mail")]
	send_mail(email, validate_token, customer_id.to_string()).await;

	let out = CustomerRegisterOutput {
		customer_id,
	};

	echo(out)
}

pub(crate) async fn done_register(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//app id check here because the first req is called from an email via get parameter but to the frontend dashboard.
	//then the dashboard calls this route with an app token
	//the customer must be logged in in the dashboard when sending this req

	let body = get_raw_body(&mut req).await?;
	let input: CustomerDoneRegistrationInput = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let db_token = customer_model::get_email_token(customer_id.to_string()).await?;

	if input.token != db_token.0 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::CustomerEmailTokenValid,
			"Email address is not valid".to_string(),
			None,
		));
	}

	customer_model::done_register(customer_id.to_string()).await?;

	echo_success()
}

pub(crate) async fn prepare_login(mut req: Request) -> JRes<PrepareLoginSaltServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user_identifier: PrepareLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	let out = user::user_service::prepare_login(app_data, user_identifier).await?;

	echo(out)
}

pub(crate) async fn done_login(mut req: Request) -> JRes<CustomerDoneLoginOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	//TODO dont get all keys just the jwt (auth user and jwt creation)
	let user_keys = user::user_service::done_login(app_data, done_login).await?;

	let customer_data = customer_model::get_customer_email_data(user_keys.user_id.to_string()).await?;

	let validate_email = match customer_data.email_valid {
		0 => false,
		1 => true,
		_ => false,
	};

	let out = CustomerDoneLoginOutput {
		user_keys,

		email_data: CustomerEmailData {
			validate_email,
			email: customer_data.email,
			email_send: customer_data.email_send,
			email_status: customer_data.email_status,
		},
	};

	echo(out)
}

pub(crate) async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;

	user::user_service::delete(user).await?;

	customer_model::delete(user.id.to_string()).await?;

	echo_success()
}

//TODO resend email

//TODO email update, with valid

//TODO save real data, e.g. real name or company, address, etc

//TODO reset and change password (check ofr validated email)

//__________________________________________________________________________________________________

/**
Send the validation email.
*/
#[cfg(feature = "send_mail")]
async fn send_mail(email: &str, token: String, customer_id: CustomerId)
{
	//don't wait for the response
	tokio::task::spawn(process_send_mail(email.to_string(), token, customer_id));
}

#[cfg(feature = "send_mail")]
async fn process_send_mail(email: String, token_string: String, customer_id: CustomerId) -> AppRes<()>
{
	let text = format!(
		"Thanks for registration at sentc. Here is your e-mail validation token: {}",
		token_string
	);

	let status = match send_mail_registration(email.as_str(), "Validate email address for sentc", text).await {
		Ok(_) => RegisterEmailStatus::Success,
		Err(e) => {
			match e.api_error_code {
				ApiErrorCodes::EmailMessage => RegisterEmailStatus::FailedMessage(e.msg),
				ApiErrorCodes::EmailSend => RegisterEmailStatus::FailedSend(e.msg),
				_ => RegisterEmailStatus::Other(e.msg),
			}
		},
	};

	customer_model::sent_mail(customer_id, status).await
}

fn generate_email_validate_token() -> AppRes<String>
{
	let mut rng = rand::thread_rng();

	let mut token = [0u8; 30];

	rng.try_fill_bytes(&mut token).map_err(|_| {
		HttpErr::new(
			400,
			ApiErrorCodes::AppTokenWrongFormat,
			"Can't create secret token".to_string(),
			None,
		)
	})?;

	let token_string = base64::encode_config(token, base64::URL_SAFE_NO_PAD);

	Ok(token_string)
}
