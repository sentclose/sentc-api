pub mod customer_controller;
pub mod customer_entities;
pub(crate) mod customer_model;
pub mod customer_util;

#[cfg(feature = "send_mail")]
mod send_mail;

#[cfg(feature = "send_mail")]
enum EmailTopic
{
	Register,
	PwReset,
	EmailUpdate,
}
