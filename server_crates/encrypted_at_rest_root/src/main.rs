use std::env;

fn main()
{
	//load the env
	dotenv::from_filename("sentc.env").ok();

	let args: Vec<String> = env::args().collect();

	if let Some(a) = args.get(1) {
		match a.as_str() {
			"add" => add_new_key(),
			_ => add_new_key(),
		}
	} else {
		add_new_key()
	}
}

fn add_new_key()
{
	let new_key_string = encrypted_at_rest_root::generate_and_export_new_key();

	println!("new key: ");
	println!("{}", new_key_string);
}
