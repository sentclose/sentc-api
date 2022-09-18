use rustgram::Request;
use sentc_crypto::sdk_common::PartId;
use server_core::input_helper::{bytes_to_json, get_raw_body};
use server_core::url_helper::get_name_param_from_req;
use tokio::sync::{OnceCell, RwLock};

static PART_IDS: OnceCell<RwLock<Vec<PartId>>> = OnceCell::const_new();

pub(crate) async fn upload_part(req: Request) -> &'static str
{
	let part_id = get_name_param_from_req(&req, "part_id").unwrap();

	let mut parts = PART_IDS
		.get_or_init(|| async move { RwLock::new(Vec::with_capacity(502)) })
		.await
		.write()
		.await;

	parts.push(part_id.to_string());

	""
}

pub(crate) async fn delete(mut req: Request) -> &'static str
{
	println!("delete");

	let body = get_raw_body(&mut req).await.unwrap();

	let input: Vec<PartId> = bytes_to_json(&body).unwrap();

	let ids = PART_IDS
		.get_or_init(|| async move { RwLock::new(Vec::with_capacity(502)) })
		.await
		.write()
		.await;

	println!("input ids: {:?}", input);

	for id in input {
		//every part id must be found
		let res = ids.iter().find(|x| x.as_str() == id.as_str());

		assert_ne!(res, None);
	}

	""
}
