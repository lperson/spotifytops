use super::super::CONFIG;
use handlebars::Handlebars;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

use std::sync::{ Mutex, Arc, RwLock };

use super::super::mru_hashmap::MruHashmap;

lazy_static! {
    pub static ref STATE: State = Default::default();
}

pub struct State {
    pub http_client: Client<HttpsConnector<HttpConnector>>,
    pub handlebars: Handlebars,
    pub tokens: Arc<RwLock<MruHashmap<String, String>>>,
}

impl Default for State {
    fn default() -> Self {
        let https = HttpsConnector::new(4).unwrap();
        let http_client = Client::builder().build::<_, hyper::Body>(https);

        let tokens = Arc::new(RwLock::new(MruHashmap::with_capacity(250)));

        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_file(
                "top_artists",
                format!("{}/top_artists.hbs", CONFIG.template_dir),
            )
            .unwrap();
        handlebars
            .register_template_file(
                "top_tracks",
                format!("{}/top_tracks.hbs", CONFIG.template_dir),
            )
            .unwrap();
        handlebars
            .register_template_file("tops", format!("{}/tops.hbs", CONFIG.template_dir))
            .unwrap();
        handlebars
            .register_template_file("recently_played", format!("{}/recently_played.hbs", CONFIG.template_dir))
            .unwrap();

        State {
            http_client,
            handlebars,
            tokens
        }
    }
}
