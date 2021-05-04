#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]
//! crate for working with Connectwise Manage API
//!
//! In the connectwise api <https://developer.connectwise.com/Products/Manage> some results are
//! returned as a single 'object' and most are returned as a list.
//! Normally you will be getting a list of results (even a list of one) so you would use [Client.get].
//! In some cases, (/system/info for example) it does not return a list, in this case use [Client.get_single].
//! Consult the api documentation (above) for more details.
//!
//! # Get Example
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
//! # Post Example
//! ```
//! use cwmanage::Client;
//! use serde_json::json;
//! use dotenv::dotenv;
//! dotenv().ok();
//! let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
//! let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
//! let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
//! let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
//! let client = Client::new(company_id, public_key, private_key, client_id).build();
//! let body = json!({"foo": "bar"}).to_string();
//! let result = client.post("/system/members", body);
//! ```
//!
//! # Patch Example
//! ```
//! use cwmanage::{Client, PatchOp};
//! use serde_json::json;
//! use dotenv::dotenv;
//! dotenv().ok();
//! let company_id: String = dotenv::var("CWMANAGE_COMPANY_ID").unwrap();
//! let public_key: String = dotenv::var("CWMANAGE_PUBLIC_KEY").unwrap();
//! let private_key: String = dotenv::var("CWMANAGE_PRIVATE_KEY").unwrap();
//! let client_id: String = dotenv::var("CWMANAGE_CLIENT_ID").unwrap();
//! let client = Client::new(company_id, public_key, private_key, client_id).build();
//! let op = PatchOp::Replace;
//! let path = "name";
//! let value = json!("test_basic_patch_replace");
//! let result = client.patch("/sales/activities/100", op, path, value);
//! ```
//!
//! # Query examples
//! See the connectwise api for further details
//!
//! - No query - `[("", "")]`
//! - Only get the id field `[("fields", "id")]`
//! - Also apply some conditions `[("fields", "id"), ("conditions", "name LIKE '%foo%'")]`
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::string::ToString;
use strum_macros;
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

/// Our possible patch operations
#[derive(Debug, strum_macros::ToString)]
pub enum PatchOp {
    /// Add to a non-existing field
    #[strum(serialize = "add")]
    Add,
    /// Replace existing value with the provided one
    #[strum(serialize = "replace")]
    Replace,
    /// Remove the specified viewed
    #[strum(serialize = "remove")]
    Remove,
}

