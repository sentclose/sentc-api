use rustgram::{r, Router};
use server_api::sentc_file_controller::{download_part, get_file, get_file_in_group, register_file, register_file_in_group, upload_part};

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
		"/api/v1/file/:file_id",
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

	//file for a group
	router.post(
		"/api/v1/group/:group_id/file",
		r(register_file_in_group)
			.add(server_api::sentc_group_mw)
			.add(server_api::sentc_jwt_mw)
			.add(server_api::sentc_app_mw),
	);

	router.get(
		"/api/v1/group/:group_id/file/:file_id",
		r(get_file_in_group)
			.add(server_api::sentc_group_mw)
			.add(server_api::sentc_jwt_mw)
			.add(server_api::sentc_app_mw),
	);
}
