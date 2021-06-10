use structopt::StructOpt;
use url::Url;

#[derive(StructOpt, Debug)]
#[structopt(name = "sf-api")]
pub struct Opt {
    #[structopt(
        short = "s",
        long = "node-server",
        default_value = "ws://127.0.0.1:9944"
    )]
    pub node_server: Url,
    #[structopt(short = "l", long = "listen", default_value = "http://127.0.0.1:4000")]
    pub listen: Url,
}

pub fn get_node_url_from_opt() -> String {
    let opt = Opt::from_args();
    opt.node_server.into()
}
