use std::error::Error;
use std::str::FromStr;

use jsonwebtoken::{decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use ring::rand;
use ring::signature::{self, KeyPair};
use rustgram::Request;
use sentc_crypto::sdk_common::AppId;
use sentc_crypto_common::{DeviceId, GroupId, UserId};
use serde::{Deserialize, Serialize};
use server_core::cache::{CacheVariant, LONG_TTL};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::input_helper::{bytes_to_json, json_to_string};
use server_core::res::AppRes;
use server_core::{cache, get_time_in_sec};

use crate::customer_app::app_entities::AppJwt;
use crate::user::user_entities::UserJwtEntity;
use crate::user::{user_model, user_service};
use crate::util::api_res::ApiErrorCodes;
use crate::util::{get_app_jwt_sign_key, get_app_jwt_verify_key, get_user_in_app_key};

pub const JWT_ALG: &str = "ES384";

#[derive(Debug, Serialize, Deserialize)]
struct Claims
{
	//jwt defaults
	aud: UserId,   //the user id
	sub: DeviceId, //the device id
	exp: usize,
	iat: usize,
	fresh: bool, //was this token from refresh jwt or from login
}

#[allow(clippy::collapsible_match)]
pub fn get_jwt_data_from_param(req: &Request) -> Result<&UserJwtEntity, SentcCoreError>
{
	match req.extensions().get::<Option<UserJwtEntity>>() {
		Some(p) => {
			//p should always be some for non optional jwt
			match p {
				Some(p1) => Ok(p1),
				None => {
					Err(SentcCoreError::new_msg(
						400,
						ApiErrorCodes::JwtNotFound,
						"No valid jwt",
					))
				},
			}
		},
		None => {
			Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::JwtNotFound,
				"No valid jwt",
			))
		},
	}
}

pub(crate) async fn create_jwt(internal_user_id: &str, device_id: &str, customer_jwt_data: &AppJwt, fresh: bool) -> Result<String, SentcCoreError>
{
	let iat = get_time_in_sec()?;
	let expiration = iat + 60 * 5; //exp in 5 min

	let claims = Claims {
		iat: iat as usize,
		aud: internal_user_id.into(),
		sub: device_id.into(),
		exp: expiration as usize,
		fresh,
	};

	let mut header = Header::new(Algorithm::from_str(&customer_jwt_data.jwt_alg).unwrap());
	header.kid = Some(customer_jwt_data.jwt_key_id.to_string());

	//get it from the db (no cache for the sign key)
	let sign_key = get_sign_key(&customer_jwt_data.jwt_key_id).await?;
	//decode sign key
	let sign_key = decode_jwt_key(sign_key)?;

	encode(&header, &claims, &EncodingKey::from_ec_der(&sign_key)).map_err(|e| {
		SentcCoreError::new_msg_and_debug(
			401,
			ApiErrorCodes::JwtCreation,
			"Can't create jwt",
			Some(format!("err in jwt creation: {}", e)),
		)
	})
}

pub async fn auth(app_id: AppId, jwt: &str, check_exp: bool) -> Result<(UserJwtEntity, usize), SentcCoreError>
{
	let header = decode_header(jwt).map_err(|_e| SentcCoreError::new_msg(401, ApiErrorCodes::JwtWrongFormat, "Can't decode the jwt"))?;

	let key_id = match header.kid {
		Some(k) => k,
		None => {
			return Err(SentcCoreError::new_msg(
				401,
				ApiErrorCodes::JwtWrongFormat,
				"Can't decode the jwt",
			))
		},
	};
	let alg = header.alg;

	//it is secure when using only the key id without app ref. Only the backend with the right sign key can create a jwt which can be verified by the verify key.
	//so faking the key id but using another sign key for the sign would be an error.

	//use a separate cache for the keys because the validation is done only when the jwt was never cached before (see jwt middleware)
	let verify_key = get_verify_key(&key_id).await?;

	//decode the key
	let verify_key = decode_jwt_key(verify_key)?;

	let mut validation = Validation::new(alg);
	validation.validate_exp = check_exp;

	let decoded = decode::<Claims>(jwt, &DecodingKey::from_ec_der(&verify_key), &validation)
		.map_err(|_e| SentcCoreError::new_msg(401, ApiErrorCodes::JwtValidation, "Wrong jwt"))?;

	//now check if the user is in the app
	//this is necessary because now we check if the values inside the jwt are correct.
	//fetch the device group id too, this id can not be faked and is safe to use internally
	let group_id = get_user_in_app(app_id, &decoded.claims.aud).await?;

	Ok((
		UserJwtEntity {
			id: decoded.claims.aud,
			device_id: decoded.claims.sub,
			group_id,
			fresh: decoded.claims.fresh,
		},
		decoded.claims.exp,
	))
}

pub fn create_jwt_keys() -> Result<(String, String, &'static str), SentcCoreError>
{
	let rng = rand::SystemRandom::new();
	let bytes = signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P384_SHA384_FIXED_SIGNING, &rng).map_err(map_create_key_err)?;

	let keypair = signature::EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P384_SHA384_FIXED_SIGNING, bytes.as_ref()).map_err(map_create_key_err)?;

	let verify_key = keypair.public_key();

	let verify_key = base64::encode(verify_key);
	let keypair = base64::encode(bytes);

	Ok((keypair, verify_key, JWT_ALG))
}

fn decode_jwt_key(key: String) -> Result<Vec<u8>, SentcCoreError>
{
	base64::decode(key).map_err(|_e| SentcCoreError::new_msg(401, ApiErrorCodes::JwtWrongFormat, "Can't decode the jwt"))
}

