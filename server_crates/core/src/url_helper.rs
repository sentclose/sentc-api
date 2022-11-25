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
