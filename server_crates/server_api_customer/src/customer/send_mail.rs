use std::env;

use rustgram_server_util::res::AppRes;

use crate::customer::customer_entities::RegisterEmailStatus;
use crate::customer::{customer_model, EmailTopic};
use crate::email::send_mail::send_mail_registration;

/**
Send the validation email.
 */
pub(super) async fn send_mail(email: impl Into<String>, token: String, customer_id: impl Into<sentc_crypto_common::CustomerId>, topic: EmailTopic)
{
	//don't wait for the response
	tokio::task::spawn(process_send_mail(email.into(), customer_id.into(), token, topic));
}

async fn process_send_mail(email: String, customer_id: sentc_crypto_common::CustomerId, token: String, topic: EmailTopic) -> AppRes<()>
{
	let (text_body, html, title) = get_text(token, topic);

	let status = match send_mail_registration(email.as_str(), title, text_body, html).await {
		Ok(_) => RegisterEmailStatus::Success,
		Err(e) => {
			match e.error_code {
				51 /* ApiErrorCodes::EmailMessage */ => RegisterEmailStatus::FailedMessage(e.msg.to_string()),
				50 /* ApiErrorCodes::EmailSend */ => RegisterEmailStatus::FailedSend(e.msg.to_string()),
				_ => RegisterEmailStatus::Other(e.msg.to_string()),
			}
		},
	};

	customer_model::sent_mail(customer_id, status).await
}

fn get_text(token: String, topic: EmailTopic) -> (String, String, &'static str)
{
	let url = env::var("PUBLIC_URL").unwrap();

	let (text, title, url) = match topic {
		EmailTopic::Register => {
			(
				"Thanks for registration at sentc. Please verify your Email.",
				"Sentc Email validation for registration",
				url + "/dashboard/customer/validation/register",
			)
		},
		EmailTopic::PwReset => {
			(
				"Your forgot your password at sentc? Please verify your Email before resetting the password.",
				"Sentc Password reset",
				url + "/dashboard/customer/validation/pw_reset",
			)
		},
		EmailTopic::EmailUpdate => {
			(
				"You updated your Email address for sentc. Please verify your new Email address.",
				"Sentc Email update",
				url + "/dashboard/customer/validation/register",
			)
		},
	};

	let text_body = format!(
		r"{}
Go to {}/?token={} or enter your token: {}
	",
		text, url, token, token
	);

	//language=HTML
	let html = format!(
		r#"<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Sentc {}</title>
</head>
<body>
	<div style="display: flex; flex-direction: column; align-items: center">
		<div style="text-align: left;">
			<h1>{}</h1>
	
			<p>
				{}
			</p>
	
			<p>
				<a href="{}/?token={}">Click here</a> 
				<br>
				<br>
				Or enter your token: <br>
				{}
			</p>
		</div>
	</div>
</body>
</html>"#,
		title, title, text, url, token, token
	);

	(text_body, html, title)
}
