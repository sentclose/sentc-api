use std::collections::HashMap;

use rustgram::{Request, RouteParams};

use crate::error::{CoreErrorCodes, SentcCoreError, SentcErrorConstructor};

pub fn get_params(req: &Request) -> Result<&RouteParams, SentcCoreError>
{
	match req.extensions().get::<RouteParams>() {
		Some(p) => Ok(p),
		None => {
			Err(SentcCoreError::new_msg(
				400,
				CoreErrorCodes::NoParameter,
				"No parameter sent",
			))
		},
	}
}

pub fn get_name_param_from_req<'a>(req: &'a Request, name: &str) -> Result<&'a str, SentcCoreError>
{
	let params = get_params(req)?;

	match params.get(name) {
		None => {
			Err(SentcCoreError::new_msg(
				400,
				CoreErrorCodes::NoParameter,
				"Parameter not found",
			))
		},
		Some(n) => Ok(n),
	}
}

pub fn get_name_param_from_params<'a>(params: &'a RouteParams, name: &str) -> Result<&'a str, SentcCoreError>
{
	//this is useful if we need more than one params, so we don't need to get it from req multiple times
	match params.get(name) {
		None => {
			Err(SentcCoreError::new_msg(
				400,
				CoreErrorCodes::NoParameter,
				"Parameter not found",
			))
		},
		Some(n) => Ok(n),
	}
}

pub fn get_query_params(req: &Request) -> Result<HashMap<String, String>, SentcCoreError>
{
	let query = match req.uri().query() {
		Some(q) => q,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				CoreErrorCodes::NoUrlQuery,
				"Url query not found",
			));
		},
	};

	let params: HashMap<String, String> = query
		.split('&')
		.map(|p| p.split('=').map(|s| s.to_string()).collect::<Vec<String>>())
		.filter(|p| p.len() == 2)
		.map(|p| (p[0].clone(), p[1].clone()))
		.collect();

	Ok(params)
}

pub fn get_time_from_url_param(time: &str) -> Result<u128, SentcCoreError>
{
	time.parse().map_err(|_e| {
		SentcCoreError::new_msg(
			400,
			CoreErrorCodes::UnexpectedTime,
			"Time is wrong. It must be a number",
		)
	})
}
