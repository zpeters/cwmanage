//! crate for working with Connectwise Manage API
//!
//! In the connectwise api <https://developer.connectwise.com/Products/Manage> some results are
//! returned as a single 'object' and most are returned as a list.
//! Normally you will be getting a list of results (even a list of one) so you would use [get].
//! In some cases, (/system/info for example) it does not return a list, in this case use [get_single].
//! Consult the api documentation (above) for more details.
//!
//! # Query examples
//! See the connectwise api for further details
//!
//! - No query - `[("", "")]`
//! - Only get the id field `[("fields", "id")]`
//! - Also apply some conditions `[("fields", "id"), ("conditions", "name LIKE '%foo%'")]`
use std::collections::HashMap;
use url::Url;
use serde_json::{Result, Value};

// TODO make this config
/// Do something
const BASE_APIURL: &str = "na.myconnectwise.net";
// TODO make this config
/// Do something
const CODEBASE: &str = "v2020_3";

// *** Structs ***
/// authentication credentials for the connectwise api
/// * `company_id` is your _short name_ (ie the one you use to login to CW)
/// * `public_key` is obtained by creating an api member with keys
/// * `private_key` is obtained by creating an api member with keys
/// * the `client_id` is generated <https://developer.connectwise.com/ClientID>
pub struct Credentials {
    pub company_id: &'static str,
    pub public_key: &'static str,
    pub private_key: &'static str,
    pub client_id: &'static str,
}
// *** Private Functions ***
fn gen_basic_auth(creds: &Credentials) -> String {
    let encoded = base64::encode(format!(
        "{}+{}:{}",
        creds.company_id, creds.public_key, creds.private_key
    ));
    format!("Basic {}", encoded)
}

fn gen_api_url(path: &str) -> String {
    format!("https://{}/{}/apis/3.0{}", BASE_APIURL, CODEBASE, path)
}

fn get_page_id(hdrs: &reqwest::header::HeaderMap) -> String {
    let url = hdrs
        .get("link")
        .unwrap()
        .to_str()
        .unwrap()
        .split("link =")
        .collect::<Vec<&str>>()[0]
        .split('<')
        .collect::<Vec<&str>>()[1]
        .split('>')
        .collect::<Vec<&str>>()[0];

    let parsed_url = Url::parse(url).unwrap();
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

    match hash_query.contains_key("pageId") {
        false => "".to_string(),
        true => hash_query["pageId"].to_string(),
    }
}

// *** Public Functions ***
/// GETs a path from the connectwise api.  `get_single` is only used on certain api endpoints.
/// It is expecting the response from the connectwise api to be a single "object" and not a list
/// like it normally returns
///
/// # Arguments
///
/// - `creds` - your connectwise [Credentials]
/// - `path` - the api path you want to retrieve (example `/service/info`)
/// - `query` - additional query options *must be set*.  If non, use [("", "")]
///
/// # Known Endpoints
///
/// - /system/info
///
/// # Example
///
/// ```
/// use cwmanage::{get_single, Credentials};
/// static TESTING_CREDS: Credentials = Credentials {
///   # company_id:  "buscominctraining",
///   // company_id: "YOURCOMPANY",
///   # public_key: "qIos0KKmMgBOCd2q",
///   // public_key: "YOURPUBLICKEY",
///   # private_key: "tHtksPC80j3FG4df",
///   // private_key: "YOURPRIVATEKEY",
///   # client_id: "a089ca10-d6ea-461a-a274-cf3c1177bde8",
///   // client_id: "YOURCLIENTID",
/// };
/// let query = [("", "")];
/// let path = "/system/info";
/// let result = get_single(&TESTING_CREDS, &path, &query).unwrap();
///
/// assert_eq!(&result["isCloud"], true);
/// ```
pub fn get_single(creds: &Credentials, path: &str, query: &[(&str, &str)]) -> Result<Value> {
    let res = reqwest::blocking::Client::new()
        .get(&gen_api_url(path))
        .header("Authorization", gen_basic_auth(creds))
        .header("Content-Type", "application/json")
        .header("clientid", creds.client_id)
        .header("pagination-type", "forward-only")
        .query(&query)
        .send()
        .unwrap()
        .text()
        .unwrap();

    let v: Value = serde_json::from_str(&res).unwrap();
    Ok(v)
}

