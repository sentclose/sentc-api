use std::env;
use std::future::Future;

use lettre::message::{header, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tokio::sync::OnceCell;

use crate::error::{CoreError, CoreErrorCodes};

static EMAIL_SENDER_REGISTER: OnceCell<Email> = OnceCell::const_new();

pub async fn init_email_register()
{
	let from = format!("Sentc <{}>", env::var("EMAIL_ADDRESS").unwrap());

	EMAIL_SENDER_REGISTER
		.get_or_init(move || {
			async {
				//init email with env vars
				Email::new(from)
			}
		})
		.await;
}

pub fn send_mail_registration<'a>(
	to: &'a str,
	subject: &'a str,
	body_txt: String,
	body_html: String,
) -> impl Future<Output = Result<(), CoreError>> + 'a
{
	let email = EMAIL_SENDER_REGISTER.get().unwrap();

	email.send_email_smpt(to, subject, body_txt, body_html)
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
		let user = env::var("EMAIL_USER").unwrap();
		let pw = env::var("EMAIL_PW").unwrap();
		let server = env::var("EMAIL_SERVER").unwrap();
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

	pub async fn send_email_smpt(&self, to: &str, subject: &str, body_txt: String, body_html: String) -> Result<(), CoreError>
	{
		let smtp_credentials = Credentials::new(self.user.to_string(), self.pw.to_string());
		let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(self.server.as_str())
			.unwrap()
			.credentials(smtp_credentials)
			.port(self.port)
			.build();

		let to = to.parse().map_err(|_e| {
			CoreError::new(
				400,
				CoreErrorCodes::EmailMessage,
				"Error in email message".to_string(),
				None,
			)
		})?;

		let email = Message::builder()
			.from(self.from.clone())
			.to(to)
			.subject(subject)
			.multipart(
				MultiPart::alternative()
					.singlepart(
						//plain text fallback
						SinglePart::builder()
							.header(header::ContentType::TEXT_PLAIN)
							.body(body_txt),
					)
					.singlepart(
						SinglePart::builder()
							.header(header::ContentType::TEXT_HTML)
							.body(body_html),
					),
			)
			.map_err(|_e| {
				CoreError::new(
					400,
					CoreErrorCodes::EmailMessage,
					"Error in email message".to_string(),
					None,
				)
			})?;

		mailer.send(email).await.map_err(|e| {
			CoreError::new(
				400,
				CoreErrorCodes::EmailSend,
				format!("Error in email send: {}", e),
				None,
			)
		})?;

		Ok(())
	}
}
