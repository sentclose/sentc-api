use sentc_crypto_common::{AppId, CustomerId};
use server_api_common::customer::CustomerUpdateInput;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};
use crate::core::db::{exec, query_first, query_string};
use crate::core::get_time;
#[cfg(feature = "send_mail")]
use crate::customer::customer_entities::RegisterEmailStatus;
use crate::customer::customer_entities::{
	CustomerAppList,
	CustomerDataByEmailEntity,
	CustomerDataEntity,
	CustomerEmailByToken,
	CustomerEmailToken,
	CustomerEmailValid,
};
use crate::set_params;

pub(super) async fn check_customer_valid(customer_id: CustomerId) -> AppRes<CustomerEmailValid>
{
	//language=SQL
	let sql = "SELECT email_validate FROM sentc_customer WHERE id = ?";

	let valid: Option<CustomerEmailValid> = query_first(sql, set_params!(customer_id)).await?;

	let valid = match valid {
		Some(v) => v,
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::CustomerNotFound,
				"No account found for this id".to_string(),
				None,
			))
		},
	};

	Ok(valid)
}

pub(super) async fn register_customer(email: String, customer_id: CustomerId, validate_token: String) -> AppRes<()>
{
	//customer id comes from the user register before

	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_customer (id, email, email_validate_sent, email_validate, email_status, email_token) VALUES (?,?,?,?,?,?)";

	#[cfg(feature = "send_mail")]
	let email_status = 0;
	#[cfg(feature = "send_mail")]
	let email_validate = 0;

	//for testing -> don't send email
	#[cfg(not(feature = "send_mail"))]
	let email_status = 1;
	#[cfg(not(feature = "send_mail"))]
	let email_validate = 1;

	exec(
		sql,
		set_params!(
			customer_id,
			email,
			time.to_string(),
			email_validate,
			email_status,
			validate_token
		),
	)
	.await?;

	Ok(())
}

#[cfg(feature = "send_mail")]
pub(super) async fn sent_mail(customer_id: CustomerId, status: RegisterEmailStatus) -> AppRes<()>
{
	let (status, err) = match status {
		RegisterEmailStatus::Success => (1, None),
		RegisterEmailStatus::FailedMessage(err) => (2, Some(err)),
		RegisterEmailStatus::FailedSend(err) => (3, Some(err)),
		RegisterEmailStatus::Other(err) => (4, Some(err)),
	};

	//language=SQL
	let sql = "UPDATE sentc_customer SET email_status = ?, email_error_msg = ? WHERE id = ?";

	exec(sql, set_params!(status, err, customer_id)).await?;

	Ok(())
}

pub(super) async fn done_register(customer_id: CustomerId) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_customer SET email_validate = 1, email_status = 1 WHERE id = ?";

	exec(sql, set_params!(customer_id)).await?;

	Ok(())
}

pub(super) async fn get_email_token(customer_id: CustomerId) -> AppRes<CustomerEmailToken>
{
	//language=SQL
	let sql = "SELECT email_token, email FROM sentc_customer WHERE id = ?";

	let token: Option<CustomerEmailToken> = query_first(sql, set_params!(customer_id)).await?;

	match token {
		Some(t) => Ok(t),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::CustomerNotFound,
				"No token found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_email_by_token(email: String) -> AppRes<CustomerEmailByToken>
{
	//language=SQL
	let sql = "SELECT email, id FROM sentc_customer WHERE email_token = ?";

	let token: Option<CustomerEmailByToken> = query_first(sql, set_params!(email)).await?;

	match token {
		Some(t) => Ok(t),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::CustomerNotFound,
				"No token found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_customer_email_data(customer_id: CustomerId) -> AppRes<CustomerDataEntity>
{
	//language=SQL
	let sql = "SELECT email,email_validate, email_validate_sent, email_status FROM sentc_customer WHERE id= ?";

	let customer: Option<CustomerDataEntity> = query_first(sql, set_params!(customer_id)).await?;

	match customer {
		Some(c) => Ok(c),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::CustomerNotFound,
				"Customer not found".to_string(),
				None,
			))
		},
	}
}

pub(super) async fn get_customer_email_data_by_email(email: String) -> AppRes<CustomerDataByEmailEntity>
{
	//language=SQL
	let sql = "SELECT id, email_validate, email_validate_sent, email_status FROM sentc_customer WHERE email= ?";

	let customer: Option<CustomerDataByEmailEntity> = query_first(sql, set_params!(email)).await?;

	match customer {
		Some(c) => Ok(c),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::CustomerNotFound,
				"Customer not found".to_string(),
				None,
			))
		},
	}
}

//__________________________________________________________________________________________________

pub(super) async fn delete(customer_id: CustomerId) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_customer WHERE id = ?";

	exec(sql, set_params!(customer_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn update(data: CustomerUpdateInput, customer_id: CustomerId, validate_token: String) -> AppRes<()>
{
	let time = get_time()?;

	#[cfg(feature = "send_mail")]
	let email_status = 0;
	#[cfg(feature = "send_mail")]
	let email_validate = 0;

	//for testing -> don't send email
	#[cfg(not(feature = "send_mail"))]
	let email_status = 1;
	#[cfg(not(feature = "send_mail"))]
	let email_validate = 1;

	//language=SQL
	let sql = r"
UPDATE sentc_customer 
SET 
    email = ?, 
    email_validate_sent = ?, 
    email_validate = ?, 
    email_status = ?, 
    email_token = ? 
WHERE id = ?";

	exec(
		sql,
		set_params!(
			data.new_email,
			time.to_string(),
			email_validate,
			email_status,
			validate_token,
			customer_id,
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn reset_password_token_save(customer_id: CustomerId, validate_token: String) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_customer SET email_token = ? WHERE id = ?";

	exec(sql, set_params!(validate_token, customer_id)).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn get_all_apps(customer_id: CustomerId, last_fetched_time: u128, last_app_id: AppId) -> AppRes<Vec<CustomerAppList>>
{
	//language=SQL
	let sql = "SELECT id,identifier, time FROM app WHERE customer_id = ?".to_string();

	let (sql, params) = if last_fetched_time > 0 {
		let sql = sql + " AND time >=? AND (time > ? OR (time = ? AND id > ?)) ORDER BY time, id LIMIT 20";
		(
			sql,
			set_params!(
				customer_id,
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_fetched_time.to_string(),
				last_app_id
			),
		)
	} else {
		let sql = sql + " ORDER BY time, id LIMIT 20";
		(sql, set_params!(customer_id))
	};

	let list: Vec<CustomerAppList> = query_string(sql, params).await?;

	Ok(list)
}
