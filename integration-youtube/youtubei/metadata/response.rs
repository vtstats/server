use serde::de::IgnoredAny;
use serde::Deserialize;
use serde_with::serde_as;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response<'a> {
    #[serde(default)]
    pub actions: Vec<Action<'a>>,
    #[serde(default)]
    pub continuation: Option<Continuation<'a>>,
}

impl<'a> Response<'a> {
    pub fn timeout(&self) -> Option<Duration> {
        self.continuation
            .as_ref()
            .map(|c| Duration::from_millis(c.timed_continuation_data.timeout_ms))
    }

    pub fn is_waiting(&self) -> bool {
        self.actions.iter().any(|action| {
            action
                .update_viewership_action
                .as_ref()
                .and_then(|action| action.view_count.as_ref())
                .and_then(|view_count| view_count.video_view_count_renderer.view_count.as_ref())
                .map(|view_count| view_count.simple_text.contains("waiting"))
                .unwrap_or(false)
        })
    }

    pub fn view_count(&self) -> Option<i32> {
        self.actions.iter().find_map(|action| {
            action
                .update_viewership_action
                .as_ref()
                .and_then(|action| action.view_count.as_ref())
                .and_then(|view_count| view_count.video_view_count_renderer.view_count.as_ref())
                .and_then(|view_count| view_count.simple_text.strip_suffix("watching now"))
                .and_then(|v| v.replace(',', "").trim().parse().ok())
        })
    }

    pub fn like_count(&self) -> Option<i32> {
        self.actions
            .iter()
            .find_map(|action| match &action.update_toggle_button_text_action {
                Some(action) if action.button_id == "TOGGLE_BUTTON_ID_TYPE_LIKE" => {
                    let text = &action.default_text.simple_text;

                    text.strip_suffix('K')
                        .and_then(|c| c.parse::<f32>().ok())
                        .map(|c| (c * 1_000f32) as i32)
                        .or_else(|| {
                            text.strip_suffix('M')
                                .and_then(|c| c.parse::<f32>().ok())
                                .map(|c| (c * 1_000_000f32) as i32)
                        })
                        .or_else(|| text.parse().ok())
                }
                _ => None,
            })
    }

    pub fn title(&self) -> Option<String> {
        self.actions.iter().find_map(|action| {
            action.update_title_action.as_ref().map(|action| {
                action
                    .title
                    .runs
                    .iter()
                    .fold(String::new(), |acc, run| acc + &run.text)
            })
        })
    }

    pub fn continuation(&self) -> Option<&str> {
        self.continuation
            .as_ref()
            .map(|c| c.timed_continuation_data.continuation.as_ref())
    }

