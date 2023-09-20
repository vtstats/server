use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use twitch_message::messages::{MessageKind, Ping, Privmsg};
use twitch_message::{parse, ParseResult};

use super::parse::{parse_privmsg, LiveChatMessage};

pub async fn connect_chat_room(login: &str) -> anyhow::Result<BufReader<TcpStream>> {
    let stream = TcpStream::connect(twitch_message::TWITCH_IRC_ADDRESS).await?;
    let mut stream = BufReader::new(stream);

    // https://dev.twitch.tv/docs/irc/capabilities/
    stream
        .write_all(b"CAP REQ :twitch.tv/commands twitch.tv/membership twitch.tv/tags\r\n")
        .await?;
    stream.write_all(b"PASS justinfan2434\r\n").await?;
    stream.write_all(b"NICK justinfan2434\r\n").await?;

    let mut line = String::new();
    loop {
        line.clear();
        stream.read_line(&mut line).await?;

        if parse(&line)?.message.kind == MessageKind::Ready {
            stream
                .write_all(format!("JOIN #{login}\r\n").as_bytes())
                .await?;
            return Ok(stream);
        }
    }
}

pub async fn read_live_chat_message(
    stream: &mut BufReader<TcpStream>,
) -> anyhow::Result<Option<LiveChatMessage>> {
    let mut line = String::new();

    loop {
        line.clear();
        stream.read_line(&mut line).await?;

        // Sometime tcp stream will only emit nothing but empty string
        // in that cases, we just reconnect the chat room
        if line.trim().is_empty() {
            return Ok(None);
        }

        let Ok(ParseResult { message, .. }) = parse(&line) else {
            tracing::warn!("failed to parse message line {line}");
            continue;
        };

        if message.as_typed_message::<Ping>().is_some() {
            stream.write_all(b"PONG :tmi.twitch.tv\r\n").await?;
        } else if let Some(msg) = message.as_typed_message::<Privmsg>() {
            if let Some(msg) = parse_privmsg(msg) {
                return Ok(Some(msg));
            }
        }
    }
}
