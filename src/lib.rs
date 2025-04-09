use core::fmt;
//use async_std::task;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

const _MOVIE_FOUND_RESULT: &str = r#"
    {"Title":"The Matrix","Year":"1999","Rated":"R","Released":"31 Mar 1999","Runtime":"136 min","Genre":"Action, Sci-Fi","Director":"Lana Wachowski, Lilly Wachowski","Writer":"Lilly Wachowski, Lana Wachowski","Actors":"Keanu Reeves, Laurence Fishburne, Carrie-Anne Moss","Plot":"When a beautiful stranger leads computer hacker Neo to a forbidding underworld, he discovers the shocking truth--the life he knows is the elaborate deception of an evil cyber-intelligence.","Language":"English","Country":"United States, Australia","Awards":"Won 4 Oscars. 42 wins & 52 nominations total","Poster":"https://m.media-amazon.com/images/M/MV5BN2NmN2VhMTQtMDNiOS00NDlhLTliMjgtODE2ZTY0ODQyNDRhXkEyXkFqcGc@._V1_SX300.jpg","Ratings":[{"Source":"Internet Movie Database","Value":"8.7/10"},{"Source":"Rotten Tomatoes","Value":"83%"},{"Source":"Metacritic","Value":"73/100"}],"Metascore":"73","imdbRating":"8.7","imdbVotes":"2,124,695","imdbID":"tt0133093","Type":"movie","DVD":"N/A","BoxOffice":"$172,076,928","Production":"N/A","Website":"N/A","Response":"True"}
"#;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Default)]
// #[serde(rename_all="PascalCase")]
#[serde(rename_all(serialize = "lowercase", deserialize = "PascalCase"))]
#[serde(default)]
pub struct MovieResult {
    pub title: String,
    pub year: String,
    pub rated: String,
    pub released: String,
    pub runtime: String,
    #[serde(deserialize_with = "split_comma_deserialize")]
    #[serde(serialize_with ="join_comma_serialize")]
    pub genre: Vec<String>,
    #[serde(deserialize_with = "split_comma_deserialize")]
    #[serde(serialize_with ="join_comma_serialize")]
    pub director: Vec<String>,
    #[serde(deserialize_with = "split_comma_deserialize")]
    #[serde(serialize_with ="join_comma_serialize")]
    pub actors: Vec<String>,
    pub plot: String,
    pub language: String,
    pub country: String,
    pub awards: String,
    pub poster: String,
    pub metascore: String,
    #[serde(rename(deserialize="imdbRating", serialize="imdb_rating"))]
    pub imdb_rating: String,
    // #[serde(rename="imdbVotes")]
    #[serde(rename(deserialize="imdbVotes", serialize="imdb_votes"))]
    pub imdb_votes: String,
    // #[serde(rename="imdbID")]
    #[serde(rename(deserialize="imdbID", serialize="imdb_id"))]
    pub imdb_id: String,
    // #[serde(rename="Type")]
    #[serde(rename(deserialize="Type", serialize="picture_type"))]
    pub picture_type: String,
    #[serde(rename(deserialize="DVD", serialize="dvd"))]
    // #[serde(rename="DVD")]
    pub dvd: String,
    pub box_office: String,
    pub production: String,
    pub website: String,
}