/// GETs a path from the connectwise api.  `get` will return *all* results so make sure you
/// set your `query` with the appropriate conditions. This follows the api pagination so, again,
/// *all* results will be returned  For example `/service/tickets` will
/// return **ever** ticket in the system.  The result is a vec of
/// [serde_json::value::Value](https://docs.serde.rs/serde_json/value/enum.Value.html)
///
/// # Arguments
///
/// - `creds` - your connectwise [Credentials]
/// - `path` - the api path you want to retrieve (example `/service/tickets`)
/// - `query` - additional query options *must be set*.  If non, use [("", "")]
/// # Example
///
/// ```
/// use cwmanage::{get, Credentials};
/// static TESTING_CREDS: Credentials = Credentials {
///   # company_id:  "buscominctraining",
///   // company_id: "YOURCOMPANY",
///   # public_key: "qIos0KKmMgBOCd2q",
///   // public_key: "YOURPUBLICKEY",
///   # private_key: "tHtksPC80j3FG4df",
///   // private_key: "YOURPRIVATEKEY",
///   # client_id: "a089ca10-d6ea-461a-a274-cf3c1177bde8",
///   // client_id: "YOURCLIENTID",
/// };
/// let query = [("fields", "id")];
/// let path = "/system/members";
/// let result = get(&TESTING_CREDS, &path, &query).unwrap();
///
/// assert!(result.len() > 30);
/// ```
pub fn get(creds: &Credentials, path: &str, query: &[(&str, &str)]) -> Result<Vec<Value>> {
    let mut collected_res: Vec<Value> = Vec::new();
    let mut page: String = "1".to_string();
    let mut next: bool = true;

    while next {
        let res = reqwest::blocking::Client::new()
            .get(&gen_api_url(path))
            .header("Authorization", gen_basic_auth(creds))
            .header("Content-Type", "application/json")
            .header("clientid", creds.client_id)
            .header("pagination-type", "forward-only")
            .query(&[("pageid", &page)])
            .query(&query)
            .send()
            .unwrap();

        let hdrs = res.headers();

        next = match hdrs.get("link") {
            Some(link) => {
                if link.is_empty() {
                    false
                } else {
                    page = get_page_id(hdrs);
                    true
                }
            }
            None => false,
        };

        let body = res.text().unwrap();
        let mut v: Vec<Value> = serde_json::from_str(&body).unwrap();
        collected_res.append(&mut v);
    }

    Ok(collected_res)
}

// *** Tests ***
#[cfg(test)]
mod tests {
    use super::*;

    // TODO definitely need to do something here
    // dotenv?
    static TESTING_CREDS: Credentials = Credentials {
        company_id: "buscominctraining",
        public_key: "qIos0KKmMgBOCd2q",
        private_key: "tHtksPC80j3FG4df",
        client_id: "a089ca10-d6ea-461a-a274-cf3c1177bde8",
    };

    #[test]
    fn test_basic_auth() {
        let expected: String = "Basic bXljbytwdWI6cHJpdg==".to_string();
        let c: Credentials = Credentials {
            company_id: "myco",
            public_key: "pub",
            private_key: "priv",
            client_id: "something",
        };
        let result = gen_basic_auth(&c);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_gen_url() {
        let expected = "https://na.myconnectwise.net/v2020_3/apis/3.0/system/info";
        let path = "/system/info";
        let result = gen_api_url(path);
        assert_eq!(result, expected);
    }

    // TODO test a failure case
    #[test]
    fn test_basic_get_single() {
        let query = [];
        let result = &get_single(&TESTING_CREDS, "/system/info", &query).unwrap();
        assert_eq!(&result["cloudRegion"], "NA");
        assert_eq!(&result["isCloud"], true);
        assert_eq!(&result["serverTimeZone"], "Eastern Standard Time");
    }

    #[test]
    fn test_basic_get() {
        let query = [];
        let result = &get(&TESTING_CREDS, "/system/members", &query).unwrap();

        assert!(result.len() > 40);

        let zach = &result[0];
        assert_eq!(&zach["adminFlag"], true);
        assert_eq!(&zach["dailyCapacity"], 8.0);
        assert_eq!(&zach["identifier"], "ZPeters");
    }

}
