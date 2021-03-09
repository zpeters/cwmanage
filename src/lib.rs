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
//! let client = Client::new("company_id", "public_key", "private_key", "client_id").build();
//! let query = [("", "")]
//! let result = client.get_single("/system/info", query).unwrap();
//! ```
//! Override the api_version
//! ```
//! let client = Client::new("company_id", "public_key", "private_key", "client_id").api_version("2020_4").build();
//! let query = [("", "")]
//! let result = client.get_single("/system/info", query).unwrap();
//! ```
//!
//! Get an endpoint with multiple results
//! let client = Client::new("company_id", "public_key", "private_key", "client_id").build();
//! let query = [("pagesize", "100")]
//! let result = client.get("/service/tickets", queryh).unwrap();
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

// *** Structs ***
/// authentication credentials for the connectwise api
/// * `company_id` is your _short name_ (ie the one you use to login to CW)
/// * `public_key` is obtained by creating an api member with keys
/// * `private_key` is obtained by creating an api member with keys
/// * the `client_id` is generated <https://developer.connectwise.com/ClientID>
#[derive(Debug, PartialEq)]
pub struct Client<'a> {
    company_id: &'a str,
    public_key: &'a str,
    private_key: &'a str,
    client_id: &'a str,
    api_url: &'a str,
    codebase: &'a str,
    api_version: &'a str,
}
impl<'a> Client<'a> {
    pub fn new(
        company_id: &'a str,
        public_key: &'a str,
        private_key: &'a str,
        client_id: &'a str,
    ) -> Client<'a> {
        Client {
            company_id,
            public_key,
            private_key,
            client_id,
            // TODO convert to const and document
            api_url: "na.myconnectwise.net",
            // TODO convert to const and document
            codebase: "v4_6_release",
            // TODO convert to const and document
            api_version: "3.0",
        }
    }
    pub fn build(&self) -> Client<'a> {
        Client {
            company_id: self.company_id,
            public_key: self.public_key,
            private_key: self.private_key,
            client_id: self.client_id,
            api_url: self.api_url,
            codebase: self.codebase,
            api_version: self.api_version,
        }
    }

    pub fn api_version(mut self, api_version: &'a str) -> Client {
        self.api_version = api_version;
        self
    }

    pub fn api_url(mut self, api_url: &'a str) -> Client {
        self.api_url = api_url;
        self
    }

    pub fn codebase(mut self, codebase: &'a str) -> Client {
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
    /// let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();
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
    /// let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();
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
            .header("clientid", self.client_id)
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
    /// let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();
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
    /// let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();
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
                .header("clientid", self.client_id)
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

    // TODO create a function to generate a test client

    #[test]
    fn test_basic_auth() {
        let expected: String = "Basic bXljbytwdWI6cHJpdg==".to_string();
        let client = Client::new("myco", "pub", "priv", "something").build();
        let result = client.gen_basic_auth();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_gen_url() {
        let expected = "https://na.myconnectwise.net/v4_6_release/apis/3.0/system/info";
        let client = Client::new("myco", "pub", "priv", "something").build();
        let result = client.gen_api_url("/system/info");
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic]
    fn test_basic_get_panic() {
        dotenv().ok();
        let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
        let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
        let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
        let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();

        let query = [];

        let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();

        let _result = client.get_single("/this/is/a/bad/path", &query).unwrap();
    }

    #[test]
    fn test_basic_get_single() {
        dotenv().ok();
        let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
        let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
        let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
        let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();

        let query = [];

        let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();

        let result = client.get_single("/system/info", &query).unwrap();
        assert_eq!(&result["cloudRegion"], "NA");
        assert_eq!(&result["isCloud"], true);
        assert_eq!(&result["serverTimeZone"], "Eastern Standard Time");
    }

    #[test]
    fn test_basic_get() {
        dotenv().ok();
        let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
        let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
        let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
        let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();

        let query = [];

        let client = Client::new(&company_id, &public_key, &private_key, &client_id).build();

        let result = client.get("/system/members", &query).unwrap();

        assert!(result.len() > 40);

        let zach = &result[0];
        assert_eq!(&zach["adminFlag"], true);
        assert_eq!(&zach["dailyCapacity"], 8.0);
        assert_eq!(&zach["identifier"], "ZPeters");
    }

    #[test]
    fn test_new_client_default() {
        let input_company_id = "myco";
        let input_public_key = "public";
        let input_private_key = "private";
        let input_client_id = "clientid";

        let expected = Client {
            company_id: "myco",
            public_key: "public",
            private_key: "private",
            client_id: "clientid",
            api_version: "3.0",
            api_url: "na.myconnectwise.net",
            codebase: "v4_6_release",
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
        let input_company_id = "myco";
        let input_public_key = "public";
        let input_private_key = "private";
        let input_client_id = "clientid";
        let input_api_version = "api";

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .api_version(&input_api_version)
        .build();

        assert_eq!(result.api_version, input_api_version);
    }

    #[test]
    fn test_new_client_api_url() {
        let input_company_id = "myco";
        let input_public_key = "public";
        let input_private_key = "private";
        let input_client_id = "clientid";
        let input_api_url = "mybase";

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .api_url(&input_api_url)
        .build();

        assert_eq!(result.api_url, input_api_url);
    }

    #[test]
    fn test_new_client_codebase() {
        let input_company_id = "myco";
        let input_public_key = "public";
        let input_private_key = "private";
        let input_client_id = "clientid";
        let input_codebase = "codebase";

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .codebase(&input_codebase)
        .build();

        assert_eq!(result.codebase, input_codebase);
    }

    #[test]
    fn test_new_client_chained_options() {
        let input_company_id = "myco";
        let input_public_key = "public";
        let input_private_key = "private";
        let input_client_id = "clientid";
        let input_api_url = "api";
        let input_codebase = "codebase";

        let result = Client::new(
            input_company_id,
            input_public_key,
            input_private_key,
            input_client_id,
        )
        .codebase(&input_codebase)
        .api_url(&input_api_url)
        .build();

        assert_eq!(result.api_url, input_api_url);
        assert_eq!(result.codebase, input_codebase);
    }
}
