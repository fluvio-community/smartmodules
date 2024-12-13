## RSS XML to JSON Smartmodule

SmartModule that transforms RSS feed `XML` records into `json` records. This SmartModule is [map] type, where each record-in generates a record-out.

### Expected Input

RSS compatible XML (hackernews sample):

```xml
<rss version="2.0" xmlns:dc="http://purl.org/dc/elements/1.1/"
  xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>Hacker News: Newest</title>
    <link>https://news.ycombinator.com/newest</link>
    <description>Hacker News RSS</description>
    <docs>https://hnrss.org/</docs>
    <generator>hnrss v2.1</generator>
    <lastBuildDate>Tue, 02 May 2023 18:37:30 +0000</lastBuildDate>
    <atom:link href="https://hnrss.org/newest" rel="self" type="application/rss+xml"></atom:link>
    <item>
      <title><![CDATA[Smith's notes on Category Theory I updated]]></title>
      <description><![CDATA[
<p>Article URL: <a href="https://www.logicmatters.net/categories/">https://www.logicmatters.net/categories/</a></p>
<p>Comments URL: <a href="https://news.ycombinator.com/item?id=35792188">https://news.ycombinator.com/item?id=35792188</a></p>
<p>Points: 1</p>
<p># Comments: 1</p>
]]></description>
      <pubDate>Tue, 02 May 2023 18:35:27 +0000</pubDate>
      <link>https://www.logicmatters.net/categories/</link>
      <dc:creator>KurtGodelLives</dc:creator>
      <comments>https://news.ycombinator.com/item?id=35792188</comments>
      <guid isPermaLink="false">https://news.ycombinator.com/item?id=35792188</guid>
    </item>
  </channel>
</rss>
```

### Expected Ouptput

JSON equivalent:

```json
{
  "title": "Hacker News: Newest",
  "link": "https://news.ycombinator.com/newest",
  "description": "Hacker News RSS",
  "docs": "https://hnrss.org/",
  "generator": "hnrss v2.1",
  "last_build_date": "Tue, 02 May 2023 18:37:30 +0000",
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
  "namespaces": {
    "atom": "http://www.w3.org/2005/Atom",
    "dc": "http://purl.org/dc/elements/1.1/"
  },
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
}
```

### Compile and Test

Use `smdk` to build and test the smartmodule:

```
smdk build
```

Test the smartmodule:

```
smdk test --file ./test-data/input.xml --raw
```


### Acknowledgements

This project uses [rust-syndication/rss] to parse the XML.


[rust-syndication/rss]: https://github.com/rust-syndication/rss
[map]: https://www.fluvio.io/docs/smartmodules/features/operators/map