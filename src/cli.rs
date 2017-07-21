use clap::{App, Arg};

pub fn new<'a, 'b>() -> App<'a, 'b> {
    App::new("hapi")
        .version("0.1.0")
        .author("geauxvirtual")
        .about("API Server")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets custom configuration file")
             .takes_value(true))
}