fn map_create_key_err<E: Error>(e: E) -> SentcCoreError
{
	SentcCoreError::new_msg_and_debug(
		500,
		ApiErrorCodes::JwtKeyCreation,
		"Can't create keys",
		Some(format!("Err in Jwt key creation: {}", e)),
	)
}

async fn get_sign_key(key_id: &str) -> AppRes<String>
{
	//use a separate cache for the keys because the validation is done only when the jwt was never cached before (see jwt middleware)
	let sign_key_cache_key = get_app_jwt_sign_key(key_id);

	match cache::get(&sign_key_cache_key).await {
		Some(c) => {
			match bytes_to_json::<CacheVariant<String>>(c.as_bytes())? {
				CacheVariant::Some(k) => Ok(k),
				CacheVariant::None => {
					Err(SentcCoreError::new_msg(
						200,
						ApiErrorCodes::JwtKeyNotFound,
						"No matched key to this key id",
					))
				},
			}
		},
		None => {
			//key was not in the cache -> search with the model
			match user_model::get_jwt_sign_key(key_id).await? {
				Some(key) => {
					cache::add(
						sign_key_cache_key,
						json_to_string(&CacheVariant::Some(&key.0))?,
						LONG_TTL,
					)
					.await;

					Ok(key.0)
				},
				None => {
					//cache wrong keys too
					cache::add(
						sign_key_cache_key,
						json_to_string(&CacheVariant::<String>::None)?,
						LONG_TTL,
					)
					.await;

					Err(SentcCoreError::new_msg(
						200,
						ApiErrorCodes::JwtKeyNotFound,
						"No matched key to this key id",
					))
				},
			}
		},
	}
}

async fn get_verify_key(key_id: &str) -> AppRes<String>
{
	//use a separate cache for the keys because the validation is done only when the jwt was never cached before (see jwt middleware)
	let verify_key_cache_key = get_app_jwt_verify_key(key_id);

	match cache::get(&verify_key_cache_key).await {
		Some(c) => {
			match bytes_to_json::<CacheVariant<String>>(c.as_bytes())? {
				CacheVariant::Some(k) => Ok(k),
				CacheVariant::None => {
					Err(SentcCoreError::new_msg(
						200,
						ApiErrorCodes::JwtKeyNotFound,
						"No matched key to this key id",
					))
				},
			}
		},
		None => {
			//key was not in the cache -> search with the model
			match user_model::get_jwt_verify_key(key_id).await? {
				Some(key) => {
					cache::add(
						verify_key_cache_key,
						json_to_string(&CacheVariant::Some(&key.0))?,
						LONG_TTL,
					)
					.await;

					Ok(key.0)
				},
				None => {
					//cache wrong keys too
					cache::add(
						verify_key_cache_key,
						json_to_string(&CacheVariant::<String>::None)?,
						LONG_TTL,
					)
					.await;

					Err(SentcCoreError::new_msg(
						200,
						ApiErrorCodes::JwtKeyNotFound,
						"No matched key to this key id",
					))
				},
			}
		},
	}
}

async fn get_user_in_app(app_id: AppId, user_id: &str) -> AppRes<GroupId>
{
	let cache_key = get_user_in_app_key(&app_id, user_id);

	match cache::get(&cache_key).await {
		Some(c) => {
			match bytes_to_json::<CacheVariant<GroupId>>(c.as_bytes())? {
				CacheVariant::Some(k) => Ok(k),
				CacheVariant::None => {
					Err(SentcCoreError::new_msg(
						400,
						ApiErrorCodes::UserNotFound,
						"User not found",
					))
				},
			}
		},
		None => {
			match user_service::get_user_group_id(app_id, user_id).await? {
				Some(u) => {
					cache::add(cache_key, json_to_string(&CacheVariant::Some(&u.0))?, LONG_TTL).await;

					Ok(u.0)
				},
				None => {
					//cache wrong user in app too
					cache::add(cache_key, json_to_string(&CacheVariant::<String>::None)?, LONG_TTL).await;

					Err(SentcCoreError::new_msg(
						400,
						ApiErrorCodes::UserNotFound,
						"User not found",
					))
				},
			}
		},
	}
}

#[cfg(test)]
mod test
{
	use super::*;

	#[test]
	fn test_jwt_key_creation_and_validation()
	{
		let (keypair, verify_key, alg) = create_jwt_keys().unwrap();

		//create a jwt, but raw not with the functions
		let iat = get_time_in_sec().unwrap();
		let expiration = iat + 60 * 5; //exp in 5 min

		let claims = Claims {
			iat: iat as usize,
			aud: "jo".to_string(),
			sub: "12345".to_string(),
			exp: expiration as usize,
			fresh: false,
		};

		let key_id_str = "abc".to_string();

		let mut header = Header::new(Algorithm::from_str(alg).unwrap());
		header.kid = Some(key_id_str.to_string());

		let sign_key = base64::decode(keypair).unwrap();

		let jwt = encode(&header, &claims, &EncodingKey::from_ec_der(&sign_key)).unwrap();

		//auth the jwt
		let header = decode_header(&jwt).unwrap();

		let key_id = match header.kid {
			Some(k) => k,
			None => {
				panic!("kid should be there")
			},
		};
		let alg = header.alg;

		//decode the key
		let verify_key = base64::decode(verify_key).unwrap();

		let mut validation = Validation::new(alg);
		validation.validate_exp = true;

		let decoded = decode::<Claims>(&jwt, &DecodingKey::from_ec_der(&verify_key), &validation).unwrap();

		assert_eq!(decoded.claims.aud, claims.aud);
		assert_eq!(key_id, key_id_str);
		assert!(decoded.claims.fresh);
	}
}
