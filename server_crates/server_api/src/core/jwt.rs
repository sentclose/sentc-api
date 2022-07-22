use std::str::FromStr;

use jsonwebtoken::{decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sentc_crypto_common::UserId;
use serde::{Deserialize, Serialize};

use crate::core::api_err::{ApiErrorCodes, HttpErr};
use crate::core::get_time_in_sec;
use crate::customer::customer_entities::CustomerAppJwt;
use crate::user::user_entities::UserJwtEntity;

#[derive(Debug, Serialize, Deserialize)]
struct Claims
{
	//jwt defaults
	aud: String,
	sub: String,
	exp: usize,
	iat: usize,

	//sentc
	internal_user_id: UserId,
	user_identifier: String,
}

pub async fn create_jwt(
	internal_user_id: &str,
	user_identifier: &str,
	app_id: &str,
	customer_jwt_data: &CustomerAppJwt,
	aud: &str,
) -> Result<String, HttpErr>
{
	let iat = get_time_in_sec()?;
	let expiration = iat + 60 * 5; //exp in 5 min

	let claims = Claims {
		iat: iat as usize,
		aud: aud.to_string(),
		sub: app_id.to_string(),
		exp: expiration as usize,
		internal_user_id: internal_user_id.to_string(),
		user_identifier: user_identifier.to_string(),
	};

	let mut header = Header::new(Algorithm::from_str(customer_jwt_data.jwt_alg.as_str()).unwrap());
	header.kid = Some(customer_jwt_data.jwt_key_id.to_string());

	//TODO get it from the db (no cache for the sign key)
	let sign_key = "abc";

	encode(&header, &claims, &EncodingKey::from_ec_der(sign_key.as_bytes())).map_err(|e| {
		HttpErr::new(
			401,
			ApiErrorCodes::JwtCreation,
			"Can't create jwt",
			Some(format!("err in jwt creation: {}", e)),
		)
	})
}

pub async fn auth(jwt: &str, check_exp: bool) -> Result<(UserJwtEntity, usize), HttpErr>
{
	let header = decode_header(jwt).map_err(|_e| HttpErr::new(401, ApiErrorCodes::JwtWrongFormat, "Can't decode the jwt", None))?;
	let key_id = match header.kid {
		Some(k) => k,
		None => return Err(HttpErr::new(401, ApiErrorCodes::JwtWrongFormat, "Can't decode the jwt", None)),
	};
	let alg = header.alg;

	//TODO get the verify key from the db (no cache here because we would got extreme big cache for each app, and we may get the jwt from cache too)
	let verify_key = "abc";

	let mut validation = Validation::new(alg);
	validation.validate_exp = check_exp;

	let decoded = decode::<Claims>(jwt, &DecodingKey::from_ec_der(verify_key.as_bytes()), &validation)
		.map_err(|_e| HttpErr::new(401, ApiErrorCodes::JwtValidation, "Wrong jwt", None))?;

	Ok((
		UserJwtEntity {
			id: decoded.claims.internal_user_id,
			identifier: decoded.claims.user_identifier,
			aud: decoded.claims.aud,
		},
		decoded.claims.exp,
	))
}
