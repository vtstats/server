mod continuation;
mod replay_continuation;

use base64::{engine::general_purpose::URL_SAFE, Engine};
use quick_protobuf::{MessageWrite, Result, Writer};

pub fn get_continuation(channel_id: &str, stream_id: &str) -> Result<String> {
    use continuation::{Continuation, Video, A, B, C, D, E, F};

    let mut out = Vec::new();

    Continuation {
        b: B {
            video: {
                let mut out = Vec::new();

                Video {
                    d: D {
                        c: C {
                            channel_id: channel_id.into(),
                            video_id: stream_id.into(),
                        },
                    },
                    f: F {
                        e: E {
                            video_id: stream_id.into(),
                        },
                    },
                    s4: 1,
                }
                .write_message(&mut Writer::new(&mut out))?;

                URL_SAFE.encode(&out).into()
            },
            f6: 1,
            a: A { f1: 1 },
        },
    }
    .write_message(&mut Writer::new(&mut out))?;

    Ok(URL_SAFE.encode(&out))
}

pub fn get_replay_continuation(channel_id: &str, stream_id: &str) -> Result<String> {
    use replay_continuation::{
        Continuation, ContinuationA, ContinuationB, ContinuationC, Video, VideoA, VideoB, VideoC,
        VideoD,
    };

    let mut out = Vec::new();

    Continuation {
        f156074452: ContinuationA {
            f3: {
                let mut out = Vec::new();

                Video {
                    f1: VideoA {
                        f5: VideoB {
                            f1: channel_id.into(),
                            f2: stream_id.into(),
                        },
                    },
                    f3: VideoC {
                        f48687757: VideoD {
                            f1: stream_id.into(),
                        },
                    },
                    f4: 1,
                    f6: 0,
                }
                .write_message(&mut Writer::new(&mut out))?;

                URL_SAFE.encode(&out).into()
            },
            f8: 1,
            f10: ContinuationB {
                f4: 0,
                f22: 0,
                f31: 0,
            },
            f14: ContinuationC {
                f1: 1,
                f3: 1,
                f4: 0,
            },
        },
    }
    .write_message(&mut Writer::new(&mut out))?;

    Ok(URL_SAFE.encode(&out))
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

    assert_eq!(
        "op2w0wRyGlxDaWtxSndvWVZVTnJTV2x0VjFvNVowSktVbUZ0UzBZd2NtMVFWVGgzRWd0aFRVcGxRakIzYUVVNU5Cb1Q2cWpkdVFFTkNndGhUVXBsUWpCM2FFVTVOQ0FCTUFBPUABUgggALABAPgBAHIGCAEYASAA",
        get_replay_continuation(
            "UCkIimWZ9gBJRamKF0rmPU8w",
            "aMJeB0whE94",
        ).unwrap()
    );
}
