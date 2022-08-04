use std::env;
use std::future::Future;

use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tokio::sync::OnceCell;

use crate::core::api_res::{ApiErrorCodes, AppRes, HttpErr};

static EMAIL_SENDER_REGISTER: OnceCell<Email> = OnceCell::const_new();

pub async fn init_email_register()
{
	let from = format!("Sentc registration <{}>", env::var("EMAIL_ADDRESS").unwrap());

	EMAIL_SENDER_REGISTER
		.get_or_init(move || {
			async {
				//init email with env vars
				Email::new(from)
			}
		})
		.await;
}

pub fn send_mail_registration<'a>(to: &'a str, subject: &'a str, body: String) -> impl Future<Output = AppRes<()>> + 'a
{
	let email = EMAIL_SENDER_REGISTER.get().unwrap();

	email.send_email_smpt(to, subject, body)
}

struct Email
{
	user: String,
	pw: String,
	server: String,
	port: u16,
	from: Mailbox,
}

impl Email
{
	pub fn new(from: String) -> Self
	{
		//This is executed at the server start, so unwrap is ok here
		let user = env::var("EMAIL_ADDRESS").unwrap();
		let pw = env::var("EMAIL_ADDRESS").unwrap();
		let server = env::var("EMAIL_ADDRESS").unwrap();
		let port = env::var("EMAIL_PORT").unwrap().parse().unwrap();
		let from = from.parse().unwrap();

		Self {
			user,
			pw,
			server,
			port,
			from,
		}
	}

	pub async fn send_email_smpt(&self, to: &str, subject: &str, body: String) -> AppRes<()>
	{
		let smtp_credentials = Credentials::new(self.user.to_string(), self.pw.to_string());
		let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(self.server.as_str())
			.unwrap()
			.credentials(smtp_credentials)
			.port(self.port)
			.build();

		let to = to.parse().map_err(|_e| {
			HttpErr::new(
				400,
				ApiErrorCodes::EmailMessage,
				"Error in email message".to_string(),
				None,
			)
		})?;

		let email = Message::builder()
			.from(self.from.clone())
			.to(to)
			.subject(subject)
			.body(body)
			.map_err(|_e| {
				HttpErr::new(
					400,
					ApiErrorCodes::EmailMessage,
					"Error in email message".to_string(),
					None,
				)
			})?;

		mailer.send(email).await.map_err(|e| {
			HttpErr::new(
				400,
				ApiErrorCodes::EmailSend,
				format!("Error in email send: {}", e),
				None,
			)
		})?;

		Ok(())
	}
}
