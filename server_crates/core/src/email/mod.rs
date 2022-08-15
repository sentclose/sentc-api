#[cfg(feature = "send_mail")]
pub mod send_mail;

use regex::Regex;
use tokio::sync::OnceCell;

static EMAIL_CHECKER: OnceCell<EmailChecker> = OnceCell::const_new();

pub async fn init_email_checker()
{
	EMAIL_CHECKER
		.get_or_init(move || {
			async {
				//init email with env vars
				EmailChecker::new()
			}
		})
		.await;
}

pub fn check_email(email: &str) -> bool
{
	let email_struct = EMAIL_CHECKER.get().unwrap();

	email_struct.check_address(email)
}

struct EmailChecker
{
	email_regex: Regex,
}

impl EmailChecker
{
	pub fn new() -> Self
	{
		//from here: https://stackoverflow.com/questions/201323/how-can-i-validate-an-email-address-using-a-regular-expression
		let email_regex = Regex::new(r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#).unwrap();

		Self {
			email_regex,
		}
	}

	pub fn check_address(&self, email: &str) -> bool
	{
		self.email_regex.is_match(email)
	}
}
