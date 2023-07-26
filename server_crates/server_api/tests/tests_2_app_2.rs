//this test is about creating app in customer group

use hyper::header::AUTHORIZATION;
use reqwest::StatusCode;
use sentc_crypto::sdk_utils::error::SdkUtilError;
use sentc_crypto::util::public::{handle_general_server_response, handle_server_response};
use sentc_crypto::SdkError;
use sentc_crypto_common::group::{GroupChangeRankServerInput, GroupCreateOutput, GroupNewMemberLightInput};
use sentc_crypto_common::UserId;
use serde_json::to_string;
use server_api_common::app::{AppDetails, AppJwtRegisterOutput, AppOptions, AppRegisterInput, AppRegisterOutput};
use server_api_common::customer::{
	CustomerAppList,
	CustomerDoneLoginOutput,
	CustomerGroupCreateInput,
	CustomerGroupList,
	CustomerGroupMemberFetch,
	CustomerGroupView,
};
use tokio::sync::{OnceCell, RwLock};

use crate::test_fn::{auth_header, create_test_customer, customer_delete, get_url};

mod test_fn;

pub struct CustomerState
{
	id: UserId,
	customer_data: CustomerDoneLoginOutput,
}

static CUSTOMER_STATE: OnceCell<RwLock<Vec<CustomerState>>> = OnceCell::const_new();

static GROUP_STATE: OnceCell<RwLock<Vec<String>>> = OnceCell::const_new();

pub struct AppState
{
	pub app_id: String,
	pub app_public_token: String,
	pub app_secret_token: String,
	pub jwt_data: Option<Vec<AppJwtRegisterOutput>>,
}

static APP_TEST_STATE: OnceCell<RwLock<Vec<AppState>>> = OnceCell::const_new();

#[tokio::test]
async fn aaa_init_global_test()
{
	dotenv::from_filename("sentc.env").ok();

	let mut customers = Vec::with_capacity(3);

	for i in 0..3 {
		let username = "hi".to_string() + i.to_string().as_str() + "@test2.com";

		let (id, customer_data) = create_test_customer(&username, "12345").await;

		customers.push(CustomerState {
			id,
			customer_data,
		})
	}

	CUSTOMER_STATE
		.get_or_init(|| async move { RwLock::new(customers) })
		.await;
}