/// Connectwise client.  Initinitialize with [Client::new].  Use [Client::api_url],
/// [Client::api_version] and [Client::codebase] to customize.  The finalize with [Client::build]
/// * `company_id` is your _short name_ (ie the one you use to login to CW)
/// * `public_key` is obtained by creating an api member with keys
/// * `private_key` is obtained by creating an api member with keys
/// * the `client_id` is generated <https://developer.connectwise.com/ClientID>
#[derive(Debug, PartialEq, Clone)]
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
            .send()?
            .text()?;

        let v: Value = serde_json::from_str(&res)?;
        Ok(v)
    }

    /// This will get a custom field Value, it helps with some of the juggleing of all of the
    /// custom fields that get returned
    ///
    /// # Arguments
    ///
    /// - `path` - The 'path" is the exact url to the object (`/projects/project/123`, etc).
    /// - `field` - The field we want to update (also known as the "Caption")
    ///
    /// # Example
    /// ## getting a field
    /// ```
    /// use cwmanage::Client;
    /// use serde_json::json;
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
    /// let path = "/project/projects/1799";
    /// let field_name = "EPL";
    /// let expected = Some(json!(false));
    ///
    /// let result = client.get_custom_field(path, field_name);
    ///
    /// assert_eq!(result.unwrap(), expected);
    /// ```
    pub fn get_custom_field(&self, path: &str, field: &str) -> Result<Option<Value>> {
        let query = &[("fields", "customFields")];
        let res = &self.get_single(path, query)?;

        let custom_fields = res
            .get("customFields")
            .ok_or(anyhow!("cannot get customFields"))?
            .as_array()
            .ok_or(anyhow!("cannot parse as array"))?;

        let mut found_field: Option<Value> = None;
        for f in custom_fields.iter() {
            if &f["caption"].as_str().unwrap() == &field {
                found_field = Some(f["value"].clone());
            }
        }

        Ok(found_field)
    }

    fn get_custom_field_id(&self, path: &str, field: &str) -> Result<i64> {
        let query = &[("fields", "customFields")];
        let res = &self.get_single(path, query)?;

        let custom_fields = res
            .get("customFields")
            .ok_or(anyhow!("cannot get customFields"))?
            .as_array()
            .ok_or(anyhow!("cannot convert custom fires from to array"))?;

        let mut id: i64 = 0;
        for f in custom_fields.iter() {
            if &f["caption"]
                .as_str()
                .ok_or(anyhow!("cannot convert caption to string"))?
                == &field
            {
                id = f["id"]
                    .as_i64()
                    .ok_or(anyhow!("cannot convert id to i64"))?;
            }
        }

        match id {
            0 => Err(anyhow!("couldn't get id")),
            _any => Ok(id),
        }
    }

    /// This will Patch a custom field, this abstracts out some of the operations.
    ///
    /// # Arguments
    ///
    /// - `path` - The 'path" is the exact url to the object (`/projects/project/123`, etc).
    /// - `field` - The field we want to update (also known as the "Caption")
    /// - `value` - The value we want to update it to.  This is sent as a string and then
    ///             parsed to the appropriate datatype (ie it is sent as json). Example
    ///              "1234" for `1234`, "true" for `true`, etc
    ///
    /// # Example
    /// ## updating a field
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
    /// let path = "/project/projects/1799";
    /// let field_name = "EPL";
    /// let field_value = "false";
    /// let expected = ();
    ///
    /// let result = client.patch_custom_field(path, field_name, field_value);
    ///
    /// assert_eq!(result.unwrap(), expected);
    /// ```
    pub fn patch_custom_field(&self, path: &str, field: &str, value: &str) -> Result<()> {
        let field_id = &self.get_custom_field_id(path, field)?;
        let value = json!([{ "id": field_id, "value": value}]);
        match &self.patch(path, PatchOp::Replace, "customFields", value) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("could not patch field: {:?}", e)),
        }
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
                .send()?;

            let hdrs = res.headers();

            next = match hdrs.get("link") {
                Some(link) => {
                    if link.is_empty() {
                        false
                    } else {
                        match get_page_id(hdrs) {
                            Some(p) => {
                                page = p;
                                true
                            }
                            None => false,
                        }
                    }
                }
                None => false,
            };

            let body = res.text()?;
            let mut v: Vec<Value> = serde_json::from_str(&body)?;
            collected_res.append(&mut v);
        }

        Ok(collected_res)
    }

    /// POSTS a body to an api endpoint
    /// The expected return is the object was created
    /// If an error occurs (api level, not http level) it will return an error message
    ///
    /// # Arguments
    ///
    /// - `path` - the api path you want to retrieve (example `/service/info`)
    /// - `body` - the body of the post (see api docs for details). formated as json
    ///
    /// # Example
    /// see main docs
    ///
    pub fn post(&self, path: &str, body: String) -> Result<Value> {
        let res = reqwest::blocking::Client::new()
            .post(&self.gen_api_url(path))
            .header("Authorization", &self.gen_basic_auth())
            .header("Content-Type", "application/json")
            .header("clientid", self.client_id.to_owned())
            .header("pagination-type", "forward-only")
            .body(body)
            .send()?
            .text()?;

        let v: Value = serde_json::from_str(&res)?;

        match &v["errors"].as_array() {
            Some(_e) => Err(anyhow!("we got some errors: {:?}", &v["errors"].as_array())),
            None => {
                // Sometimes 'errors' is null but there is a message
                match &v["message"].as_str() {
                    Some(_e) => Err(anyhow!("we got some errors: {:?}", &v["message"].as_str())),
                    None => Ok(v),
                }
            }
        }
    }

    /// Patch (aka updated) to provided `patch_path` (field) on the object specified by path
    /// The expected return is the new version of the object that was modified
    /// If an error occurs (api level, not http level) it will return an error message
    ///
    /// # Arguments
    ///
    /// - `path` - the api path you want to retrieve (example `/service/info`)
    /// - `op` - one fo the allowed `PatchOp` values (Add | Replace | Remove)
    /// - `path_path` - field you want to modify (example `summmary`, `member/id`)
    /// - `value` - the value you want to update (example `New Name`)
    ///
    /// # Example
    /// see main docs
    pub fn patch(
        &self,
        path: &str,
        op: PatchOp,
        patch_path: &str,
        value: serde_json::Value,
    ) -> Result<Value> {
        // create the body - please note the [] square brackets
        let body = json!([{
            "op": op.to_string(),
            "path": patch_path,
            "value": value,
        }])
        .to_string();

        let res = reqwest::blocking::Client::new()
            .patch(&self.gen_api_url(path))
            .header("Authorization", &self.gen_basic_auth())
            .header("Content-Type", "application/json")
            .header("clientid", self.client_id.to_owned())
            .header("pagination-type", "forward-only")
            .body(body)
            .send()?
            .text()?;

        let v: Value = serde_json::from_str(&res)?;

        match &v["message"].as_str() {
            Some(_e) => Err(anyhow!("we got some errors: {:?}", &v)),
            None => Ok(v),
        }
    }
}

