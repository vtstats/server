use base64::{engine::general_purpose::URL_SAFE, Engine};
use continuation::{Continuation, A};
use quick_protobuf::{MessageWrite, Result, Writer};

pub fn get_continuation(video_id: &str, timestamp: i64) -> Result<String> {
    let mut out = Vec::new();

    Continuation {
        a: A {
            video: video_id.into(),
            timestamp,
            f7: 1,
            f8: 1,
        },
    }
    .write_message(&mut Writer::new(&mut out))?;

    Ok(URL_SAFE.encode(&out))
}

#[test]
fn test() {
    assert_eq!(
        get_continuation("HI-Ck3Tgmi4", 1662217569).unwrap(),
        "-of5rQMXGgtISS1DazNUZ21pNCDh2s2YBjgBQAE="
    );
}

pub mod continuation;
