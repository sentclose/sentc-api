use server_api::start_app;

#[tokio::main]
pub async fn main()
{
	start_app().await;
}
