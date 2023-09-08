use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use twitch_message::messages::{MessageKind, Ping, Privmsg};
use twitch_message::{parse, ParseResult};

use super::parse::{parse_privmsg, LiveChatMessage};

pub async fn connect_chat_room(login: String) -> anyhow::Result<BufReader<TcpStream>> {
    let stream = TcpStream::connect(twitch_message::TWITCH_IRC_ADDRESS).await?;
    let mut stream = BufReader::new(stream);

    // https://dev.twitch.tv/docs/irc/capabilities/
    stream
        .write(b"CAP REQ :twitch.tv/commands twitch.tv/membership twitch.tv/tags\r\n")
        .await?;
    stream.write(b"PASS justinfan2434\r\n").await?;
    stream.write(b"NICK justinfan2434\r\n").await?;

    let mut line = String::new();
    loop {
        line.clear();
        stream.read_line(&mut line).await?;

        if parse(&line)?.message.kind == MessageKind::Ready {
            stream
                .write(format!("JOIN #{login}\r\n").as_bytes())
                .await?;
            return Ok(stream);
        }
    }
}

pub async fn read_live_chat_message(
    stream: &mut BufReader<TcpStream>,
) -> anyhow::Result<LiveChatMessage> {
    let mut line = String::new();

    loop {
        line.clear();
        stream.read_line(&mut line).await?;

        let ParseResult { message, .. } = parse(&line)?;

        if message.as_typed_message::<Ping>().is_some() {
            stream.write(b"PONG :tmi.twitch.tv\r\n").await?;
        } else if let Some(msg) = message.as_typed_message::<Privmsg>() {
            if let Some(msg) = parse_privmsg(msg) {
                return Ok(msg);
            }
        }
    }
}
