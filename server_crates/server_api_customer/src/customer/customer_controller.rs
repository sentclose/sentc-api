use std::env;

use rand::RngCore;
use rustgram::Request;
use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::input_helper::{bytes_to_json, get_raw_body};
use rustgram_server_util::res::{echo, echo_success, AppRes, JRes, ServerSuccessOutput};
use rustgram_server_util::url_helper::{get_name_param_from_params, get_name_param_from_req, get_params, get_time_from_url_param};
use sentc_crypto_common::group::{GroupChangeRankServerInput, GroupCreateOutput, GroupNewMemberLightInput};
use sentc_crypto_common::user::{
	CaptchaCreateOutput,
	ChangePasswordData,
	DoneLoginLightServerOutput,
	DoneLoginServerInput,
	JwtRefreshInput,
	OtpInput,
	OtpRecoveryKeysOutput,
	OtpRegister,
	PrepareLoginSaltServerOutput,
	PrepareLoginServerInput,
	UserDeviceRegisterInput,
	UserUpdateServerInput,
	VerifyLoginInput,
};
use server_api::sentc_group_user_service::NewUserType;
use server_api::sentc_user_entities::{DoneLoginServerOutput, DoneLoginServerReturn};
use server_api::{sentc_auth_service, sentc_group_service, sentc_group_user_service, sentc_user_light_service, sentc_user_service};
use server_api_common::customer_app::get_app_data_from_req;
use server_api_common::group::{get_group_user_data_from_req, GROUP_TYPE_NORMAL};
use server_api_common::user::get_jwt_data_from_param;
use server_api_common::SENTC_ROOT_APP;
use server_dashboard_common::customer::{
	CustomerAppList,
	CustomerData,
	CustomerDoneLoginOutput,
	CustomerDonePasswordResetInput,
	CustomerDoneRegistrationInput,
	CustomerGroupCreateInput,
	CustomerGroupList,
	CustomerGroupView,
	CustomerRegisterData,
	CustomerRegisterOutput,
	CustomerResetPasswordInput,
	CustomerUpdateInput,
};

use crate::customer::customer_entities::CustomerGroupMemberFetch;
use crate::customer::{customer_model, customer_util};
#[cfg(feature = "send_mail")]
use crate::customer::{send_mail, EmailTopic};
use crate::customer_app::app_service;
use crate::{captcha, email, ApiErrorCodes};

pub async fn customer_captcha(req: Request) -> JRes<CaptchaCreateOutput>
{
	//in extra controller fn because we need the internal app id
	let app_data = get_app_data_from_req(&req)?;

	let (id, png) = captcha::captcha(&app_data.app_data.app_id).await?;

	echo(CaptchaCreateOutput {
		captcha_id: id,
		png,
	})
}

pub async fn register(mut req: Request) -> JRes<CustomerRegisterOutput>
{
	let register_enabled = env::var("CUSTOMER_REGISTER").unwrap_or_else(|_| "1".into());

	if register_enabled.as_str() != "1" && register_enabled.as_str() != "true" {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerDisable,
			"Register is disabled.",
		));
	}

	let body = get_raw_body(&mut req).await?;

	let register_data: CustomerRegisterData = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	//check the captcha
	captcha::validate_captcha(
		&app_data.app_data.app_id,
		register_data.captcha_input.captcha_id,
		register_data.captcha_input.captcha_solution,
	)
	.await?;

	let email = register_data.email.as_str();

	let email_check = email::check_email(email);

	if !email_check {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid",
		));
	}

	let out = sentc_user_light_service::register_light(&app_data.app_data.app_id, register_data.register_data, false).await?;
	let customer_id = out.user_id;

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
	//the customer must be logged in the dashboard when sending this req

	let body = get_raw_body(&mut req).await?;
	let input: CustomerDoneRegistrationInput = bytes_to_json(&body)?;

	let customer = get_jwt_data_from_param(&req)?;
	let customer_id = &customer.id;

	let db_token = customer_model::get_email_token(customer_id).await?;

	if input.token != db_token.email_token {
		return Err(ServerCoreError::new_msg(
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

	let out = sentc_auth_service::prepare_login(app_data, &user_identifier.user_identifier).await?;

	echo(out)
}

pub async fn done_login(mut req: Request) -> JRes<DoneLoginServerReturn>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: DoneLoginServerInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	let out = sentc_auth_service::done_login(app_data, done_login).await?;

	echo(out)
}

pub async fn validate_mfa(mut req: Request) -> JRes<DoneLoginServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: OtpInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	let out = sentc_auth_service::validate_mfa(app_data, input).await?;

	echo(out)
}

pub async fn validate_recovery_otp(mut req: Request) -> JRes<DoneLoginServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: OtpInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	let out = sentc_auth_service::validate_recovery_otp(app_data, input).await?;

	echo(out)
}

