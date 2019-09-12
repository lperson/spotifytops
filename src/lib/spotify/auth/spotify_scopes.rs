use std::vec::Vec;

fn sort_vector<T: std::cmp::Ord + Clone>(vector: &mut Vec<T>) -> Vec<T> {
    vector.sort();
    vector.clone()
}

lazy_static! {
    pub static ref SPOTIFY_SCOPES: std::vec::Vec<String> = sort_vector(
        &mut ([
            "user-read-recently-played",
            "user-top-read",
            "user-read-currently-playing",
            "user-read-email",
            "user-read-birthdate",
            "user-read-private",
            "user-library-read",
            "playlist-read-collaborative",
            "playlist-read-private",
            "playlist-modify-private",
            "playlist-modify-public",
        ]
        .to_vec()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
    );
}

pub fn get_scopes() -> String {
    SPOTIFY_SCOPES.join(" ")
}

pub fn are_scopes_current() -> bool {
    true
}
