INSERT INTO
    vtubers (vtuber_id, native_name)
VALUES
    ('vtuber1', 'vtuber1'),
    ('vtuber2', 'vtuber2'),
    ('vtuber3', 'vtuber3');

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
    ),
    (
        2,
        'youtube',
        'platform_channel_id2',
        'main',
        'vtuber2'
    ),
    (
        3,
        'youtube',
        'platform_channel_id3',
        'main',
        'vtuber3'
    );