    pub fn unknown(&self) -> Vec<String> {
        self.actions
            .iter()
            .filter_map(|action| action.unknown.keys().next())
            .cloned()
            .collect()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Continuation<'a> {
    pub timed_continuation_data: TimedContinuationData<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct TimedContinuationData<'a> {
    pub timeout_ms: u64,
    #[serde_as(as = "BorrowCow")]
    pub continuation: Cow<'a, str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Action<'a> {
    #[serde(default)]
    pub update_viewership_action: Option<UpdateViewershipAction<'a>>,
    #[serde(default)]
    pub update_toggle_button_text_action: Option<UpdateToggleButtonTextAction<'a>>,
    #[serde(default)]
    pub update_date_text_action: IgnoredAny,
    #[serde(default)]
    pub update_title_action: Option<UpdateTitleAction<'a>>,
    #[serde(default)]
    pub update_description_action: IgnoredAny,
    #[serde(flatten)]
    pub unknown: HashMap<String, IgnoredAny>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateViewershipAction<'a> {
    #[serde(default)]
    pub view_count: Option<UpdateViewershipActionViewCount<'a>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateViewershipActionViewCount<'a> {
    pub video_view_count_renderer: VideoViewCountRenderer<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoViewCountRenderer<'a> {
    #[serde(default)]
    pub view_count: Option<VideoViewCountRendererViewCount<'a>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct VideoViewCountRendererViewCount<'a> {
    #[serde_as(as = "BorrowCow")]
    pub simple_text: Cow<'a, str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTitleAction<'a> {
    pub title: UpdateTitleActionTitle<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTitleActionTitle<'a> {
    pub runs: Vec<UpdateTitleActionTitleRun<'a>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct UpdateTitleActionTitleRun<'a> {
    #[serde_as(as = "BorrowCow")]
    pub text: Cow<'a, str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct UpdateToggleButtonTextAction<'a> {
    #[serde_as(as = "BorrowCow")]
    pub button_id: Cow<'a, str>,
    pub default_text: UpdateToggleButtonTextActionDefaultText<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde_as]
pub struct UpdateToggleButtonTextActionDefaultText<'a> {
    #[serde_as(as = "BorrowCow")]
    pub simple_text: Cow<'a, str>,
}

#[test]
fn de() {
    {
        let response =
            serde_json::from_str::<Response>(include_str!("./testdata/nothing.json")).unwrap();
        assert!(!response.is_waiting());
        assert_eq!(response.like_count(), None);
        assert_eq!(response.continuation(), None);
        assert_eq!(response.view_count(), None);
        assert_eq!(response.title(), None);
        assert_eq!(response.timeout(), None);
    }

    {
        let response =
            serde_json::from_str::<Response>(include_str!("./testdata/timed1.json")).unwrap();
        assert!(!response.is_waiting());
        assert_eq!(
            response.continuation(),
            Some("-of5rQMXGgtzUmVqME9KdFZMNCCq0s6YBjgBQAE%3D")
        );
        assert_eq!(response.view_count(), Some(3647));
        assert_eq!(response.like_count(), Some(7100));
        assert_eq!(response.title(), None);
        assert_eq!(response.timeout(), Some(Duration::from_secs(5)));
    }

    {
        let response =
            serde_json::from_str::<Response>(include_str!("./testdata/timed2.json")).unwrap();
        assert!(!response.is_waiting());
        assert_eq!(
            response.continuation(),
            Some("-of5rQMXGgtRWi1DZzR1cFI5ayCM89CYBjgBQAE%3D")
        );
        assert_eq!(response.view_count(), Some(2465));
        assert_eq!(response.like_count(), Some(3300));
        assert_eq!(
            response.title(),
            Some("【WATCHALONG】 The Lord of the Rings: film 3 extended edition 【NIJISANJI EN | Elira Pendora】".into())
        );
        assert_eq!(response.timeout(), Some(Duration::from_secs(5)));
    }

    {
        let response =
            serde_json::from_str::<Response>(include_str!("./testdata/waiting1.json")).unwrap();
        assert!(response.is_waiting());
        assert_eq!(
            response.continuation(),
            Some("-of5rQMXGgs2RU5pVEcwNTBBOCC5hNGYBjgBQAA%3D")
        );
        assert_eq!(response.view_count(), None);
        assert_eq!(response.like_count(), Some(5));
        assert_eq!(
            response.title(),
            Some("【星のカービィ ディスカバリー】バカのプレイ！3rd play!!!".into())
        );
        assert_eq!(response.timeout(), Some(Duration::from_secs(60)));
    }

    {
        let response =
            serde_json::from_str::<Response>(include_str!("./testdata/waiting2.json")).unwrap();
        assert!(response.is_waiting());
        assert_eq!(
            response.continuation(),
            Some("-of5rQMXGgtMejhFWm5vbGhkOCD2tdGYBjgBQAA%3D")
        );
        assert_eq!(response.view_count(), None);
        assert_eq!(response.like_count(), Some(491));
        assert_eq!(
            response.title(),
            Some(
                "【 Minecraft 】ホグワーツ編！まもなく門完成だ～～～！✨【ホロライブ/はあちゃま】"
                    .into()
            )
        );
        assert_eq!(response.timeout(), Some(Duration::from_secs(60)));
    }
}
