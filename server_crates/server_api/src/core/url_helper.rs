use rustgram::{Request, RouteParams};

use crate::core::api_res::{ApiErrorCodes, HttpErr};

pub fn get_params(req: &Request) -> Result<&RouteParams, HttpErr>
{
	match req.extensions().get::<RouteParams>() {
		Some(p) => Ok(p),
		None => {
			Err(HttpErr::new(
				400,
				ApiErrorCodes::NoParameter,
				"No parameter sent",
				None,
			))
		},
	}
}

pub fn get_name_param_from_req<'a>(req: &'a Request, name: &str) -> Result<&'a str, HttpErr>
{
	let params = get_params(&req)?;

	match params.get(name) {
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::NoParameter,
				"Parameter not found",
				None,
			))
		},
		Some(n) => Ok(n),
	}
}

pub fn get_name_param_from_params<'a>(params: &'a RouteParams, name: &str) -> Result<&'a str, HttpErr>
{
	//this is useful if we need more than one params, so we don't need to get it from req multiple times
	match params.get(name) {
		None => {
			return Err(HttpErr::new(
				400,
				ApiErrorCodes::NoParameter,
				"Parameter not found",
				None,
			))
		},
		Some(n) => Ok(n),
	}
}
