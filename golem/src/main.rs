mod config;
mod discord;
mod tcl;

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();

    let mut config_path: String = "./config.yml".to_string();

    println!("{:?}", args);

    if args.len() > 2 {
        if args[1] == "--config" || args[1] == "-c" {
            config_path = args[2].to_string();
        }
    }

    discord::discord_init(&config_path);
}