pub async fn verify_login(mut req: Request) -> JRes<CustomerDoneLoginOutput>
{
	let body = get_raw_body(&mut req).await?;
	let done_login: VerifyLoginInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;

	let (verify, data) = sentc_user_light_service::verify_login_light(app_data, done_login).await?;

	let customer_data = customer_model::get_customer_data(&data.user_id).await?;

	echo(CustomerDoneLoginOutput {
		verify,
		email_data: customer_data.into(),
	})
}

pub async fn refresh_jwt(mut req: Request) -> JRes<DoneLoginLightServerOutput>
{
	let body = get_raw_body(&mut req).await?;
	let input: JwtRefreshInput = bytes_to_json(&body)?;
	let app_data = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	let out = sentc_user_service::refresh_jwt(app_data, &user.device_id, input).await?;

	echo(out)
}

//__________________________________________________________________________________________________

pub async fn delete(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;

	let app_data = get_app_data_from_req(&req)?;

	sentc_user_service::delete(user, &app_data.app_data.app_id).await?;

	//files must be deleted before customer delete. we need the apps for the customer
	server_api_common::file::delete_file_for_customer(user.id.as_str()).await?;

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
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid",
		));
	}

	let user = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	//update in user table too
	sentc_user_service::update(
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
	sentc_user_service::change_password(user, &app_data.app_data.app_id, update_data).await?;

	echo_success()
}

