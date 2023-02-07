use server_api_common::customer::{CustomerData, CustomerUpdateInput};
use server_core::db::{exec, query_first, I32Entity};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;
use server_core::{get_time, set_params, str_get, str_t, u128_get};

#[cfg(feature = "send_mail")]
use crate::customer::customer_entities::RegisterEmailStatus;
use crate::customer::customer_entities::{CustomerDataByEmailEntity, CustomerDataEntity, CustomerEmailByToken, CustomerEmailToken};
use crate::util::api_res::ApiErrorCodes;

pub(super) async fn check_customer_valid(customer_id: str_t!()) -> AppRes<I32Entity>
{
	//language=SQL
	let sql = "SELECT email_validate FROM sentc_customer WHERE id = ?";

	let valid: Option<I32Entity> = query_first(sql, set_params!(str_get!(customer_id))).await?;

	let valid = match valid {
		Some(v) => v,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::CustomerNotFound,
				"No account found for this id",
			))
		},
	};

	Ok(valid)
}

pub(super) async fn register_customer(email: str_t!(), data: CustomerData, customer_id: str_t!(), validate_token: str_t!()) -> AppRes<()>
{
	//customer id comes from the user register before

	let time = get_time()?;

	//language=SQL
	let sql = "INSERT INTO sentc_customer (id, email, name, first_name, company, email_validate_sent, email_validate, email_status, email_token) VALUES (?,?,?,?,?,?,?,?,?)";

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
			str_get!(customer_id),
			str_get!(email),
			data.name,
			data.first_name,
			data.company,
			u128_get!(time),
			email_validate,
			email_status,
			str_get!(validate_token)
		),
	)
	.await?;

	Ok(())
}

#[cfg(feature = "send_mail")]
pub(super) async fn sent_mail(customer_id: sentc_crypto_common::CustomerId, status: RegisterEmailStatus) -> AppRes<()>
{
	//owned customer id because it is already owned in the worker because of tokio spawn

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

pub(super) async fn done_register(customer_id: str_t!()) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_customer SET email_validate = 1, email_status = 1 WHERE id = ?";

	exec(sql, set_params!(str_get!(customer_id))).await?;

	Ok(())
}

pub(super) async fn get_email_token(customer_id: str_t!()) -> AppRes<CustomerEmailToken>
{
	//language=SQL
	let sql = "SELECT email_token, email FROM sentc_customer WHERE id = ?";

	let token: Option<CustomerEmailToken> = query_first(sql, set_params!(str_get!(customer_id))).await?;

	match token {
		Some(t) => Ok(t),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::CustomerNotFound,
				"No token found",
			))
		},
	}
}

pub(super) async fn get_email_by_token(email: String) -> AppRes<CustomerEmailByToken>
{
	//language=SQL
	let sql = r"
SELECT email, c.id as customer_id, cd.id as device_id  
FROM 
    sentc_customer c,
    sentc_user_device cd 
WHERE 
    email_token = ? AND 
    c.id = user_id";

	let token: Option<CustomerEmailByToken> = query_first(sql, set_params!(email)).await?;

	match token {
		Some(t) => Ok(t),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::CustomerNotFound,
				"No token found",
			))
		},
	}
}

pub(super) async fn get_customer_data(customer_id: str_t!()) -> AppRes<CustomerDataEntity>
{
	//language=SQL
	let sql = "SELECT email,email_validate, email_validate_sent, email_status, company, first_name, name FROM sentc_customer WHERE id= ?";

	let customer: Option<CustomerDataEntity> = query_first(sql, set_params!(str_get!(customer_id))).await?;

	match customer {
		Some(c) => Ok(c),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::CustomerNotFound,
				"Customer not found",
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
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::CustomerNotFound,
				"Customer not found",
			))
		},
	}
}

//__________________________________________________________________________________________________

pub(super) async fn delete(customer_id: str_t!()) -> AppRes<()>
{
	//language=SQL
	let sql = "DELETE FROM sentc_customer WHERE id = ?";

	exec(sql, set_params!(str_get!(customer_id))).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn update(data: CustomerUpdateInput, customer_id: str_t!(), validate_token: String) -> AppRes<()>
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
			u128_get!(time),
			email_validate,
			email_status,
			validate_token,
			str_get!(customer_id),
		),
	)
	.await?;

	Ok(())
}

pub(super) async fn reset_password_token_save(customer_id: str_t!(), validate_token: str_t!()) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_customer SET email_token = ? WHERE id = ?";

	exec(sql, set_params!(str_get!(validate_token), str_get!(customer_id))).await?;

	Ok(())
}

//__________________________________________________________________________________________________

pub(super) async fn update_data(id: str_t!(), data: CustomerData) -> AppRes<()>
{
	//language=SQL
	let sql = "UPDATE sentc_customer SET name = ?, first_name = ?, company = ? WHERE id = ?";

	exec(
		sql,
		set_params!(data.name, data.first_name, data.company, str_get!(id)),
	)
	.await?;

	Ok(())
}