#[derive(Debug)]
pub enum OmdbResult {
    MovieResult(MovieResult),
    String(String),
    JsonValue(Value),

}
impl OmdbResult {
    pub fn as_movie_result(self) -> Option<MovieResult> {
        match self {
            OmdbResult::MovieResult(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_string(self) -> Option<String> {
        match self {
            OmdbResult::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_json_value(self) -> Option<Value> {
        match self {
            OmdbResult::JsonValue(v) => Some(v),
            _ => None,
        }
    }
}
fn join_comma_serialize<S>(v: &Vec<String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(v.join(",").as_str())
}

fn split_comma_deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    Ok(str_sequence
        .split(',')
        .map(|item| item.to_owned())
        .collect())
}

pub struct OmdbApi {
    pub search_type: SearchType,
    pub plot_type: PlotType,
    pub response_format: ResponseFormat,
    pub result_format: ResultFormat,
    api_key: String,
    base_url: String,

}

impl Default for OmdbApi {
    fn default() -> Self {
        Self {
            search_type: SearchType::Movie,
            plot_type: PlotType::Short,
            response_format: ResponseFormat::Json,
            result_format: ResultFormat::SerdeJsonTyped,
            api_key: "".into(),
            base_url: "https://www.omdbapi.com".into(),
        }
    }
}

impl OmdbApi {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            ..OmdbApi::default()
        }
    }


    pub fn test_url(&self, title: String) -> Result<OmdbResult, OmdbError> {
        let url = format!(
            "{}/?apikey={}&t={}&type={}&plot={}&r={}",
            self.base_url,
            self.api_key,
            title,
            self.search_type,
            self.plot_type,
            self.response_format,
        );
         Ok(OmdbResult::String(url))
    }

    pub fn get_by_title(&self, title: String) -> Result<OmdbResult, OmdbError> {
        let url = format!(
            "{}/?apikey={}&t={}&type={}&plot={}&r={}",
            self.base_url,
            self.api_key,
            utf8_percent_encode(&title, NON_ALPHANUMERIC).to_string(),
            self.search_type,
            self.plot_type,
            self.response_format,
        );

        let body = ureq::get(&url)
            .header("Api-User-Agent", "thanks for your api")
            .call()?
            .body_mut()
            .read_to_string()?;


        self.parse_response(body, "".into())


    }
    pub fn get_example(&self) -> Result<OmdbResult, OmdbError> {
        self.parse_response(_MOVIE_FOUND_RESULT.into(), "The Matrix".into())
    }
    pub fn get_by_imdb_id(&self, imdb_id: String) -> Result<OmdbResult, OmdbError> {
        let url = format!(
            "{}/?apikey={}&i={}",
            self.base_url,
            self.api_key,
            imdb_id
        );

        let body = ureq::get(&url)
            .header("Api-User-Agent", "thanks for your api")
            .call()?
            .body_mut()
            .read_to_string()?;


        self.parse_response(body, imdb_id)
    }
    pub fn search_by_title(&self, title: String) -> Result<String, OmdbError> {
        let url = format!(
            "{}/?apikey={}&s={}",
            self.base_url,
            self.api_key,
            utf8_percent_encode(&title, NON_ALPHANUMERIC).to_string());

        let body = ureq::get(&url)
            .header("Api-User-Agent", "thanks for your api")
            .call()?
            .body_mut()
            .read_to_string()?;

        Ok(body)

    }

    pub fn parse_response(&self, body_str: String, title_or_id: String) -> Result<OmdbResult, OmdbError> {
        let v: Value = serde_json::from_str(&body_str)?;
        if v["Response"] == "False" {
            if v["Error"] == "Movie not found!" {
                Err(OmdbError::TitleNotFound(title_or_id))
            } else if v["Error"] == "Incorrect IMDb ID." {
                Err(OmdbError::ImdbIdNotFound(title_or_id))
            } else {
                Err(OmdbError::KeyError(v["Error"].to_string()))
            }

        } else {
            match self.result_format {
                ResultFormat::SerdeJsonTyped => {
                    let m = serde_json::from_value(v)?;
                    if self.response_format == ResponseFormat::Json {
                        Ok(OmdbResult::MovieResult(m))
                    } else {
                        Err(OmdbError::OtherError("Incompatible result and response formmat (json/xml)".into()))
                    }

                },
                ResultFormat::RawString => Ok(OmdbResult::String(body_str)),
                ResultFormat::SerdeJsonValue => {
                    if self.response_format == ResponseFormat::Json {
                        Ok(OmdbResult::JsonValue(v))
                    } else {
                        Err(OmdbError::OtherError("Incompatible result and response format (json/xml)".into()))
                    }
                }
            }

        }

    }
    pub fn set_search_type(&mut self, search_type: SearchType) {
        self.search_type = search_type;
    }
    pub fn set_plot_type(&mut self, plot_type: PlotType) {
        self.plot_type = plot_type;
    }
    pub fn set_response_format(&mut self, response_format: ResponseFormat) {
        if response_format == ResponseFormat::Xml {
            panic!("xml response format not implemented");
        } else {
            self.response_format = response_format;
        }

    }
    pub fn set_result_format(&mut self, result_format: ResultFormat) {
        self.result_format = result_format;
    }

}

pub enum ResultFormat {
    SerdeJsonTyped,
    SerdeJsonValue,
    RawString,

}
pub enum SearchType {
    Movie,
    Series,
    Episode,
}

impl fmt::Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Movie => write!(f, "movie"),
            SearchType::Series => write!(f, "series"),
            SearchType::Episode => write!(f, "episode"),
        }
    }
}

