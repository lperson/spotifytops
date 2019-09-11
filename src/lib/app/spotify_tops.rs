use hyper::{Body, Request, Response};

extern crate tokio;

use futures::{future::join_all, prelude::*, Future};

use simple_error::SimpleError;

use std::collections::btree_map::BTreeMap;

use serde_json;

use super::super::server;

use super::super::spotify_future::SpotifyFuture;

use super::super::app::STATE;
use super::super::spotify::Retriever;
use super::super::spotify::TopArtistResponse;
use super::super::spotify::TopTrackResponse;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

pub fn handle(req: &Request<Body>) -> BoxFut {
    let parameters = server::get_query(&req).unwrap();
    let token = &parameters.get("t").unwrap().clone().unwrap();

    let mut auth_code: Option<String> = None;
    {
        let mut tokens = STATE.tokens.lock().unwrap();
        if let Some(stored_auth_code) = tokens.get(token) {
            auth_code = Some(stored_auth_code.clone());
        }
    }

    if auth_code == None {
        // TODO(lmp) -- redirect to / or send a not authorized
    }

    let auth_code = auth_code.unwrap();

    let the_artists_futures = vec![
        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(
            &auth_code,
            "artists",
            "short_term",
        )),
        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(
            &auth_code,
            "artists",
            "medium_term",
        )),
        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(&auth_code, "artists", "long_term")),
    ];

    let the_tracks_futures = vec![
        SpotifyFuture::<TopTrackResponse>::new(Retriever::new(&auth_code, "tracks", "short_term")),
        SpotifyFuture::<TopTrackResponse>::new(Retriever::new(&auth_code, "tracks", "medium_term")),
        SpotifyFuture::<TopTrackResponse>::new(Retriever::new(&auth_code, "tracks", "long_term")),
    ];

    let the_future = join_all(the_artists_futures)
        .join(join_all(the_tracks_futures))
        .map(|(artists_results, tracks_results)| {
            let mut data = BTreeMap::new();
            data.insert("test", serde_json::to_value("Hello Larry").unwrap());
            data.insert("artists", serde_json::to_value(artists_results).unwrap());
            data.insert("tracks", serde_json::to_value(tracks_results).unwrap());

            let rendered = STATE.handlebars.render("tops", &data).unwrap();

            Response::<Body>::new(Body::from(rendered))
        })
        .map_err(|x| x);

    Box::new(the_future)
}
