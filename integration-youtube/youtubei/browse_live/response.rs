use serde::{Deserialize, Serialize};

impl Response {
    fn items(&self) -> Option<&Vec<Content2>> {
        if let Some(contents) = &self.contents {
            let content = contents
                .two_column_browse_results_renderer
                .tabs
                .iter()
                .filter_map(|tab| tab.tab_renderer.as_ref())
                .find(|r| r.title == "Live")?
                .content
                .as_ref()?;
            Some(&content.rich_grid_renderer.contents)
        } else if let Some(action) = self.on_response_received_actions.first() {
            Some(&action.append_continuation_items_action.continuation_items)
        } else {
            None
        }
    }

    pub fn ended_streams(&self) -> Vec<&str> {
        let Some(items) = self.items() else {
            return vec![];
        };

        items
            .iter()
            .filter_map(|content| {
                let item = content.rich_item_renderer.as_ref()?;
                item.content.video_renderer.published_time_text.as_ref()?;
                Some(item.content.video_renderer.video_id.as_str())
            })
            .collect()
    }

    pub fn next_continuation(&self) -> Option<&str> {
        Some(
            self.items()?
                .iter()
                .find_map(|content| content.continuation_item_renderer.as_ref())?
                .continuation_endpoint
                .continuation_command
                .token
                .as_str(),
        )
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(default)]
    pub contents: Option<Contents>,
    #[serde(default)]
    pub on_response_received_actions: Vec<OnResponseReceivedAction>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnResponseReceivedAction {
    pub append_continuation_items_action: AppendContinuationItemsAction,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendContinuationItemsAction {
    pub continuation_items: Vec<Content2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contents {
    pub two_column_browse_results_renderer: TwoColumnBrowseResultsRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoColumnBrowseResultsRenderer {
    pub tabs: Vec<Tab>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub tab_renderer: Option<TabRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabRenderer {
    pub title: String,
    pub content: Option<Content>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub rich_grid_renderer: RichGridRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichGridRenderer {
    pub contents: Vec<Content2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content2 {
    pub rich_item_renderer: Option<RichItemRenderer>,
    pub continuation_item_renderer: Option<ContinuationItemRenderer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichItemRenderer {
    pub content: Content3,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content3 {
    pub video_renderer: VideoRenderer,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoRenderer {
    pub video_id: String,
    pub title: Title,
    pub upcoming_event_data: Option<UpcomingEventData>,
    pub published_time_text: Option<PublishedTimeText>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub runs: Vec<Run>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingEventData {
    pub start_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedTimeText {
    pub simple_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationItemRenderer {
    pub continuation_endpoint: ContinuationEndpoint,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationEndpoint {
    pub continuation_command: ContinuationCommand,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationCommand {
    pub token: String,
}

#[test]
fn parse() {
    let res = serde_json::from_str::<Response>(include_str!("./testdata/live.0.json")).unwrap();
    assert_eq!(res.ended_streams(), vec!["f-lActNjX2I"]);
    assert_eq!(res.next_continuation(), Some("4qmFsgKpCRIYVUNrSWltV1o5Z0JKUmFtS0Ywcm1QVTh3GowJOGdibEJocmlCbkxmQmdyYUJncXhCZ0VwWDN1MHJQSjIzSWh5VWZHakk1VTNadEhfUjJoZE5hS2dBb0dROWZiZ2JVdW83QmVBU0JGYmFUd1J2bzZvcUtSdU5sNkVCZTNGQ2d3VnZBMXpwZmN4VXU3SlQ0cHhmcWtFalJtOEl1V1A3TWQwX0xiU1AxcWRYY2N6dGZ0U1oxdGU0V1hwQWRzNUZ2UWlKaW84eURVa2tPbXdUbUlTck1sdDNDVGtoUVIzODE1bUh0SVU4NGVtUDFtWWFzcDJWMml2NDJTY0o1T01RY2NDeThDLUpmaUM3d2ROZkFaOW93bWQ1ODN4R0JobS1FQV9pR3I4RGtBa3U0LWRNa0tDc3J2dlBKNHBGVFVJbkM1ZzdtZlNpS3NsWUNOZng3UkJEbV9lMXpzT3l3MWhxbzhXQ0E1RjRGSUFhNkMtOFNGVEtwVUpxejhJTVJCQUd5d045TndWclNkTGlyMjhLRlJ6X3V1OFdVMF96VVN6VVNPVFZibnNWclJaajNabWFHUlhpRGpKRU0xbDZIU19oMF96LXRKMVNscDd1SmRNUHhwNVRVc1BBaGt6cHJUMHNwcG1RbS05VE1Gd0cycGpfbzlVd0JqdmtweWVTUk10UGRqWTdDQzVqYlE0Y0RRSmJzUHJhVkFvUGVyVVBtYkFFYzRPU3EzQUJLbTVEa2dPSjFlcUdRZFVsYkJTRzlxeVNpN2dMNEFJRWZMV3VwUjZEZlQ0MWJpU3paNlE1a2VLenB2dzk5ckRQWXNjUnRjcE80Q0lmbnpSUF81dFpUYl8tVFFScm5QNk5zTU51UHRfbFdrOTlzRVo4dDdDV0FTOE5oOXBVOGNVcGR5M29ONFVWS3hpOTdtSXZ6STFLaW92d1JtZzRFcmlZYmtUTy1PdTE0bDVtNXIyZVVlT2x0cEw0bXp2VmpNUG1Bb3NidEZOOTY0X1J0MURhVFY4R2hjV2M1aGJsbXQ0SEVwY3ZRLXdfenh3TEgzUWloWjBUV2dNRnlyQkYxdmJIaGZGS2lJamU4S3piMlU2c2w2M082TjJ6cmdkbURlOGx6VWtGZDVnUWxNYjVDLThuOWhKa0gzVEY1OFdZRlFscFBMdWQzZHB3NGFZWHByaTVncHljMVhfTGZocmRrYjhWYUIySFBrRHRwVGF3a0tlM0dIcUxqYkNkd3JCQV90WXRGVFE0UkZEd05zbGUyWF8yRVF3bDYzLUxvNjFTNU5mZ18tSDNnWWJnVWZrVDlIc0hIWkZCOWs2TmpsZnZvc2dzbk95TGVYWUg1YTJkUVlJc0lGNl9KYkF2UTN2cXk5bnY3SE5PbnBsTldwdHVTX1JoeTRNRVJoeWhpSGxtOEVWaHZUZGktUk0wM2JKQi1QVldTTzdSa0U5WEhlQUllcUc1c2lKSldZUUR1LU4xdU5SU2xQWUpJbnl5eDJoaWtxaVgwdHdxbjVXVEI1Sm9OY1NKRFkxTW1ZeVl6VmlMVEF3TURBdE1qQm1NQzA1TkRreUxUVTRNalF5T1dNME5qSTRZeGdC"));

    let res = serde_json::from_str::<Response>(include_str!("./testdata/append.json")).unwrap();
    assert_eq!(res.ended_streams(), vec!["QJl-AGI86Ro", "LCgR-hP0TOU"]);
    assert_eq!(res.next_continuation(), Some("4qmFsgKpCRIYVUNrSWltV1o5Z0JKUmFtS0Ywcm1QVTh3GowJOGdibEJocmlCbkxmQmdyYUJncXhCZ0VwWDN1MFllTGV0NjFaX010Uk40SVNyM011WWg4Q0NhSTBhWFQ2VWJ4b0FuTmVsZ2FJalBlVDBqTjhLV2Jkd3FwNXh4QUtUdUNqa0t4T1pTYndXNG9ac3VHSnV1eDJCTWNaR0pEMHc1dHVPMXJ6bHVkYlhickJ4WG16cEdfcTVTeFZCRFJMa09INnJBYll5ZlJTVVFISTdMU2dwVndhaHZ4OWphZ3lSRGZCSEZxS1B6QkFoWGszY0RDeXU3SGdWODJXQ0s2NnpTZUlITmszdTNLUDBmaTZPQ0R3UUJaYml0Zkkxd25lYUt0OUdoNXhnYmhXYVFUR0xuaTZ6NTBhVWFGc1RGNkxfQkNhOGVXVHgyZ011Yml3V00tZDdzVlBDUk1lRk9EZWRrLWJaeGFKOEx3YU15eUVkYW5ZaFhTbmhxZG1IU29pcVB6bk1maTBqbUhzVTNfWERKTUZaMjNNQ3BaWS1NUDBsc25zcHYwbm9CMWNRM1ZoVU1wR0NRaTRwZ0UwUU9pWUlJbGN3UFVXeTdmeHdSNEc1eWlzWXZRNXJVdXVDU3A1d1ZXd0FUUHc5WHNuMGtsSUQ0QWVlUTNJTWdfanFra2ZzU21BSmtxaEkxakZ2a0xTb0J4UndETkEwZTFIWmQwNmE2ODVibTlqTkxqV1FYcUI5NmEyX0d3NDl3NTc5WjFOd2pzb3k4aWFEU1hSUWhmYTR5bUl3WXAtWTE4Y05KY2JLaDc4YXlOa0N3b0hTSUFSMnJLWGlyVm92cV92MFJtUThyNGFlSVBtdmFGSTc0cFZmdTJLYmkzNTJpMG5BQklON2dCQVB5NjVCR181VjhwUEE4N0FHS0lPbG9PSE8xb0FFZC1QXzFkN21UMW1LR3BuWUpTbDVFQjVIX1NDRk5Wd2xhRVI3ZVJKUlhTNDA2bm8yYU1INEh2ZHlnRmltNm16WFl0MHVhRHM5Z2tTbDJjdXNYSDVVRTEwV3ptaE5SM0F3aVI2eEtraWhCdFk4NEl1c3pFZTNoNWdsVmJXUmx6MHJ2b0FYdm1fNUhBazZhMHFsTE1PRUhqN1NHakQyd200RWhTRjZ4R3RST1ZOTkVpU05WV2FqTWczTlJ1NkotU2EyQmR5S3ZuM3pWYXdXUzROQnp4cW0zM1U5QkFKcmc3ZXpPUkdYN0xYRVBhSFVmTTUwRmxHNTdHX1RzQm9uTXdvZWQwSktPc3Q5eDQ2VDFTeHlOblU1VUVtZk9OZ3NqN3NDNWhBZVZKbWNqQllmRGFTUFdCSTRxaEZBTWIyOXBhY0tfSjRvSzhHSHBpMzk1SDhVbHFvQ0YySzRKMVZ0QTRyMHhBRnowLUtDTnh5WE12MmNUdDJWTnA5My11WXU4dDAwNjhKZTBZd3htY0N4cE15MVNNdVYxMlJyLXVRTmQ3dW5NMVgtZHdxTWZ5enktaVpsamZVNW9JOVF4OFNKRFkxTW1ZeVl6VmlMVEF3TURBdE1qQm1NQzA1TkRreUxUVTRNalF5T1dNME5qSTRZeGdC"));
}