#[tokio::test]
async fn test_10_create_group()
{
	let creator = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let client = reqwest::Client::new();
	let res = client
		.post(get_url("api/v1/customer/group".to_owned()))
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.body(
			to_string(&CustomerGroupCreateInput {
				name: None,
				des: None,
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: GroupCreateOutput = handle_server_response(&body).unwrap();
	let group_id = out.group_id;

	//try to fetch te group

	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/".to_owned() + &group_id))
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: CustomerGroupView = handle_server_response(&body).unwrap();

	assert_eq!(out.data.rank, 0);
	assert_eq!(out.data.group_name, None);
	assert!(out.apps.is_empty());

	GROUP_STATE
		.get_or_init(|| async move { RwLock::new(vec![out.data.id]) })
		.await;

	//fetch all groups
	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/all/0/none".to_owned()))
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerGroupList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, group_id);

	//fetch 2nd page
	let client = reqwest::Client::new();
	let res = client
		.get(get_url(
			"api/v1/customer/group/all/".to_owned() + out[0].time.to_string().as_str() + "/" + &out[0].id,
		))
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerGroupList> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_11_create_app_in_group()
{
	let creator = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &creator[0];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let url = get_url("api/v1/customer/app/".to_owned() + group_id);

	let input = AppRegisterInput {
		identifier: Some("My app".to_string()),
		options: AppOptions::default(),
		file_options: Default::default(),
		group_options: Default::default(),
	};

	let client = reqwest::Client::new();
	let res = client
		.post(url)
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.body(input.to_string().unwrap())
		.send()
		.await
		.unwrap();

	assert_eq!(res.status(), StatusCode::OK);

	let body = res.text().await.unwrap();
	let out: AppRegisterOutput = handle_server_response(&body).unwrap();

	APP_TEST_STATE
		.get_or_init(|| {
			async move {
				RwLock::new(vec![AppState {
					app_id: out.app_id,
					app_public_token: out.public_token,
					app_secret_token: out.secret_token,
					jwt_data: Some(vec![out.jwt_data]),
				}])
			}
		})
		.await;
}

#[tokio::test]
async fn test_12_get_all_apps_for_group()
{
	let creator = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &creator[0];
	let customer_jwt = &creator.customer_data.verify.jwt;

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let apps = APP_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/apps/0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerAppList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_name, None);
	assert_eq!(out[0].id, apps[0].app_id);

	//test 2nd page

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/apps/" + out[0].time.to_string().as_str() + "/" + &out[0].id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerAppList> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_13_invite_new_member_to_group()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/invite/" + &users[1].id);

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.body(
			to_string(&GroupNewMemberLightInput {
				rank: None,
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_13_z_get_group_member_list()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[0];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let client = reqwest::Client::new();
	let res = client
		.get(get_url(
			"api/v1/customer/group/".to_owned() + group_id + "/member/0/none",
		))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: CustomerGroupMemberFetch = handle_server_response(&body).unwrap();

	assert_eq!(out.group_member.len(), 1);
	assert_eq!(out.customer_data.len(), 1);

	assert_eq!(out.customer_data[0].id, *users[1].id);
}

#[tokio::test]
async fn test_14_new_member_should_fetch_the_group()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[1];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	//try to fetch the group

	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/".to_owned() + group_id))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: CustomerGroupView = handle_server_response(&body).unwrap();

	assert_eq!(out.data.rank, 4); //default rank
	assert_eq!(out.data.group_name, None);
	assert_eq!(out.apps.len(), 1); //the new app

	//fetch all groups
	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/all/0/none".to_owned()))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerGroupList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].id, *group_id);

	//fetch 2nd page
	let client = reqwest::Client::new();
	let res = client
		.get(get_url(
			"api/v1/customer/group/all/".to_owned() + out[0].time.to_string().as_str() + "/" + &out[0].id,
		))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerGroupList> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_15_get_all_apps_for_group_as_new_user()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[1];
	let customer_jwt = &user.customer_data.verify.jwt;

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let apps = APP_TEST_STATE.get().unwrap().read().await;

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/apps/0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerAppList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 1);
	assert_eq!(out[0].group_name, None);
	assert_eq!(out[0].id, apps[0].app_id);

	//test 2nd page

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/apps/" + out[0].time.to_string().as_str() + "/" + &out[0].id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerAppList> = handle_server_response(&body).unwrap();
	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_16_access_single_app_for_new_member()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[1];
	let customer_jwt = &user.customer_data.verify.jwt;

	let apps = APP_TEST_STATE.get().unwrap().read().await;
	let app_id = &apps[0].app_id;

	let url = get_url("api/v1/customer/app/".to_owned() + app_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(customer_jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	let out: AppDetails = handle_server_response(body.as_str()).unwrap();

	assert_eq!(out.options.group_list, 1);
}

#[tokio::test]
async fn test_17_not_access_group_without_access()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[2];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	//try to fetch the group

	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/".to_owned() + group_id))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<CustomerGroupView>(&body) {
		Ok(_) => panic!("Should be an error"),
		Err(e) => {
			if let SdkError::Util(SdkUtilError::ServerErr(s, _)) = e {
				assert_eq!(s, 310);
			} else {
				panic!("Should be server error")
			}
		},
	}

	//fetch all groups
	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/all/0/none".to_owned()))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerGroupList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_18_not_access_group_apps_without_access()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[2];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/apps/0/none");
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<CustomerGroupView>(&body) {
		Ok(_) => panic!("Should be an error"),
		Err(e) => {
			if let SdkError::Util(SdkUtilError::ServerErr(s, _)) = e {
				assert_eq!(s, 310);
			} else {
				panic!("Should be server error")
			}
		},
	}

	//not access single app

	let apps = APP_TEST_STATE.get().unwrap().read().await;
	let app_id = &apps[0].app_id;

	let url = get_url("api/v1/customer/app/".to_owned() + app_id);
	let client = reqwest::Client::new();
	let res = client
		.get(url)
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<AppDetails>(&body) {
		Ok(_) => panic!("Should be an error"),
		Err(e) => {
			if let SdkError::Util(SdkUtilError::ServerErr(s, _)) = e {
				assert_eq!(s, 200);
			} else {
				panic!("Should be server error")
			}
		},
	}
}

#[tokio::test]
async fn test_19_change_user_rank()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/change_rank");

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.body(
			to_string(&GroupChangeRankServerInput {
				changed_user_id: user.id.clone(),
				new_rank: 2,
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_20_new_member_should_fetch_the_group_with_new_rank()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[1];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	//try to fetch the group

	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/".to_owned() + group_id))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: CustomerGroupView = handle_server_response(&body).unwrap();

	assert_eq!(out.data.rank, 2); //default rank
}

#[tokio::test]
async fn test_21_kick_member_from_group()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &users[0];
	let user = &users[1];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/kick/" + &user.id);

	let client = reqwest::Client::new();
	let res = client
		.delete(url)
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();
}

#[tokio::test]
async fn test_22_kicked_member_should_not_fetch_the_group()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let user = &users[1];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	//try to fetch the group

	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/".to_owned() + group_id))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();

	match handle_server_response::<CustomerGroupView>(&body) {
		Ok(_) => panic!("Should be an error"),
		Err(e) => {
			if let SdkError::Util(SdkUtilError::ServerErr(s, _)) = e {
				assert_eq!(s, 310);
			} else {
				panic!("Should be server error")
			}
		},
	}

	//fetch all groups
	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/all/0/none".to_owned()))
		.header(AUTHORIZATION, auth_header(&user.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: Vec<CustomerGroupList> = handle_server_response(&body).unwrap();

	assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn test_23_update_group()
{
	let users = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &users[0];

	let groups = GROUP_STATE.get().unwrap().read().await;
	let group_id = &groups[0];

	let url = get_url("api/v1/customer/group/".to_owned() + group_id + "/update");

	let client = reqwest::Client::new();
	let res = client
		.put(url)
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.body(
			to_string(&CustomerGroupCreateInput {
				des: Some("Hello".to_string()),
				name: Some("Hi".to_string()),
			})
			.unwrap(),
		)
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	handle_general_server_response(&body).unwrap();

	//fetch the group with new values
	let client = reqwest::Client::new();
	let res = client
		.get(get_url("api/v1/customer/group/".to_owned() + group_id))
		.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
		.send()
		.await
		.unwrap();

	let body = res.text().await.unwrap();
	let out: CustomerGroupView = handle_server_response(&body).unwrap();

	assert_eq!(out.data.group_name, Some("Hi".to_string()));
	assert_eq!(out.data.des, Some("Hello".to_string()));
}

#[tokio::test]
async fn zzz_clean_up()
{
	let groups = GROUP_STATE.get().unwrap().read().await;

	let customer = CUSTOMER_STATE.get().unwrap().read().await;
	let creator = &customer[0];

	for group in groups.iter() {
		//delete customer group
		let client = reqwest::Client::new();
		let res = client
			.delete(get_url("api/v1/customer/group/".to_owned() + group))
			.header(AUTHORIZATION, auth_header(&creator.customer_data.verify.jwt))
			.send()
			.await
			.unwrap();

		let body = res.text().await.unwrap();
		handle_general_server_response(&body).unwrap();
	}

	for c in customer.iter() {
		customer_delete(&c.customer_data.verify.jwt).await;
	}
}
