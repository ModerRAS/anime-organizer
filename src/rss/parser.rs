//! RSS 2.0 XML 解析模块
//!
//! 解析 RSS feed，提取 item 标题、magnet 链接、torrent URL 等信息。

use crate::error::{AppError, Result};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use regex::Regex;

/// RSS 项结构体
#[derive(Debug, Clone)]
pub struct RssItem {
    /// 标题
    pub title: String,
    /// Magnet 链接
    pub magnet: Option<String>,
    /// Torrent URL
    pub torrent_url: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 发布日期
    pub pub_date: Option<String>,
    /// 全局唯一标识
    pub guid: Option<String>,
}

/// 解析 RSS 2.0 XML 字符串
///
/// # 参数
/// - `xml`: RSS XML 字符串
///
/// # 返回
/// - `Result<Vec<RssItem>>`: 解析出的 RSS 项列表
pub fn parse_rss(xml: &str) -> Result<Vec<RssItem>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut buf = Vec::new();
    let mut in_item = false;
    let mut current_item: Option<RssItemBuilder> = None;
    let mut current_tag: Option<String> = None;

    let mut current_enclosure_url: Option<String> = None;
    let mut current_enclosure_type: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(AppError::MetadataFetchError(format!(
                    "RSS 解析错误 at position {}: {:?}",
                    reader.error_position(),
                    e
                )));
            }
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let name = e.name();
                if name == QName(b"item") {
                    in_item = true;
                    current_item = Some(RssItemBuilder::default());
                    current_enclosure_url = None;
                    current_enclosure_type = None;
                } else if name == QName(b"enclosure") && in_item {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key {
                            b"url" => current_enclosure_url = Some(value),
                            b"type" => current_enclosure_type = Some(value),
                            _ => {}
                        }
                    }
                } else if in_item {
                    if name == QName(b"title") {
                        current_tag = Some("title".to_string());
                    } else if name == QName(b"link") {
                        current_tag = Some("link".to_string());
                    } else if name == QName(b"description") {
                        current_tag = Some("description".to_string());
                    } else if name == QName(b"pubDate") {
                        current_tag = Some("pubDate".to_string());
                    } else if name == QName(b"guid") {
                        current_tag = Some("guid".to_string());
                    } else {
                        current_tag = None;
                    }
                }
            }

            Ok(Event::End(e)) => {
                let name = e.name();
                if name == QName(b"item") {
                    if let Some(mut builder) = current_item.take() {
                        let link_for_magnet = builder.link.clone();

                        if let Some(ref url) = current_enclosure_url {
                            if url.starts_with("magnet:?") {
                                builder.magnet = Some(url.clone());
                            } else if current_enclosure_type.as_deref()
                                == Some("application/x-bittorrent")
                                || url.ends_with(".torrent")
                            {
                                builder.torrent_url = Some(url.clone());
                            }
                        }

                        if builder.magnet.is_none() {
                            if let Some(ref desc) = builder.description {
                                if let Some(magnet) = extract_magnet(desc) {
                                    builder.magnet = Some(magnet);
                                }
                            }
                        }

                        if builder.magnet.is_none() {
                            if let Some(ref link) = link_for_magnet {
                                if link.starts_with("magnet:?") {
                                    builder.magnet = Some(link.clone());
                                }
                            }
                        }

                        let item = builder.into_rss_item();
                        items.push(item);
                    }
                    in_item = false;
                    current_enclosure_url = None;
                    current_enclosure_type = None;
                }
                current_tag = None;
            }

            Ok(Event::Text(e)) if in_item => {
                if let Some(ref mut builder) = current_item {
                    if let Some(ref tag) = current_tag {
                        let text = match e.unescape() {
                            Ok(t) => t.into_owned(),
                            Err(_) => continue,
                        };
                        builder.collect_text(tag, text);
                    }
                }
            }

            Ok(Event::CData(e)) if in_item => {
                if let Some(ref mut builder) = current_item {
                    if let Some(ref tag) = current_tag {
                        let text = String::from_utf8_lossy(e.as_ref()).trim().to_string();
                        builder.collect_text(tag, text);
                    }
                }
            }

            Ok(Event::Empty(e)) => {
                let name = e.name();
                if name == QName(b"enclosure") && in_item {
                    let mut url: Option<String> = None;
                    let mut enclosure_type: Option<String> = None;

                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        match key {
                            b"url" => url = Some(value),
                            b"type" => enclosure_type = Some(value),
                            _ => {}
                        }
                    }

                    if let Some(ref url_str) = url {
                        if url_str.starts_with("magnet:?") {
                            if let Some(ref mut builder) = current_item {
                                builder.magnet = Some(url_str.clone());
                            }
                        } else if enclosure_type.as_deref() == Some("application/x-bittorrent")
                            || url_str.ends_with(".torrent")
                        {
                            if let Some(ref mut builder) = current_item {
                                builder.torrent_url = Some(url_str.clone());
                            }
                        }
                    }
                }
            }

            _ => {}
        }
        buf.clear();
    }

    Ok(items)
}

