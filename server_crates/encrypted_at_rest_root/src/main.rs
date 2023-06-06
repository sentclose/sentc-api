use std::env;

fn main()
{
	//load the env
	dotenv::dotenv().ok();

	let args: Vec<String> = env::args().collect();

	match args[1].as_str() {
		"add" => add_new_key(),
		"del" => remove_key(args.get(2)),
		_ => panic!("Wrong args, please choose add or del"),
	}
}

fn add_new_key()
{
	let key_string = match env::var("ROOT_KEYS") {
		Ok(s) => s,
		Err(_) => String::new(),
	};

	let new_key_string = encrypted_at_rest_root::generate_and_add_new_key(&key_string);

	println!("new keys: ");
	println!("{:?}", new_key_string);
}

fn remove_key(id: Option<&String>)
{
	let id = if let Some(i) = id {
		i
	} else {
		panic!("No id set to delete");
	};

	let key_string = match env::var("ROOT_KEYS") {
		Ok(s) => s,
		Err(_) => {
			panic!("No keys set");
		},
	};

	let key_string = encrypted_at_rest_root::delete_key(&key_string, id);

	println!("new keys: ");
	println!("{:?}", key_string);
}
