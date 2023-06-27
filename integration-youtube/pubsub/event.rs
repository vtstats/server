use roxmltree::Document;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Event {
    Modification {
        platform_channel_id: String,
        platform_stream_id: String,
    },

    Deletion {
        platform_channel_id: String,
        platform_stream_id: String,
    },
}

impl FromStr for Event {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let doc = match Document::parse(&s) {
            Ok(doc) => doc,
            Err(err) => anyhow::bail!("XML Syntax Error: {}", err),
        };

        parse_modification(&doc)
            .or_else(|| parse_deletion(&doc))
            .ok_or_else(|| anyhow::anyhow!("Unknown xml schema"))
    }
}

fn parse_modification(doc: &Document) -> Option<Event> {
    let stream_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "videoId")
        .and_then(|n| n.text())?;

    let channel_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "channelId")
        .and_then(|n| n.text())?;

    Some(Event::Modification {
        platform_channel_id: channel_id.into(),
        platform_stream_id: stream_id.into(),
    })
}

fn parse_deletion(doc: &Document) -> Option<Event> {
    let stream_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "deleted-entry")
        .and_then(|n| n.attribute("ref"))
        .and_then(|r| r.get("yt:video:".len()..))?;

    let channel_id = doc
        .descendants()
        .find(|n| n.tag_name().name() == "uri")
        .and_then(|n| n.text())
        .and_then(|n| n.get("https://www.youtube.com/channel/".len()..))?;

    Some(Event::Deletion {
        platform_channel_id: channel_id.into(),
        platform_stream_id: stream_id.into(),
    })
}

#[test]
fn from_str() {
    assert_eq!(
        Event::from_str(include_str!("./testdata/deletion.0.xml")).unwrap(),
        Event::Deletion {
            platform_channel_id: "UCdyqAaZDKHXg4Ahi7VENThQ".into(),
            platform_stream_id: "HJiD8KcZKfs".into()
        }
    );
    assert_eq!(
        Event::from_str(include_str!("./testdata/deletion.1.xml")).unwrap(),
        Event::Deletion {
            platform_channel_id: "UCdyqAaZDKHXg4Ahi7VENThQ".into(),
            platform_stream_id: "HJiD8KcZKfs".into()
        }
    );
    assert_eq!(
        Event::from_str(include_str!("./testdata/modification.0.xml")).unwrap(),
        Event::Modification {
            platform_channel_id: "UC7fk0CB07ly8oSl0aqKkqFg".into(),
            platform_stream_id: "hAo6NGQlkOA".into()
        }
    );
    assert_eq!(
        Event::from_str(include_str!("./testdata/modification.1.xml")).unwrap(),
        Event::Modification {
            platform_channel_id: "UC7fk0CB07ly8oSl0aqKkqFg".into(),
            platform_stream_id: "hAo6NGQlkOA".into()
        }
    );
}