/// 从文本中提取 magnet 链接
fn extract_magnet(text: &str) -> Option<String> {
    let magnet_regex = Regex::new(r"magnet:\?xt=urn:btih:[a-fA-F0-9]+").ok()?;
    magnet_regex.find(text).map(|m| m.as_str().to_string())
}

/// RSS 项构建器
#[derive(Debug, Default)]
struct RssItemBuilder {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    pub_date: Option<String>,
    guid: Option<String>,
    magnet: Option<String>,
    torrent_url: Option<String>,
}

impl RssItemBuilder {
    fn collect_text(&mut self, tag: &str, text: String) {
        match tag {
            "title" => {
                self.title = Some(text);
            }
            "link" => {
                self.link = Some(text);
            }
            "description" => {
                self.description = Some(text);
            }
            "pubDate" => {
                self.pub_date = Some(text);
            }
            "guid" => {
                self.guid = Some(text);
            }
            _ => {}
        }
    }

    fn into_rss_item(self) -> RssItem {
        RssItem {
            title: self.title.unwrap_or_default(),
            magnet: self.magnet,
            torrent_url: self.torrent_url,
            description: self.description,
            pub_date: self.pub_date,
            guid: self.guid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DMHY_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>动漫花园 RSS</title>
    <link>https://dmhy.org</link>
    <description>动漫花园 RSS 订阅</description>
    <item>
      <title>[ANi] 孤独摇滚 - 01 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4</title>
      <link>https://dmhy.org/topics/view/123456.html</link>
      <description>发布日期: 2024-01-15</description>
      <guid isPermaLink="false">dmhy-123456</guid>
      <pubDate>Mon, 15 Jan 2024 12:00:00 +0800</pubDate>
      <enclosure url="magnet:?xt=urn:btih:d41d8cd98f00b204e9800998ecf8427e" length="0" type="application/x-bittorrent"/>
    </item>
    <item>
      <title>[ANi] 迷宫饭 - 05 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4</title>
      <link>https://dmhy.org/topics/view/123457.html</link>
      <description>发布日期: 2024-01-16</description>
      <guid isPermaLink="false">dmhy-123457</guid>
      <pubDate>Tue, 16 Jan 2024 12:00:00 +0800</pubDate>
      <enclosure url="magnet:?xt=urn:btih:e71f2e5f00b204e9800998ecf8427e5" length="0" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

    const ANIRIP_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>AniRip RSS Feed</title>
    <link>https://ani.rip</link>
    <description>AniRip RSS Feed</description>
    <item>
      <title>[SubsPlease] Spy x Family S2 - 12 (1080p) [A8C1E3D4].mkv</title>
      <link>https://ani.rip/download/12345</link>
      <description>AniRip - Spy x Family Season 2 Episode 12</description>
      <guid isPermaLink="false">anirip-12345</guid>
      <pubDate>Wed, 17 Jan 2024 18:00:00 +0000</pubDate>
      <enclosure url="https://tds.ani.rip/torrents/spy_family_s2_12.torrent" type="application/x-bittorrent" length="1234567"/>
    </item>
    <item>
      <title>[SubsPlease] Frieren - 24 (1080p) [B2C3D4E5].mkv</title>
      <link>https://ani.rip/download/12346</link>
      <description>AniRip - Frieren Episode 24</description>
      <guid isPermaLink="false">anirip-12346</guid>
      <pubDate>Thu, 18 Jan 2024 18:00:00 +0000</pubDate>
      <enclosure url="https://tds.ani.rip/torrents/frieren_24.torrent" type="application/x-bittorrent" length="2345678"/>
    </item>
  </channel>
</rss>"#;

    const MAGNET_IN_DESC_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Magnet Test RSS</title>
    <link>https://example.com</link>
    <description>Test RSS with magnet in description</description>
    <item>
      <title>Test Anime Episode 01</title>
      <link>https://example.com/episode/01</link>
      <description>Download from https://example.com or magnet:?xt=urn:btih:f61f2e5f00b204e9800998ecf8427e5f for better quality</description>
      <guid isPermaLink="false">test-001</guid>
      <pubDate>Fri, 19 Jan 2024 12:00:00 +0000</pubDate>
    </item>
    <item>
      <title>Test Anime Episode 02</title>
      <link>magnet:?xt=urn:btih:a11f2e5f00b204e9800998ecf8427ea2</link>
      <description>Direct magnet link in link tag</description>
      <guid isPermaLink="false">test-002</guid>
      <pubDate>Sat, 20 Jan 2024 12:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

    #[test]
    fn test_parse_dmhy_rss_with_magnet_enclosure() {
        let items = parse_rss(DMHY_RSS).expect("Failed to parse DMHY RSS");

        assert_eq!(items.len(), 2);

        assert_eq!(
            items[0].title,
            "[ANi] 孤独摇滚 - 01 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4"
        );
        assert_eq!(
            items[0].magnet,
            Some("magnet:?xt=urn:btih:d41d8cd98f00b204e9800998ecf8427e".to_string())
        );
        assert!(items[0].torrent_url.is_none());
        assert_eq!(items[0].guid, Some("dmhy-123456".to_string()));
        assert_eq!(
            items[0].pub_date,
            Some("Mon, 15 Jan 2024 12:00:00 +0800".to_string())
        );

        assert_eq!(
            items[1].title,
            "[ANi] 迷宫饭 - 05 [1080P][Baha][WEB-DL][AAC AVC][CHT].mp4"
        );
        assert_eq!(
            items[1].magnet,
            Some("magnet:?xt=urn:btih:e71f2e5f00b204e9800998ecf8427e5".to_string())
        );
    }

    #[test]
    fn test_parse_anirip_rss_with_torrent_enclosure() {
        let items = parse_rss(ANIRIP_RSS).expect("Failed to parse AniRip RSS");

        assert_eq!(items.len(), 2);

        assert_eq!(
            items[0].title,
            "[SubsPlease] Spy x Family S2 - 12 (1080p) [A8C1E3D4].mkv"
        );
        assert!(items[0].magnet.is_none());
        assert_eq!(
            items[0].torrent_url,
            Some("https://tds.ani.rip/torrents/spy_family_s2_12.torrent".to_string())
        );
        assert_eq!(items[0].guid, Some("anirip-12345".to_string()));
        assert_eq!(
            items[0].pub_date,
            Some("Wed, 17 Jan 2024 18:00:00 +0000".to_string())
        );

        assert_eq!(
            items[1].title,
            "[SubsPlease] Frieren - 24 (1080p) [B2C3D4E5].mkv"
        );
        assert_eq!(
            items[1].torrent_url,
            Some("https://tds.ani.rip/torrents/frieren_24.torrent".to_string())
        );
    }

    #[test]
    fn test_parse_rss_with_magnet_in_description() {
        let items =
            parse_rss(MAGNET_IN_DESC_RSS).expect("Failed to parse RSS with magnet in description");

        assert_eq!(items.len(), 2);

        assert_eq!(items[0].title, "Test Anime Episode 01");
        assert_eq!(
            items[0].magnet,
            Some("magnet:?xt=urn:btih:f61f2e5f00b204e9800998ecf8427e5f".to_string())
        );
        assert!(items[0].torrent_url.is_none());
        assert_eq!(items[0].guid, Some("test-001".to_string()));

        assert_eq!(items[1].title, "Test Anime Episode 02");
        assert_eq!(
            items[1].magnet,
            Some("magnet:?xt=urn:btih:a11f2e5f00b204e9800998ecf8427ea2".to_string())
        );
        assert_eq!(items[1].guid, Some("test-002".to_string()));
    }

    #[test]
    fn test_parse_empty_rss() {
        let empty_rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Empty RSS</title>
    <link>https://example.com</link>
    <description>No items here</description>
  </channel>
</rss>"#;

        let items = parse_rss(empty_rss).expect("Failed to parse empty RSS");
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_rss_with_torrent_extension_url() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Torrent URL Test</title>
    <item>
      <title>Test Episode</title>
      <enclosure url="https://example.com/download.torrent" type="application/octet-stream"/>
    </item>
  </channel>
</rss>"#;

        let items = parse_rss(rss).expect("Failed to parse RSS");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].torrent_url,
            Some("https://example.com/download.torrent".to_string())
        );
    }

    #[test]
    fn test_parse_rss_with_cdata_title() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title><![CDATA[动漫花园 RSS]]></title>
    <item>
      <title><![CDATA[[jibaketa合成&音頻壓制] 测试动画 - 01 [1080P][WEB-DL][AAC AVC][CHT][MP4]]]></title>
      <link>https://dmhy.org/topics/view/123456.html</link>
      <guid isPermaLink="false">dmhy-cdata-001</guid>
      <pubDate>Wed, 08 Apr 2026 12:00:00 +0800</pubDate>
      <enclosure url="magnet:?xt=urn:btih:ABCDEF1234567890" type="application/x-bittorrent"/>
    </item>
  </channel>
</rss>"#;

        let items = parse_rss(rss).expect("Failed to parse CDATA RSS");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].title,
            "[jibaketa合成&音頻壓制] 测试动画 - 01 [1080P][WEB-DL][AAC AVC][CHT][MP4]"
        );
        assert_eq!(
            items[0].magnet,
            Some("magnet:?xt=urn:btih:ABCDEF1234567890".to_string())
        );
    }
}
