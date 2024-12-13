
use serde_json::json;
use serde_json::value::Value;
use rss::Channel;

use fluvio_smartmodule::dataplane::smartmodule::SmartModuleExtraParams;
use fluvio_smartmodule::{
    smartmodule, Result, SmartModuleRecord, RecordData
};

/// Recursively remove keys with following values:
///  - null, {}, []
fn remove_empty_keys(value: &mut Value) {
    match value {
        Value::Object(ref mut map) => {
            let mut to_remove = Vec::new();

            for (key, value) in map.into_iter() {
                if value.is_null() ||
                    value.is_object() &&value.as_object().unwrap().len() == 0 ||
                    value.is_array() && value.as_array().unwrap().len() == 0 {
                    to_remove.push(key.clone());
                } else {
                    remove_empty_keys(value);
                }
            }

            for key in to_remove {
                map.remove(&key);
            }
        }
        Value::Array(ref mut array) => {
            for value in array.into_iter() {
                remove_empty_keys(value);
            }
        }
        _ => {}
    }
}

#[smartmodule(map)]
pub fn map(record: &SmartModuleRecord) -> Result<(Option<RecordData>, RecordData)> {
    let key = record.key.clone();

    let channel = Channel::read_from(record.value.as_ref()).unwrap();
    let mut json = json!(channel);
    remove_empty_keys(&mut json);

    let serialized_output = serde_json::to_vec(&json)?;

    Ok((key, RecordData::from(serialized_output)))
}


#[smartmodule(init)]
fn init(_params: SmartModuleExtraParams) -> Result<()> {
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_empty_keys_test() {
        let input = r#"{
            "categories": [],
            "cloud": null,
            "copyright": null,
            "description": "Hacker News RSS",
            "docs": "https://hnrss.org/",
            "dublin_core_ext": null,
            "extensions": {
              "atom": {
                "link": [
                  {
                    "attrs": {
                      "href": "https://hnrss.org/newest",
                      "rel": "self",
                      "type": "application/rss+xml"
                    },
                    "children": {},
                    "name": "atom:link",
                    "value": null
                  }
                ]
              }
            },
            "generator": "hnrss v2.1",
            "image": null,
            "items": [
              {
                "author": null,
                "categories": [],
                "comments": "https://news.ycombinator.com/item?id=35792188",
                "content": null,
                "description": "\n<p>Article URL: <a href=\"https://www.logicmatters.net/categories/\">https://www.logicmatters.net/categories/</a></p>\n<p>Comments URL: <a href=\"https://news.ycombinator.com/item?id=35792188\">https://news.ycombinator.com/item?id=35792188</a></p>\n<p>Points: 1</p>\n<p># Comments: 1</p>\n",
                "dublin_core_ext": {
                  "contributors": [],
                  "coverages": [],
                  "creators": [
                    "KurtGodelLives"
                  ],
                  "dates": [],
                  "descriptions": [],
                  "formats": [],
                  "identifiers": [],
                  "languages": [],
                  "publishers": [],
                  "relations": [],
                  "rights": [],
                  "sources": [],
                  "subjects": [],
                  "titles": [],
                  "types": []
                },
                "enclosure": null,
                "extensions": {},
                "guid": {
                  "permalink": false,
                  "value": "https://news.ycombinator.com/item?id=35792188"
                },
                "itunes_ext": null,
                "link": "https://www.logicmatters.net/categories/",
                "pub_date": "Tue, 02 May 2023 18:35:27 +0000",
                "source": null,
                "title": "Smith's notes on Category Theory I updated"
              }
            ]
        }"#;
        let expected_output = r#"{
            "description": "Hacker News RSS",
            "docs": "https://hnrss.org/",
            "extensions": {
              "atom": {
                "link": [
                  {
                    "attrs": {
                      "href": "https://hnrss.org/newest",
                      "rel": "self",
                      "type": "application/rss+xml"
                    },
                    "name": "atom:link"
                  }
                ]
              }
            },
            "generator": "hnrss v2.1",
            "items": [
              {
                "comments": "https://news.ycombinator.com/item?id=35792188",
                "description": "\n<p>Article URL: <a href=\"https://www.logicmatters.net/categories/\">https://www.logicmatters.net/categories/</a></p>\n<p>Comments URL: <a href=\"https://news.ycombinator.com/item?id=35792188\">https://news.ycombinator.com/item?id=35792188</a></p>\n<p>Points: 1</p>\n<p># Comments: 1</p>\n",
                "dublin_core_ext": {
                  "creators": [
                    "KurtGodelLives"
                  ]                
                },
                "guid": {
                  "permalink": false,
                  "value": "https://news.ycombinator.com/item?id=35792188"
                },
                "link": "https://www.logicmatters.net/categories/",
                "pub_date": "Tue, 02 May 2023 18:35:27 +0000",
                "title": "Smith's notes on Category Theory I updated"
              }
            ]
        }"#;

        let mut json_in: Value = serde_json::from_str(input).unwrap();
        let json_out: Value = serde_json::from_str(expected_output).unwrap();

        remove_empty_keys(&mut json_in);

        assert_eq!(json_in, json_out);

    }
}