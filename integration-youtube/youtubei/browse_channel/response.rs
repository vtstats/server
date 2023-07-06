use serde::Deserialize;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub metadata: Metadata,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub channel_metadata_renderer: ChannelMetadataRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMetadataRenderer {
    pub avatar: Avatar,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar {
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub url: String,
}

#[test]
fn test() {
    assert_eq!(
        serde_json::from_str::<Response>(include_str!("./testdata/browse.0.json"))
            .unwrap()
            .metadata
            .channel_metadata_renderer
            .avatar
            .thumbnails[0]
            .url,
        "https://yt3.googleusercontent.com/ytc/AOPolaSFPK_6xlqthNXIpMC7OTWfGsDAoNkR9OexBYxcpA=s900-c-k-c0x00ffffff-no-rj",
    );

    assert_eq!(
        serde_json::from_str::<Response>(include_str!("./testdata/browse.1.json"))
            .unwrap()
            .metadata
            .channel_metadata_renderer
            .avatar
            .thumbnails[0]
            .url,
        "https://yt3.googleusercontent.com/ytc/AOPolaS5rJCXVzqSpK8mofxuXXYcGp8Cki-CW2KMCxgMsw=s900-c-k-c0x00ffffff-no-rj",
    );

    assert!(serde_json::from_str::<Response>(include_str!("./testdata/browse.2.json")).is_err());
}