// *** Private Functions ***
fn get_page_id(hdrs: &reqwest::header::HeaderMap) -> Option<String> {
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

    let parsed_url = Url::parse(url).ok()?;
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

    match hash_query.contains_key("pageId") {
        false => None,
        true => Some(hash_query["pageId"].to_string()),
    }
}

// *** Tests ***
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use pretty_assertions::assert_eq;
    use serde_json::json;

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
    fn test_basic_post() {
        let body = json!({
            "name": "test from rust cwmanage",
            "assignTo": {
                "id": 149,
            }
        })
        .to_string();

        let result = testing_client().post("/sales/activities", body);
        assert!(!result.is_err());
    }

    #[test]
    fn test_project_post_error() {
        let body = json!({}).to_string();

        let result = testing_client().post("/project/projects/1/notes", body);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_post_error() {
        let body = json!({"name": "test from rust cwmanage"}).to_string();

        let result = testing_client().post("/sales/activities", body);
        assert!(result.is_err());
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

    #[test]
    /// This activity/name already exists so an add should fail
    fn test_basic_patch_add_should_fail() {
        let op = PatchOp::Add;
        let path = "name";
        let value = json!("test_basic_patch_add");

        let result = testing_client().patch("/sales/activities/99", op, path, value);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_patch_replace() {
        let op = PatchOp::Replace;
        let path = "name";
        let value = json!("test_basic_patch_replace");

        let result = testing_client().patch("/sales/activities/100", op, path, value);
        assert!(!result.is_err());
    }

    #[test]
    fn test_basic_patch_error() {
        let op = PatchOp::Add;
        let path = "summary";
        let value = json!("test_basic_patch_error_test");

        let result = testing_client().patch("/sales/activities/123", op, path, value);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_custom_field_bad_field_name() {
        let path = "/project/projects/4";
        let field_name = "A Fake Field";
        let expected = None;

        let result = testing_client().get_custom_field(path, field_name);

        assert_eq!(result.unwrap(), expected);
    }
    #[test]
    fn test_get_custom_field_something_set() {
        let path = "/project/projects/1799";
        let field_name = "E-rate";
        let expected = Some(json!(false));

        let result = testing_client().get_custom_field(path, field_name);

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_get_custom_field_id() {
        let path = "/project/projects/1799";
        let field_name = "WaitReason";
        let expected: i64 = 67;

        let result = testing_client().get_custom_field_id(path, field_name);

        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_get_custom_field_id_missing() {
        let path = "/project/projects/1799";
        let field_name = "A Fake Thing";

        let result = testing_client().get_custom_field_id(path, field_name);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_custom_field_something_else_set() {
        let path = "/project/projects/1799";
        let field_name = "WaitReason";
        let expected = Some(json!("Something Else"));

        let result = testing_client().get_custom_field(path, field_name);

        assert_eq!(result.unwrap(), expected);
    }
    #[test]
    fn test_update_custom_field_string() {
        let path = "/project/projects/1799";
        let field_name = "WaitReason";
        let field_value = "Something Else";
        let expected = ();

        let result = testing_client().patch_custom_field(path, field_name, field_value);
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_update_custom_field_bool() {
        let path = "/project/projects/1799";
        let field_name = "EPL";
        let field_value = "false";
        let expected = ();

        let result = testing_client().patch_custom_field(path, field_name, field_value);
        assert_eq!(result.unwrap(), expected);
    }
    #[test]
    fn test_update_custom_field_doesnt_exist() {
        let path = "/project/projects/1799";
        let field_name = "A Fake Field";
        let field_value = "false";

        let result = testing_client().patch_custom_field(path, field_name, field_value);
        assert!(result.is_err());
    }
}
