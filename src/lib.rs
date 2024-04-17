//! Simple testing library for the Actor Model.
//! This takes inspiration from Alice Ryhl
//! https://www.youtube.com/watch?v=fTXuGRP1ee4
// use std::{io::Write, net::TcpStream};

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

pub struct Actor {
    pub receiver: mpsc::Receiver<ActorMessage>,
    pub connection: TcpStream,
}

impl Actor {
    pub fn new(recv: mpsc::Receiver<ActorMessage>, conn: TcpStream) -> Self {
        Self {
            receiver: recv,
            connection: conn,
        }
    }
}

impl Actor {
    pub async fn handle_message(&mut self, msg: ActorMessage) -> Result<()> {
        match msg {
            ActorMessage::SendMessage {
                message,
                respond_to,
            } => {
                self.connection.write_all(message.as_bytes()).await?;
                let response = self.connection.read_u32().await?;
                let _ = respond_to.send(response);
                Ok(())
            }
        }
    }
}

#[derive(Clone)]
pub struct ActorHandler {
    pub sender: mpsc::Sender<ActorMessage>,
}

impl ActorHandler {
    pub fn new(conn: TcpStream) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let actor = Actor::new(receiver, conn);
        tokio::spawn(run_my_actor(actor));

        Self { sender }
    }
}

pub enum ActorMessage {
    SendMessage {
        message: String,
        respond_to: oneshot::Sender<u32>,
    },
}

pub async fn run_my_actor(mut actor: Actor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg).await.unwrap()
    }
}
