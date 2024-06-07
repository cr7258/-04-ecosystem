use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
const MAX_MESSAGES: usize = 128;

// 用存储和管理所有连接到服务器的客户端，每个客户端地址映射到一个消息发送通道。
// mpsc::Sender：mpsc（多生产者，单消费者）通道允许从多个生产者发送消息到一个消费者，例如当有新的客户端连接或者离开时，向所有客户端广播这条消息。
// 使用 Arc<Message> 是为了在多个任务之间高效地共享消息，而不需要复制消息的内容。
#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

// 表示单个客户端连接，包含用户名和用于处理消息流的读取部分。
// username 表示客户端的用户名。
// SplitStream<Framed<TcpStream, LinesCodec>> 类型，表示客户端的网络流，它被分割成了读取和写入两部分，可以用于并发地读取和写入数据。
#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

// 表示不同类型的消息，包含用户加入、用户离开和聊天消息。
#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // console_subscriber::init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Starting chat server on {}", addr);
    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accepted connection from: {}", addr);
        let state_cloned = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(state_cloned, addr, stream).await {
                warn!("Failed to handle client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    // Framed 是一个封装，它将底层的 I/O 流（如 TcpStream）与一个编码器/解码器（Codec）组合在一起，提供了一个异步的、分块处理的流接口。这使得我们能够以更高层次的抽象来处理数据，而不必关心底层的字节操作。
    // LinesCodec 是 tokio_util::codec 提供的一个编码器/解码器，它专门用于处理基于行的文本协议。它能够将字节流解析为一行一行的文本，或者将文本编码为字节流。
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(addr, username, stream).await;

    let message = Arc::new(Message::user_joined(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {}", addr, e);
                break;
            }
        };

        let message = Arc::new(Message::chat(&peer.username, line));

        state.broadcast(addr, message).await;
    }

    // when while loop exit, peer has left the chat or line reading failed
    // remove peer from state
    state.peers.remove(&addr);

    // notify others that a user has left
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);

    state.broadcast(addr, message).await;

    Ok(())
}

impl State {
    // 当需要广播消息时，服务器会遍历 DashMap 中的所有 mpsc::Sender，并通过它们将消息发送给每个客户端。
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("Failed to send message to {}: {}", peer.key(), e);
                // if send failed, peer might be gone, remove peer from state
                self.peers.remove(peer.key());
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);
        self.peers.insert(addr, tx);

        // ask user for username
        let (mut stream_sender, stream_receiver) = stream.split();

        // receive messages from others, and send them to the client
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        Peer {
            username,
            stream: stream_receiver,
        }
    }
}

impl Message {
    fn user_joined(username: &str) -> Self {
        let content = format!("{} has joined the chat", username);
        Self::UserJoined(content)
    }

    fn user_left(username: &str) -> Self {
        let content = format!("{} has left the chat", username);
        Self::UserLeft(content)
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserJoined(content) => write!(f, "[{}]", content),
            Self::UserLeft(content) => write!(f, "[{} :(]", content),
            Self::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}
