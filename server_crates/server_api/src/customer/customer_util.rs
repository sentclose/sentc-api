use rustgram_server_util::error::{ServerCoreError, ServerErrorConstructor};
use rustgram_server_util::res::AppRes;
use sentc_crypto_common::CustomerId;

use crate::customer::customer_model;
use crate::util::api_res::ApiErrorCodes;

pub(crate) async fn check_customer_valid(customer_id: impl Into<CustomerId>) -> AppRes<()>
{
	let valid = customer_model::check_customer_valid(customer_id).await?;

	if valid.0 == 0 {
		return Err(ServerCoreError::new_msg(
			400,
			ApiErrorCodes::CustomerEmailValidate,
			"The e-mail was never validate",
		));
	}

	Ok(())
}