pub async fn prepare_reset_password(mut req: Request) -> JRes<ServerSuccessOutput>
{
	//create a token. this is sent to the email. if no valid email -> no password reset!
	//no jwt check because the user needs a pw to log in to get a jwt

	let body = get_raw_body(&mut req).await?;
	let data: CustomerResetPasswordInput = bytes_to_json(&body)?;

	let app_data = get_app_data_from_req(&req)?;

	//check the captcha
	captcha::validate_captcha(
		&app_data.app_data.app_id,
		data.captcha_input.captcha_id,
		data.captcha_input.captcha_solution,
	)
	.await?;

	let email = &data.email;

	let email_check = email::check_email(email);

	if !email_check {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailSyntax,
			"E-mail address is not valid",
		));
	}

	//model will notify the user if the email is not found
	let email_data = customer_model::get_customer_email_data_by_email(email).await?;

	if email_data.email_valid == 0 {
		return Err(ServerCoreError::new_msg(
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

	sentc_user_light_service::reset_password_light(
		SENTC_ROOT_APP,
		UserDeviceRegisterInput {
			master_key: input.reset_password_data.master_key,
			derived: input.reset_password_data.derived,
			device_identifier: token_data.email,
		},
	)
	.await?;

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
//otp

pub async fn register_otp(req: Request) -> JRes<OtpRegister>
{
	let customer = get_jwt_data_from_param(&req)?;
	let app_data = get_app_data_from_req(&req)?;

	let out = sentc_user_service::register_otp(&app_data.app_data.app_id, &customer.id).await?;

	echo(out)
}

pub async fn reset_otp(req: Request) -> JRes<OtpRegister>
{
	let app_data = get_app_data_from_req(&req)?;
	let user = get_jwt_data_from_param(&req)?;

	let out = sentc_user_service::reset_otp(&app_data.app_data.app_id, user).await?;

	echo(out)
}

pub async fn disable_otp(req: Request) -> JRes<ServerSuccessOutput>
{
	let user = get_jwt_data_from_param(&req)?;

	sentc_user_service::disable_otp(user).await?;

	echo_success()
}

pub async fn get_otp_recovery_keys(req: Request) -> JRes<OtpRecoveryKeysOutput>
{
	let user = get_jwt_data_from_param(&req)?;

	let out = sentc_user_service::get_otp_recovery_keys(user).await?;

	echo(out)
}

//__________________________________________________________________________________________________

fn generate_email_validate_token() -> AppRes<String>
{
	let mut rng = rand::thread_rng();

	let mut token = [0u8; 30];

	rng.try_fill_bytes(&mut token)
		.map_err(|_| ServerCoreError::new_msg(400, ApiErrorCodes::AppTokenWrongFormat, "Can't create email token"))?;

	let token_string = base64::encode_config(token, base64::URL_SAFE_NO_PAD);

	Ok(token_string)
}

//__________________________________________________________________________________________________

pub async fn get_all_apps(req: Request) -> JRes<Vec<CustomerAppList>>
{
	let user = get_jwt_data_from_param(&req)?;

	let params = get_params(&req)?;
	let last_app_id = get_name_param_from_params(params, "last_app_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let list = app_service::get_all_apps(&user.id, last_fetched_time, last_app_id).await?;

	echo(list)
}

//__________________________________________________________________________________________________

//customer group

pub async fn create_customer_group(mut req: Request) -> JRes<GroupCreateOutput>
{
	let body = get_raw_body(&mut req).await?;

	let user = get_jwt_data_from_param(&req)?;

	customer_util::check_customer_valid(&user.id).await?;

	let group_id = sentc_group_service::create_group_light(SENTC_ROOT_APP, &user.id, GROUP_TYPE_NORMAL, None, None, None, false).await?;

	let input: CustomerGroupCreateInput = bytes_to_json(&body)?;

	//insert it into the customer table
	customer_model::create_customer_group(&group_id, input).await?;

	echo(GroupCreateOutput {
		group_id,
	})
}

pub async fn get_groups(req: Request) -> JRes<Vec<CustomerGroupList>>
{
	let user = get_jwt_data_from_param(&req)?;

	let params = get_params(&req)?;
	let last_id = get_name_param_from_params(params, "last_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let list = customer_model::get_customer_groups(&user.id, last_fetched_time, last_id).await?;

	echo(list)
}

pub async fn get_group(req: Request) -> JRes<CustomerGroupView>
{
	let user = get_jwt_data_from_param(&req)?;
	let group_data = get_group_user_data_from_req(&req)?;

	let details = customer_model::get_customer_group_details(&user.id, &group_data.group_data.id).await?;

	//fetch the first page of the apps
	let list = app_service::get_all_apps_group(&group_data.group_data.id, 0, "none").await?;

	echo(CustomerGroupView {
		data: details,
		apps: list,
	})
}

pub async fn get_all_apps_group(req: Request) -> JRes<Vec<CustomerAppList>>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_app_id = get_name_param_from_params(params, "last_app_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let list = app_service::get_all_apps_group(&group_data.group_data.id, last_fetched_time, last_app_id).await?;

	echo(list)
}

pub async fn invite_customer_group_member(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let user_to_invite = get_name_param_from_req(&req, "invited_user")?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupNewMemberLightInput = bytes_to_json(&body)?;

	sentc_group_user_service::invite_auto_light(group_data, input, user_to_invite, NewUserType::Normal).await?;

	echo_success()
}

pub async fn delete_customer_group(req: Request) -> JRes<ServerSuccessOutput>
{
	let group_data = get_group_user_data_from_req(&req)?;

	sentc_group_service::delete_group(
		&group_data.group_data.app_id,
		&group_data.group_data.id,
		group_data.user_data.rank,
	)
	.await?;

	server_api_common::file::delete_file_for_customer(&group_data.group_data.id).await?;

	//all apps are deleted via trigger
	customer_model::delete_customer_group(&group_data.group_data.id).await?;

	echo_success()
}

pub async fn delete_group_user(req: Request) -> JRes<ServerSuccessOutput>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let user_to_kick = get_name_param_from_req(&req, "user_id")?;

	sentc_group_user_service::kick_user_from_group(group_data, user_to_kick, false).await?;

	echo_success()
}

pub async fn update_member(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;

	let group_data = get_group_user_data_from_req(&req)?;

	let input: GroupChangeRankServerInput = bytes_to_json(&body)?;

	sentc_group_user_service::change_rank(group_data, input.changed_user_id, input.new_rank).await?;

	echo_success()
}

pub async fn update_group(mut req: Request) -> JRes<ServerSuccessOutput>
{
	let body = get_raw_body(&mut req).await?;
	let group_data = get_group_user_data_from_req(&req)?;
	let input: CustomerGroupCreateInput = bytes_to_json(&body)?;

	customer_model::update_group(&group_data.group_data.id, input).await?;

	echo_success()
}

pub async fn get_group_member_list(req: Request) -> JRes<CustomerGroupMemberFetch>
{
	let group_data = get_group_user_data_from_req(&req)?;

	let params = get_params(&req)?;
	let last_user_id = get_name_param_from_params(params, "last_user_id")?;
	let last_fetched_time = get_name_param_from_params(params, "last_fetched_time")?;
	let last_fetched_time = get_time_from_url_param(last_fetched_time)?;

	let list_fetch = sentc_group_user_service::get_group_member(
		&group_data.group_data.id,
		&group_data.user_data.user_id,
		last_fetched_time,
		last_user_id,
	)
	.await?;

	if list_fetch.is_empty() {
		return echo(CustomerGroupMemberFetch {
			group_member: vec![],
			customer_data: vec![],
		});
	}

	let mut customers = Vec::with_capacity(list_fetch.len());

	for item in &list_fetch {
		customers.push(item.user_id.clone());
	}

	let customer_list = customer_model::get_customers(customers).await?;

	echo(CustomerGroupMemberFetch {
		group_member: list_fetch,
		customer_data: customer_list,
	})
}
