use rand::RngCore;
use rustgram::Request;
use sentc_crypto_common::user::{
	CaptchaCreateOutput,
	ChangePasswordData,
	DoneLoginLightServerOutput,
	DoneLoginServerInput,
	JwtRefreshInput,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	UserUpdateServerInput,
};
use server_api_common::customer::{
	CustomerData,
	CustomerDoneLoginOutput,
	CustomerDonePasswordResetInput,
	CustomerDoneRegistrationInput,
	CustomerRegisterData,
	CustomerRegisterOutput,
	CustomerResetPasswordInput,
	CustomerUpdateInput,
};
use server_core::email;
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::res::{echo, echo_success, AppRes, JRes, ServerSuccessOutput};

use crate::customer::customer_model;
#[cfg(feature = "send_mail")]
use crate::customer::{send_mail, EmailTopic};
use crate::customer_app::app_util::get_app_data_from_req;
use crate::file::file_service;
use crate::user;
use crate::user::jwt::get_jwt_data_from_param;
use crate::util::api_res::ApiErrorCodes;

pub async fn customer_captcha(req: Request) -> JRes<CaptchaCreateOutput>
{
	//in extra controller fn because we need the internal app id
	let app_data = get_app_data_from_req(&req)?;

	let (id, png) = user::captcha::captcha(&app_data.app_data.app_id).await?;

	echo(CaptchaCreateOutput {
		captcha_id: id,
		png,
	})
}

pub async fn register(mut req: Request) -> JRes<CustomerRegisterOutput>
{
	let body = get_raw_body(&mut req).await?;

	let register_data: CustomerRegisterData = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	//check the captcha
	user::captcha::validate_captcha(
		&app_data.app_data.app_id,
		register_data.captcha_input.captcha_id,
		register_data.captcha_input.captcha_solution,
	)
	.await?;

	let email = register_data.email.as_str();

	let email_check = email::check_email(email);

	if !email_check {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid",
		));
	}

	let (customer_id, _) = user::user_service::register_light(&app_data.app_data.app_id, register_data.register_data).await?;

	//send the normal token via email
	let validate_token = generate_email_validate_token()?;

	customer_model::register_customer(
		&register_data.email,
		register_data.customer_data,
		&customer_id,
		&validate_token,
	)
	.await?;

	#[cfg(feature = "send_mail")]
	send_mail::send_mail(email, validate_token, &customer_id, EmailTopic::Register).await;

	let out = CustomerRegisterOutput {
		customer_id,
	};

	echo(out)
}

pub async fn done_register(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//the first req is called from an email via get parameter but to the frontend dashboard.
	//then the dashboard calls this route with an app token
	//the customer must be logged in in the dashboard when sending this req

	let body = get_raw_body(&mut req).await?;
	let input: CustomerDoneRegistrationInput = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let db_token = customer_model::get_email_token(customer_id).await?;

	if input.token != db_token.email_token {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailTokenValid,
			"Email address is not valid",
		));
	}

	customer_model::done_register(customer_id).await?;

	echo_success()
}

pub async fn resend_email(req: Request) -> JRes<ServerSuccessOutput>
{
	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let _token = customer_model::get_email_token(customer_id).await?;

	#[cfg(feature = "send_mail")]
	send_mail::send_mail(_token.email, _token.email_token, customer_id, EmailTopic::Register).await;

	echo_success()
}

//__________________________________________________________________________________________________

pub async fn prepare_login(mut req: Request) -> JRes<PrepareLoginSaltServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let user_identifier: PrepareLoginServerInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	let out = user::user_service::prepare_login(app_data, &user_identifier.user_identifier).await?;

	echo(out)
}

pub async fn done_login(mut req: Request) -> JRes<CustomerDoneLoginOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	let user_keys = user::user_service::done_login_light(app_data, done_login).await?;

	let customer_data = customer_model::get_customer_data(&user_keys.user_id).await?;

	let out = CustomerDoneLoginOutput {
		user_keys,
		email_data: customer_data.into(),
	};

	echo(out)
}

