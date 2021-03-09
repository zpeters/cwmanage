#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
//! crate for working with Connectwise Manage API
//!
//! In the connectwise api <https://developer.connectwise.com/Products/Manage> some results are
//! returned as a single 'object' and most are returned as a list.
//! Normally you will be getting a list of results (even a list of one) so you would use [Client.get].
//! In some cases, (/system/info for example) it does not return a list, in this case use [Client.get_single].
//! Consult the api documentation (above) for more details.
//!
//! # Basic usage
//!
//! Basic client with default api_uri, codebase, and api version
//! ```
//! use cwmanage::Client;
//! use dotenv::dotenv;
//! dotenv().ok();
//! let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
//! let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
//! let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
//! let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
//! let client = Client::new(company_id, public_key, private_key, client_id).build();
//! let query = [("", "")];
//! let result = client.get_single("/system/info", &query).unwrap();
//! ```
//! Override the api_version
//! ```
//! use cwmanage::Client;
//! use dotenv::dotenv;
//! dotenv().ok();
//! let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
//! let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
//! let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
//! let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
//!
//! let client = Client::new(company_id, public_key, private_key, client_id).build();
//! let query = [("", "")];
//! let result = client.get_single("/system/info", &query).unwrap();
//! ```
//!
//! Get an endpoint with multiple results
//! ```
//! use cwmanage::Client;
//! use dotenv::dotenv;
//! dotenv().ok();
//! let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
//! let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
//! let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
//! let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
//! let client = Client::new(company_id, public_key, private_key, client_id).build();
//! let query = [("fields", "id,identifier")];
//! let result = client.get("/system/members", &query);
//! ```
//!
//! # Query examples
//! See the connectwise api for further details
//!
//! - No query - `[("", "")]`
//! - Only get the id field `[("fields", "id")]`
//! - Also apply some conditions `[("fields", "id"), ("conditions", "name LIKE '%foo%'")]`
use serde_json::{Result, Value};
use std::collections::HashMap;
use url::Url;

/// Default api url.  NA for north america.  Adjust to your cloud instance or local instance. See [Client] for how to customize
pub const DEFAULT_API_URL: &str = "na.myconnectwise.net";

/// This is the release version specified in the documentation.  
/// There is a way to dynamically look up your api version.  This
/// might be added in the future. See [Client] for how to customize
pub const DEFAULT_API_CODEBASE: &str = "v4_6_release";

/// I cannot find documentation on this , but since it is a number
/// it is customizable. See [Client] for how to customize
pub const DEFAULT_API_VERSION: &str = "3.0";

/// Connectwise client.  Initinitialize with [Client::new].  Use [Client::api_url],
/// [Client::api_version] and [Client::codebase] to customize.  The finalize with [Client::build]
/// * `company_id` is your _short name_ (ie the one you use to login to CW)
/// * `public_key` is obtained by creating an api member with keys
/// * `private_key` is obtained by creating an api member with keys
/// * the `client_id` is generated <https://developer.connectwise.com/ClientID>
#[derive(Debug, PartialEq)]
pub struct Client {
    company_id: String,
    public_key: String,
    private_key: String,
    client_id: String,
    api_url: String,
    codebase: String,
    api_version: String,
}
impl Client {
    /// Creates a new client using the default values
    pub fn new(
        company_id: String,
        public_key: String,
        private_key: String,
        client_id: String,
    ) -> Client {
        Client {
            company_id,
            public_key,
            private_key,
            client_id,
            api_url: DEFAULT_API_URL.to_string(),
            codebase: DEFAULT_API_CODEBASE.to_string(),
            api_version: DEFAULT_API_VERSION.to_string(),
        }
    }
    /// Builds (finalizes the client)
    pub fn build(&self) -> Client {
        Client {
            company_id: self.company_id.to_owned(),
            public_key: self.public_key.to_owned(),
            private_key: self.private_key.to_owned(),
            client_id: self.client_id.to_owned(),
            api_url: self.api_url.to_owned(),
            codebase: self.codebase.to_owned(),
            api_version: self.api_version.to_owned(),
        }
    }

    /// overrides the default api_version
    pub fn api_version(mut self, api_version: String) -> Client {
        self.api_version = api_version;
        self
    }

    /// overrides the default api_url
    pub fn api_url(mut self, api_url: String) -> Client {
        self.api_url = api_url;
        self
    }

