//! crate for working with Connectwise Manage API
use std::collections::HashMap;
use url::Url;

// TODO make this config
const BASE_APIURL: &str = "na.myconnectwise.net";
// TODO make this config
const CODEBASE: &str = "v2020_3";

// *** Structs ***
/// authentication credentials for the connectwise api
/// * `company_id` is your _short name_ (ie the one you use to login to CW)
/// * `public_key` and `private_key` are obtained by creating an api member with keys
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
pub fn get_one(creds: &Credentials, path: &str, query: &[(&str, &str)]) -> String {
    let res = reqwest::blocking::Client::new()
        .get(&gen_api_url(path))
        .header("Authorization", gen_basic_auth(creds))
        .header("Content-Type", "application/json")
        .header("clientid", creds.client_id)
        .header("pagination-type", "forward-only")
        .query(&query)
        .send()
        .unwrap();

    res.text().unwrap()
}

fn get_all(creds: &Credentials, path: &str, query: &[(&str, &str)]) -> Vec<String> {
    let mut all_results: Vec<String> = Vec::new();
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
        all_results.push(body);
    }
    all_results
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

    // TODO eventually we need a better test here
    #[test]
    fn test_basic_get_one() {
        let query = [];
        let result = &get_one(&TESTING_CREDS, "/system/info", &query);
        assert!(&result.contains("version"));
        assert!(&result.contains("isCloud"));
        assert!(&result.contains("licenseBits"));
    }

    // TODO eventually we need a better test here
    #[test]
    fn test_basic_get_all() {
        let query = [];
        let result = &get_all(&TESTING_CREDS, "/system/members", &query);
        // for now we are just confirming we got multiple 'pages'
        assert!(&result.len() > &5);
    }
}
