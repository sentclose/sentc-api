use sentc_crypto_common::CustomerId;

use crate::customer::customer_model;
use crate::util::api_res::{ApiErrorCodes, AppRes, HttpErr};

pub(crate) async fn check_customer_valid(customer_id: CustomerId) -> AppRes<()>
{
	let valid = customer_model::check_customer_valid(customer_id).await?;

	if valid.0 == 0 {
		return Err(HttpErr::new(
			400,
			ApiErrorCodes::CustomerEmailValidate,
			"The e-mail was never validate".to_string(),
			None,
		));
	}

	Ok(())
}
