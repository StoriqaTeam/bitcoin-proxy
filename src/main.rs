#[macro_use]
extern crate clap;
extern crate bitcoin_proxy_lib;

use clap::App;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    if let Some(_) = matches.subcommand_matches("config") {
        bitcoin_proxy_lib::print_config();
    } else if let Some(_) = matches.subcommand_matches("server") {
        bitcoin_proxy_lib::start_server();
    } else {
        let _ = app.print_help();
        println!("\n")
    }
}
