mod continuation;
mod replay_continuation;

use base64::{engine::general_purpose::STANDARD, Engine};
use quick_protobuf::{MessageWrite, Result, Writer};

pub fn get_continuation(channel_id: &str, stream_id: &str) -> Result<String> {
    use continuation::{Continuation, Video, A, B, C, D, E, F};

    let mut out = Vec::new();

    Continuation {
        b: Some(B {
            video: {
                let mut out = Vec::new();

                Video {
                    d: Some(D {
                        c: Some(C {
                            channel_id: channel_id.into(),
                            video_id: stream_id.into(),
                        }),
                    }),
                    f: Some(F {
                        e: Some(E {
                            video_id: stream_id.into(),
                        }),
                    }),
                    s4: 1,
                }
                .write_message(&mut Writer::new(&mut out))?;

                STANDARD.encode(&out).into()
            },
            f6: 1,
            a: Some(A { f1: 1 }),
        }),
    }
    .write_message(&mut Writer::new(&mut out))?;

    Ok(STANDARD.encode(&out))
}

pub fn get_replay_continuation(channel_id: &str, stream_id: &str) -> Result<String> {
    use replay_continuation::{
        Continuation, ContinuationA, ContinuationB, ContinuationC, Video, VideoA, VideoB, VideoC,
        VideoD,
    };

    let mut out = Vec::new();

    Continuation {
        f156074452: Some(ContinuationA {
            f3: {
                let mut out = Vec::new();

                Video {
                    f1: Some(VideoA {
                        f5: Some(VideoB {
                            f1: channel_id.into(),
                            f2: stream_id.into(),
                        }),
                    }),
                    f3: Some(VideoC {
                        f48687757: Some(VideoD {
                            f1: stream_id.into(),
                        }),
                    }),
                    f4: 1,
                    f6: 0,
                }
                .write_message(&mut Writer::new(&mut out))?;

                STANDARD.encode(&out).into()
            },
            f8: 1,
            f10: Some(ContinuationB {
                f4: 0,
                f22: 0,
                f31: 0,
            }),
            f14: Some(ContinuationC {
                f1: 1,
                f3: 1,
                f4: 0,
            }),
        }),
    }
    .write_message(&mut Writer::new(&mut out))?;

    Ok(STANDARD.encode(&out))
}

#[test]
fn test() {
    assert_eq!(
        "0ofMyANhGlhDaWtxSndvWVZVTnhiVE5DVVV4c1NtWjJhMVJ6V0Y5b2RtMHdWVzFCRWd0eFRtVkthRXBXUkU5VE9Cb1Q2cWpkdVFFTkNndHhUbVZLYUVwV1JFOVRPQ0FCMAGCAQIIAQ==",
        get_continuation(
            "UCqm3BQLlJfvkTsX_hvm0UmA",
            "qNeJhJVDOS8",
        ).unwrap()
    );
}
