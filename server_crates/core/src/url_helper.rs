use std::collections::HashMap;

use rustgram::{Request, RouteParams};

use crate::error::{CoreError, CoreErrorCodes};

pub fn get_params(req: &Request) -> Result<&RouteParams, CoreError>
{
	match req.extensions().get::<RouteParams>() {
		Some(p) => Ok(p),
		None => {
			Err(CoreError::new(
				400,
				CoreErrorCodes::NoParameter,
				"No parameter sent".to_owned(),
				None,
			))
		},
	}
}

pub fn get_name_param_from_req<'a>(req: &'a Request, name: &str) -> Result<&'a str, CoreError>
{
	let params = get_params(req)?;

	match params.get(name) {
		None => {
			Err(CoreError::new(
				400,
				CoreErrorCodes::NoParameter,
				"Parameter not found".to_owned(),
				None,
			))
		},
		Some(n) => Ok(n),
	}
}

pub fn get_name_param_from_params<'a>(params: &'a RouteParams, name: &str) -> Result<&'a str, CoreError>
{
	//this is useful if we need more than one params, so we don't need to get it from req multiple times
	match params.get(name) {
		None => {
			Err(CoreError::new(
				400,
				CoreErrorCodes::NoParameter,
				"Parameter not found".to_owned(),
				None,
			))
		},
		Some(n) => Ok(n),
	}
}

pub fn get_query_params(req: &Request) -> Result<HashMap<String, String>, CoreError>
{
	let query = match req.uri().query() {
		Some(q) => q,
		None => {
			return Err(CoreError::new(
				400,
				CoreErrorCodes::NoUrlQuery,
				"Url query not found".to_string(),
				None,
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