pub enum PlotType {
    Short,
    Full,
}

impl fmt::Display for PlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlotType::Short => write!(f, "short"),
            PlotType::Full => write!(f, "full"),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Xml,
}

impl fmt::Display for ResponseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseFormat::Json => write!(f, "json"),
            ResponseFormat::Xml => write!(f, "xml"),
        }
    }
}

#[derive(Debug)]
pub enum OmdbError {
    //SurfError(surf::Error),
    UreqError(ureq::Error),
    SerdeJsonError(serde_json::Error),
    SerdeParseError(String),
    TitleNotFound(String),
    ImdbIdNotFound(String),
    KeyError(String),
    OtherError(String),
    NotImplemented,
}

// impl From<surf::Error> for OmdbError {
//     fn from(err: surf::Error) -> Self {
//         OmdbError::SurfError(err)
//     }
// }
impl From<ureq::Error> for OmdbError {
    fn from(err: ureq::Error) -> Self {
        OmdbError::UreqError(err)
    }
}
impl From<serde_json::Error> for OmdbError {
    fn from(err: serde_json::Error) -> Self {
        OmdbError::SerdeJsonError(err)
    }
}

impl From<String> for OmdbError {
    fn from(err: String) -> Self {
        OmdbError::SerdeParseError(err)
    }
}


#[cfg(test)]
mod tests {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

    use crate::OmdbApi;
    use super::_MOVIE_FOUND_RESULT;



    #[test]
    fn test_get_by_id() {

    }
    #[test]
    #[ignore = "reason"]
    fn test_json_types() {
        let mut o = OmdbApi::new("key".into());
        o.set_plot_type(crate::PlotType::Full);
        o.set_result_format(crate::ResultFormat::SerdeJsonTyped);
        let _r = o.parse_response(_MOVIE_FOUND_RESULT.into(), "The Matrix".into()).unwrap();
        println!("m parse test: {:?}", _r.as_movie_result().unwrap());
        o.set_result_format(crate::ResultFormat::RawString);
        let _r = o.parse_response(_MOVIE_FOUND_RESULT.into(), "The Matrix".into()).unwrap();
        println!("m parse test: {:?}", _r.as_string().unwrap());
        //o.set_response_format(ResponseFormat::Xml);
        o.set_result_format(crate::ResultFormat::SerdeJsonValue);
        let _r = o.parse_response(_MOVIE_FOUND_RESULT.into(), "The Matrix".into()).unwrap();
        println!("m parse test: {:?}", _r.as_json_value().unwrap());

        //let _r = o.get_by_title("themovietitle".into());
    }
    #[test]
    #[ignore = "reason"]
    fn de_and_se_test() {
        let mut o = OmdbApi::new("key".into());
        o.set_result_format(crate::ResultFormat::SerdeJsonTyped);
        let _r = o.parse_response(_MOVIE_FOUND_RESULT.into(), "The Matrix".into()).unwrap().as_movie_result().unwrap();
        println!("parsed movie struct: \n{:?}", _r);
        let json = serde_json::to_string(&_r);
        println!("back to json: \n{:?}", json);

    }
    #[test]
    #[ignore = "reason"]
    fn encode_url() {
        let _url = format!(
            "{}/?apikey={}&t={}&type={}&plot={}&r={}",
            "https://www.omdbapi.com",
            "123",
            "this and that mc'ones {123}",
            "r",
            "short",
            "josn",
        );
        let title = "this and that mc'ones {123}";
        let url_encoded = utf8_percent_encode(&title, NON_ALPHANUMERIC).to_string();
        println!("encoded: {}", url_encoded);
    }


    const _ERROR_NOT_FOUND: &str = r#"
        {"Response":"False","Error":"Movie not found!"}
    "#;

    const _ERROR_INVALID_API_KEY: &str = r#"
        {"Response":"False","Error":"Invalid API key!"}
    "#;

    const _ERROR_NO_API_KEY: &str= r#"
        {"Response":"False","Error":"No API key provided."}
    "#;
}
