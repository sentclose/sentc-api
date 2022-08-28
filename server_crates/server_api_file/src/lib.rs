use rustgram::{r, Router};
use server_api::sentc_file_controller::{download_part, get_file, register_file, upload_part};

pub fn routes(router: &mut Router)
{
	router.post(
		"/api/v1/file",
		r(register_file)
			.add(server_api::sentc_jwt_mw)
			.add(server_api::sentc_app_mw),
	);

	router.post(
		"/api/v1/file/part/:session_id/:seq/:end",
		r(upload_part)
			.add(server_api::sentc_jwt_mw)
			.add(server_api::sentc_app_mw),
	);

	router.get(
		"/api/v1/file",
		r(get_file)
			.add(server_api::sentc_jwt_mw)
			.add(server_api::sentc_app_mw),
	);

	router.get(
		"/api/v1/file/part/:part_id",
		r(download_part)
			.add(server_api::sentc_jwt_mw)
			.add(server_api::sentc_app_mw),
	);
}
