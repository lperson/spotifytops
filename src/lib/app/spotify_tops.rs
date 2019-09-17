use hyper::{
    header::{HeaderValue, CONTENT_TYPE},
    Body, Response,
};
extern crate tokio;
use futures::{future::join_all, Future};
use serde::Serialize;
use serde_json;

use std::collections::btree_map::BTreeMap;

use super::super::SpotifyFuture;
use super::super::ThrottlingFuture;
use super::super::app::STATE;
use super::super::server::ResponseFuture;
use super::super::spotify::{
    recently_played_request, top_items_request, RecentlyPlayedResponse, Retriever,
    TopArtistResponse, TopTracksResponse,
};
use super::super::THROTTLE;

#[derive(Serialize)]
struct PresentationData<T>
where
    T: Serialize,
{
    header: String,
    data: T,
}

pub fn handle(auth_code: &str) -> ResponseFuture {
    let the_artists_futures = vec![
        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(
            &auth_code,
            top_items_request::make_request("artists", "short_term"),
        )),
        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(
            &auth_code,
            top_items_request::make_request("artists", "medium_term"),
        )),
        SpotifyFuture::<TopArtistResponse>::new(Retriever::new(
            &auth_code,
            top_items_request::make_request("artists", "long_term"),
        )),
    ];

    let the_tracks_futures = vec![
        SpotifyFuture::<TopTracksResponse>::new(Retriever::new(
            &auth_code,
            top_items_request::make_request("tracks", "short_term"),
        )),
        SpotifyFuture::<TopTracksResponse>::new(Retriever::new(
            &auth_code,
            top_items_request::make_request("tracks", "medium_term"),
        )),
        SpotifyFuture::<TopTracksResponse>::new(Retriever::new(
            &auth_code,
            top_items_request::make_request("tracks", "long_term"),
        )),
    ];

    let the_recently_played_future = SpotifyFuture::<RecentlyPlayedResponse>::new(Retriever::new(
        &auth_code,
        recently_played_request::make_request(),
    ));

    let time_frames = [
        "Short Term (4 weeks)",
        "Medium Term (6 months)",
        "Long Term (years)",
    ];

    let the_future = join_all(the_artists_futures)
        .join(join_all(the_tracks_futures))
        .join(the_recently_played_future)
        .map(move |((artists_results, tracks_results), recently_played_results)| {
            // TODO(lmp) this is a good candidate for macro_rules!
            let artists_results: Vec<PresentationData<TopArtistResponse>> = artists_results
                .iter()
                .zip(time_frames.iter())
                .map(move |(artists_result, header)| PresentationData {
                    header: header.to_string(),
                    data: artists_result.clone(),
                })
                .collect();

            let tracks_results: Vec<PresentationData<TopTracksResponse>> = tracks_results
                .iter()
                .zip(time_frames.iter())
                .map(move |(tracks_result, header)| PresentationData {
                    header: header.to_string(),
                    data: tracks_result.clone(),
                })
                .collect();

            let recently_played_results = PresentationData {
                header: String::from("Last 50 Tracks Played"),
                data: recently_played_results.clone()
            };

            let mut data = BTreeMap::new();
            data.insert("artists", serde_json::to_value(artists_results).unwrap());
            data.insert("tracks", serde_json::to_value(tracks_results).unwrap());
            data.insert("recent", serde_json::to_value(recently_played_results).unwrap());

            let rendered = STATE.handlebars.render("tops", &data).unwrap();

            let mut response = Response::<Body>::new(Body::from(rendered));
            response.headers_mut().insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            response
        })
        .map_err(|x| x);

        let the_future = ThrottlingFuture::new(Box::new(the_future), &THROTTLE);

    Box::new(the_future)
}
