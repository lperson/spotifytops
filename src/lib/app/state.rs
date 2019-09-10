use hyper::{Client, client::HttpConnector};
use hyper_tls::HttpsConnector;
use handlebars::{Handlebars};
use super::super::CONFIG;

lazy_static! {
    pub static ref STATE: State = Default::default();
}

pub struct State {
    pub http_client: Client<HttpsConnector<HttpConnector>>,
    pub handlebars: Handlebars,
}

impl Default for State {
    fn default() -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let http_client = Client::builder().build::<_, hyper::Body>(https);

        let mut handlebars = Handlebars::new();
        handlebars.register_template_file("top_artists", format!("{}/top_artists.hbs", CONFIG.template_dir)).unwrap();
        handlebars.register_template_file("top_tracks", format!("{}/top_artists.hbs", CONFIG.template_dir)).unwrap();
        handlebars.register_template_file("tops", format!("{}/tops.hbs", CONFIG.template_dir)).unwrap();

        State { http_client, handlebars }
    }
}


