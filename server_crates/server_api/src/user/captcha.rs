use base64::encode;
use captcha::{gen, Difficulty};
use server_core::error::{SentcCoreError, SentcErrorConstructor};
use server_core::get_time;
use server_core::res::AppRes;

use crate::user::user_model;
use crate::util::api_res::ApiErrorCodes;

pub async fn captcha(app_id: &str) -> AppRes<(String, String)>
{
	let (solution, png) = create_captcha()?;
	let id = user_model::save_captcha_solution(app_id, solution).await?;

	Ok((id, png))
}

pub async fn validate_captcha(app_id: &str, captcha_id: String, solution: String) -> AppRes<()>
{
	let captcha = match user_model::get_captcha_solution(&captcha_id, app_id).await? {
		Some(c) => c,
		None => {
			return Err(SentcCoreError::new_msg(
				400,
				ApiErrorCodes::CaptchaNotFound,
				"Captcha not found",
			))
		},
	};

	//captcha is 20 min valid
	if captcha.time + (1000 * 20 * 60) < get_time()? {
		user_model::delete_captcha(app_id, captcha_id).await?;

		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CaptchaTooOld,
			"Captcha is too old, please do the captcha again",
		));
	}

	if captcha.solution != solution {
		user_model::delete_captcha(app_id, captcha_id).await?;

		return Err(SentcCoreError::new_msg(
			400,
			ApiErrorCodes::CaptchaWrong,
			"Captcha is wrong",
		));
	}

	Ok(())
}

fn create_captcha() -> AppRes<(String, String)>
{
	let (solution, png) = gen(Difficulty::Easy)
		.as_tuple()
		.ok_or_else(|| SentcCoreError::new_msg(400, ApiErrorCodes::CaptchaCreate, "Can't create a captcha"))?;

	let png = encode(png);

	Ok((solution, png))
}
