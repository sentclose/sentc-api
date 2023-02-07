use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::res::AppRes;

use crate::customer::customer_model;
use crate::util::api_res::ApiErrorCodes;

pub(crate) async fn check_customer_valid(customer_id: &str) -> AppRes<()>
{
	let valid = customer_model::check_customer_valid(customer_id).await?;

	if valid.0 == 0 {
		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailValidate,
			"The e-mail was never validate",
		));
	}

	Ok(())
}