    /// overrides the default codebase
    pub fn codebase(mut self, codebase: String) -> Client {
        self.codebase = codebase;
        self
    }
    fn gen_basic_auth(&self) -> String {
        let encoded = base64::encode(format!(
            "{}+{}:{}",
            self.company_id, self.public_key, self.private_key
        ));
        format!("Basic {}", encoded)
    }
    fn gen_api_url(&self, path: &str) -> String {
        format!(
            "https://{}/{}/apis/{}{}",
            self.api_url, self.codebase, self.api_version, path
        )
    }
    /// GETs a path from the connectwise api.  `get_single` is only used on certain api endpoints.
    /// It is expecting the response from the connectwise api to be a single "object" and not a list
    /// like it normally returns
    ///
    /// # Arguments
    ///
    /// - `path` - the api path you want to retrieve (example `/service/info`)
    /// - `query` - additional query options *must be set*.  If non, use [("", "")]
    ///
    /// # Known Endpoints
    ///
    /// - /system/info
    ///
    /// # Example
    ///
    /// ## Basic get, returning parsed json
    /// ```
    /// use cwmanage::Client;
    ///
    /// // this example is using dotenv to load our settings from
    /// // the environment, you could also specify this manually
    /// use dotenv::dotenv;
    /// dotenv().ok();
    /// let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
    /// let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
    /// let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
    /// let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
    ///
    /// let client = Client::new(company_id, public_key, private_key, client_id).build();
    ///
    /// let query = [("", "")];
    /// let path = "/system/info";
    /// let result = client.get_single(&path, &query).unwrap();
    ///
    /// assert_eq!(&result["isCloud"], true);
    /// ```
    /// ## Basic get, take parsed json and convert to a struct
    /// ```
    /// use cwmanage::Client;
    /// use serde::{Deserialize};
    ///
    /// #[derive(Debug, Deserialize)]
    /// #[serde(rename_all = "camelCase")]
    /// struct SystemInfo {
    ///   version: String,
    ///   is_cloud: bool,
    ///   server_time_zone: String,
    /// }
    ///
    /// // this example is using dotenv to load our settings from
    /// // the environment, you could also specify this manually
    /// use dotenv::dotenv;
    /// dotenv().ok();
    /// let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
    /// let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
    /// let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
    /// let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
    ///
    /// let client = Client::new(company_id, public_key, private_key, client_id).build();
    ///
    /// let query = [("", "")];
    /// let path = "/system/info";
    /// let result = client.get_single(&path, &query).unwrap();
    ///
    /// // got our result, just like before.
    /// // now convert it into our struct
    /// let info: SystemInfo = serde_json::from_value(result).unwrap();
    /// assert_eq!(info.is_cloud, true);
    /// assert_eq!(info.server_time_zone, "Eastern Standard Time");
    /// ```
    pub fn get_single(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let res = reqwest::blocking::Client::new()
            .get(&self.gen_api_url(path))
            .header("Authorization", &self.gen_basic_auth())
            .header("Content-Type", "application/json")
            .header("clientid", self.client_id.to_owned())
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
    /// return **every** ticket in the system.  The result is a vec of
    /// [serde_json::value::Value](https://docs.serde.rs/serde_json/value/enum.Value.html)
    ///
    /// # Arguments
    ///
    /// - `path` - the api path you want to retrieve (example `/service/tickets`)
    /// - `query` - additional query options *must be set*.  If non, use [("", "")]
    /// # Example
    ///
    /// ## Getting all results, returning parsed json
    /// ```
    /// use cwmanage::Client;
    ///
    /// // this example is using dotenv to load our settings from
    /// // the environment, you could also specify this manually
    /// use dotenv::dotenv;
    /// dotenv().ok();
    /// let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
    /// let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
    /// let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
    /// let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
    /// let client = Client::new(company_id, public_key, private_key, client_id).build();
    ///
    /// let query = [("fields", "id")];
    /// let path = "/system/members";
    /// let result = client.get(&path, &query).unwrap();
    ///
    /// assert!(result.len() > 30);
    /// ```
    /// ## Getting all results, take parsed json and convert to a struct
    /// ```
    /// use cwmanage::Client;
    /// use serde::{Deserialize};
    /// use serde_json::Value::Array;
    ///
    /// #[derive(Debug, Deserialize)]
    /// #[serde(rename_all = "camelCase")]
    /// struct Member {
    ///   id: i32,
    ///   identifier: String,
    /// }
    ///
    /// // this example is using dotenv to load our settings from
    /// // the environment, you could also specify this manually
    /// use dotenv::dotenv;
    /// dotenv().ok();
    /// let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
    /// let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
    /// let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
    /// let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
    /// let client = Client::new(company_id, public_key, private_key, client_id).build();
    ///
    /// let query = [("", "")];
    /// let path = "/system/members";
    /// let result = client.get(&path, &query).unwrap();
    ///
    /// // got our result, just like before.
    /// // now convert it into our struct
    /// let members: Vec<Member>= serde_json::from_value(Array(result)).unwrap();
    /// assert_eq!(members.len(), 134);
    /// ```

    // pub fn get_single(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
    //     let res = reqwest::blocking::Client::new()
    pub fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Vec<Value>> {
        let mut collected_res: Vec<Value> = Vec::new();
        let mut page: String = "1".to_string();
        let mut next: bool = true;

        while next {
            let res = reqwest::blocking::Client::new()
                .get(&self.gen_api_url(path))
                .header("Authorization", self.gen_basic_auth())
                .header("Content-Type", "application/json")
                .header("clientid", self.client_id.to_owned())
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
}

// *** Private Functions ***
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

// *** Tests ***
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    fn testing_client() -> Client {
        dotenv().ok();
        let company_id: String =
            dotenv::var("CWMANAGE_COMPANY_ID").expect("CWMANAGE_COMPANY_ID needs to be set");
        let public_key: String =
            dotenv::var("CWMANAGE_PUBLIC_KEY").expect("CWMANAGE_PUBLIC_KEY needs to be set");
        let private_key: String =
            dotenv::var("CWMANAGE_PRIVATE_KEY").expect("CWMANAGE_PRIVATE_KEY needs to be set");
        let client_id: String =
            dotenv::var("CWMANAGE_CLIENT_ID").expect("CWMANAGE_CLIENT_ID needs to be set");
        Client::new(company_id, public_key, private_key, client_id).build()
    }

    #[test]
    fn test_basic_auth() {
        let expected: String = "Basic bXljbytwdWI6cHJpdg==".to_string();
        let client = Client::new(
            String::from("myco"),
            String::from("pub"),
            String::from("priv"),
            String::from("something"),
        )
        .build();
        let result = client.gen_basic_auth();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_gen_url() {
        let expected = "https://na.myconnectwise.net/v4_6_release/apis/3.0/system/info";
        let client = Client::new(
            String::from("myco"),
            String::from("pub"),
            String::from("priv"),
            String::from("something"),
        )
        .build();
        let result = client.gen_api_url("/system/info");
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic]
    fn test_basic_get_panic() {
        let query = [];
        let _result = testing_client()
            .get_single("/this/is/a/bad/path", &query)
            .unwrap();
    }

    #[test]
    fn test_basic_get_single() {
        let query = [];

        let result = testing_client().get_single("/system/info", &query).unwrap();
        assert_eq!(&result["cloudRegion"], "NA");
        assert_eq!(&result["isCloud"], true);
        assert_eq!(&result["serverTimeZone"], "Eastern Standard Time");
    }

    #[test]
    fn test_basic_get() {
        let query = [];

        let result = testing_client().get("/system/members", &query).unwrap();

        assert!(result.len() > 40);

        let zach = &result[0];
        assert_eq!(&zach["adminFlag"], true);
        assert_eq!(&zach["dailyCapacity"], 8.0);
        assert_eq!(&zach["identifier"], "ZPeters");
    }

    #[test]
    fn test_new_client_default() {
        let input_company_id = "myco".to_string();
        let input_public_key = "public".to_string();
        let input_private_key = "private".to_string();
        let input_client_id = "clientid".to_string();

        let expected = Client {
            company_id: "myco".to_string(),
            public_key: "public".to_string(),
            private_key: "private".to_string(),
            client_id: "clientid".to_string(),
            api_version: "3.0".to_string(),
            api_url: "na.myconnectwise.net".to_string(),
            codebase: "v4_6_release".to_string(),
        };

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .build();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_new_client_api_version() {
        let input_company_id = "myco".to_string();
        let input_public_key = "public".to_string();
        let input_private_key = "private".to_string();
        let input_client_id = "clientid".to_string();
        let input_api_version = "version".to_string();

        let expected_api_version = "version";

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .api_version(input_api_version)
        .build();

        assert_eq!(result.api_version, expected_api_version);
    }

    #[test]
    fn test_new_client_codebase() {
        let input_company_id = "myco".to_string();
        let input_public_key = "public".to_string();
        let input_private_key = "private".to_string();
        let input_client_id = "clientid".to_string();
        let input_codebase = "codebase".to_string();

        let expected_codebase = "codebase";

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .codebase(input_codebase)
        .build();

        assert_eq!(result.codebase, expected_codebase);
    }

    #[test]
    fn test_new_client_chained_options() {
        let result = Client::new(
            "myco".to_string(),
            "public".to_string(),
            "private".to_string(),
            "clientid".to_string(),
        )
        .codebase("codebase".to_string())
        .api_url("api".to_string())
        .build();

        assert_eq!(result.api_url, "api".to_string());
        assert_eq!(result.codebase, "codebase".to_string());
    }
}
