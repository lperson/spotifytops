use hyper::{
    header::{HeaderValue, CONTENT_TYPE},
    Body, Response,
};

extern crate tokio;

use futures::{future::join_all, Future};

use simple_error::SimpleError;

use std::collections::btree_map::BTreeMap;
use std::rc::Rc;

use serde::Serialize;
use serde_json;

use super::super::spotify_future::SpotifyFuture;

use super::super::app::STATE;
use super::super::spotify::Retriever;
use super::super::spotify::TopArtistResponse;
use super::super::spotify::TopTrackResponse;

#[derive(Serialize)]
struct PresentationData<T>
where
    T: Serialize,
{
    header: String,
    data: T,
}

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = SimpleError> + Send>;

pub fn handle(token: Rc<String>) -> BoxFut {
    let mut auth_code: Option<String> = None;
    {
        let mut tokens = STATE.tokens.lock().unwrap();
        if let Some(stored_auth_code) = tokens.get(&token) {
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

    let time_frames = [
        "Short Term (4 weeks)",
        "Medium Term (6 months)",
        "Long Term (years)",
    ];

    let the_future = join_all(the_artists_futures)
        .join(join_all(the_tracks_futures))
        .map(move |(artists_results, tracks_results)| {
            // TODO(lmp) this is a good candidate for macro_rules!
            let artists_results: Vec<PresentationData<TopArtistResponse>> = artists_results
                .iter()
                .zip(time_frames.iter())
                .map(move |(artists_result, header)| PresentationData {
                    header: header.to_string(),
                    data: artists_result.clone(),
                })
                .collect();

            let tracks_results: Vec<PresentationData<TopTrackResponse>> = tracks_results
                .iter()
                .zip(time_frames.iter())
                .map(move |(tracks_result, header)| PresentationData {
                    header: header.to_string(),
                    data: tracks_result.clone(),
                })
                .collect();

            let mut data = BTreeMap::new();
            data.insert("artists", serde_json::to_value(artists_results).unwrap());
            data.insert("tracks", serde_json::to_value(tracks_results).unwrap());

            let rendered = STATE.handlebars.render("tops", &data).unwrap();

            let mut response = Response::<Body>::new(Body::from(rendered));
            response.headers_mut()
                .insert(
                    CONTENT_TYPE,
                    HeaderValue::from_static("text/html; charset=utf-8")
                );
            response
        })
        .map_err(|x| x);

    Box::new(the_future)
}
