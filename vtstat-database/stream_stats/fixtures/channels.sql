INSERT INTO
    vtubers (vtuber_id, native_name)
VALUES
    ('vtuber1', 'vtuber1');

INSERT INTO
    channels (
        channel_id,
        platform,
        platform_id,
        kind,
        vtuber_id
    )
VALUES
    (
        1,
        'youtube',
        'platform_channel_id1',
        'main',
        'vtuber1'
    );

INSERT INTO
    streams (
        stream_id,
        platform,
        platform_id,
        title,
        channel_id,
        status,
        vtuber_id
    )
VALUES
    (
        1,
        'youtube',
        'id1',
        'title1',
        1,
        'ended',
        'vtuber1'
    );