pub(crate) async fn refresh_jwt(mut req: Request) -> JRes<DoneLoginLightServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: JwtRefreshInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	let out = user::user_service::refresh_jwt(app_data, &user.device_id, input).await?;

	echo(out)
}

//__________________________________________________________________________________________________

pub async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;

	let app_data = get_app_data_from_req(&req)?;

	user::user_service::delete(user, &app_data.app_data.app_id).await?;

	//files must be deleted before customer delete. we need the apps for the customer
	file_service::delete_file_for_customer(user.id.as_str()).await?;

	customer_model::delete(&user.id).await?;

	echo_success()
}

//__________________________________________________________________________________________________

pub async fn update(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//call done register again when validate the token

	let body = get_raw_body(&mut req).await?;
	let update_data: CustomerUpdateInput = bytes_to_json(&body)?;

	let email = update_data.new_email.to_string();

	let email_check = email::check_email(email.as_str());

	if !email_check {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid",
		));
	}

	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	//update in user table too
	user::user_service::update(
		user,
		&app_data.app_data.app_id,
		UserUpdateServerInput {
			user_identifier: email.to_string(),
		},
	)
	.await?;

	let validate_token = generate_email_validate_token()?;

	customer_model::update(update_data, &user.id, validate_token.to_string()).await?;

	#[cfg(feature = "send_mail")]
	send_mail::send_mail(email, validate_token, &user.id, EmailTopic::EmailUpdate).await;

	echo_success()
}

//__________________________________________________________________________________________________

pub async fn change_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//with a fresh jwt
	let body = get_raw_body(&mut req).await?;
	let update_data: ChangePasswordData = bytes_to_json(&body)?;

	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	//the jwt can only be created at our backend
	user::user_service::change_password(user, &app_data.app_data.app_id, update_data).await?;

	echo_success()
}

pub async fn prepare_reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//create a token. this is send to the email. if no valid email -> no password reset!
	//no jwt check because the user needs a pw to login to get a jwt

	let body = get_raw_body(&mut req).await?;
	let data: CustomerResetPasswordInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	//check the captcha
	user::captcha::validate_captcha(
		&app_data.app_data.app_id,
		data.captcha_input.captcha_id,
		data.captcha_input.captcha_solution,
	)
	.await?;

	let email = &data.email;

	let email_check = email::check_email(email);

	if !email_check {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid",
		));
	}

	//model will notify the user if the email is not found
	let email_data = customer_model::get_customer_email_data_by_email(email).await?;

	if email_data.email_valid == 0 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailValidate,
			"E-mail address is not active. Please validate your email first.",
		));
	}

	let validate_token = generate_email_validate_token()?;

	customer_model::reset_password_token_save(&email_data.id, &validate_token).await?;

	#[cfg(feature = "send_mail")]
	send_mail::send_mail(email, validate_token, email_data.id, EmailTopic::PwReset).await;

	echo_success()
}

pub async fn done_reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//this is called when the user clicks on the email link and gets send to the pw reset page (token in get param).
	//then call this fn from the frontend with the token and the new password.

	let body = get_raw_body(&mut req).await?;
	let input: CustomerDonePasswordResetInput = bytes_to_json(&body)?;

	let token_data = customer_model::get_email_by_token(input.token).await?;

	user::user_service::reset_password(&token_data.id, &token_data.device_id, input.reset_password_data).await?;

	echo_success()
}

pub async fn update_data(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: CustomerData = bytes_to_json(&body)?;

	let user = get_jwt_data_from_param(&req)?;

	customer_model::update_data(&user.id, input).await?;

	echo_success()
}

//__________________________________________________________________________________________________

fn generate_email_validate_token() -> AppRes<String>
{
	let mut rng = rand::thread_rng();

	let mut token = [0u8; 30];

	rng.try_fill_bytes(&mut token)
		.map_err(|_| SentcCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create email token"))?;

	let token_string = base64::encode_config(token, base64::URL_SAFE_NO_PAD);

	Ok(token_string)
}
