use colored::*;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::io::{self, Write};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

const SERVER_URL: &str = "ws://127.0.0.1:8080";

#[derive(Debug)]
enum Command {
    Connect,
    ConnectMultiple(usize),
    Close(usize),
    CloseAll,
    List,
    Send(usize, String),
    Help,
    Quit,
}

struct Connection {
    id: usize,
    tx: mpsc::UnboundedSender<Message>,
}

#[tokio::main]
async fn main() {
    println!("{}", "=== WebSocket Test Client ===".bright_blue().bold());
    println!("Type 'help' for available commands\n");

    let mut connections: HashMap<usize, Connection> = HashMap::new();
    let mut next_id = 1;

    loop {
        print!("{} ", ">".bright_green().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match parse_command(input) {
            Ok(Command::Connect) => {
                match create_connection(next_id, SERVER_URL).await {
                    Ok((id, tx, handle)) => {
                        connections.insert(id, Connection { id, tx });
                        tokio::spawn(handle);
                        println!("{} Connection #{} established", "✓".green(), id);
                        next_id += 1;
                    }
                    Err(e) => {
                        println!("{} Failed to connect: {}", "✗".red(), e);
                    }
                }
            }
            Ok(Command::ConnectMultiple(count)) => {
                if count == 0 || count > 20 {
                    println!("{} Please specify a number between 1 and 20", "✗".red());
                    continue;
                }
                println!("Creating {} connections...", count);
                for _ in 0..count {
                    match create_connection(next_id, SERVER_URL).await {
                        Ok((id, tx, handle)) => {
                            connections.insert(id, Connection { id, tx });
                            tokio::spawn(handle);
                            println!("{} Connection #{} established", "✓".green(), id);
                            next_id += 1;
                        }
                        Err(e) => {
                            println!("{} Failed to connect: {}", "✗".red(), e);
                            break;
                        }
                    }
                }
            }
            Ok(Command::Close(id)) => {
                if let Some(conn) = connections.remove(&id) {
                    let _ = conn.tx.send(Message::Close(None));
                    println!("{} Closed connection #{}", "✓".green(), id);
                } else {
                    println!("{} Connection #{} not found", "✗".red(), id);
                }
            }
            Ok(Command::CloseAll) => {
                let count = connections.len();
                for (_, conn) in connections.drain() {
                    let _ = conn.tx.send(Message::Close(None));
                }
                println!("{} Closed {} connection(s)", "✓".green(), count);
            }
            Ok(Command::List) => {
                if connections.is_empty() {
                    println!("No active connections");
                } else {
                    println!("{}", "Active connections:".bright_yellow());
                    let mut ids: Vec<_> = connections.keys().collect();
                    ids.sort();
                    for id in ids {
                        println!("  • Connection #{}", id);
                    }
                }
            }
            Ok(Command::Send(id, message)) => {
                if let Some(conn) = connections.get(&id) {
                    if conn.tx.send(Message::Text(message.clone())).is_ok() {
                        println!("{} Sent to connection #{}: {}", "✓".green(), id, message);
                    } else {
                        println!("{} Failed to send message to #{}", "✗".red(), id);
                    }
                } else {
                    println!("{} Connection #{} not found", "✗".red(), id);
                }
            }
            Ok(Command::Help) => {
                print_help();
            }
            Ok(Command::Quit) => {
                println!("Closing all connections and exiting...");
                for (_, conn) in connections.drain() {
                    let _ = conn.tx.send(Message::Close(None));
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                break;
            }
            Err(e) => {
                println!("{} {}", "✗".red(), e);
            }
        }
    }
}

async fn create_connection(
    id: usize,
    url: &str,
) -> Result<
    (
        usize,
        mpsc::UnboundedSender<Message>,
        tokio::task::JoinHandle<()>,
    ),
    Box<dyn std::error::Error>,
> {
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Receive messages from the server
                msg = read.next() => {
                    match msg {
                        Some(Ok(message)) => {
                            match message {
                                Message::Text(text) => {
                                    println!("\n{} Connection #{}: {}", "←".cyan(), id, text);
                                    print!("{} ", ">".bright_green().bold());
                                    io::stdout().flush().unwrap();
                                }
                                Message::Binary(data) => {
                                    println!("\n{} Connection #{}: Received {} bytes", "←".cyan(), id, data.len());
                                    print!("{} ", ">".bright_green().bold());
                                    io::stdout().flush().unwrap();
                                }
                                Message::Close(_) => {
                                    println!("\n{} Connection #{} closed by server", "!".yellow(), id);
                                    print!("{} ", ">".bright_green().bold());
                                    io::stdout().flush().unwrap();
                                    break;
                                }
                                Message::Ping(_) => {
                                    // Pings are handled automatically by the library
                                }
                                Message::Pong(_) => {
                                    // Pong received
                                }
                                _ => {}
                            }
                        }
                        Some(Err(e)) => {
                            println!("\n{} Connection #{} error: {}", "✗".red(), id, e);
                            print!("{} ", ">".bright_green().bold());
                            io::stdout().flush().unwrap();
                            break;
                        }
                        None => {
                            break;
                        }
                    }
                }
                // Send messages to the server
                msg = rx.recv() => {
                    if let Some(message) = msg {
                        if write.send(message).await.is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    });

    Ok((id, tx, handle))
}

fn parse_command(input: &str) -> Result<Command, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    match parts[0].to_lowercase().as_str() {
        "connect" | "c" => {
            if parts.len() == 1 {
                Ok(Command::Connect)
            } else if parts.len() == 2 {
                let count = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "Invalid number".to_string())?;
                Ok(Command::ConnectMultiple(count))
            } else {
                Err("Usage: connect [count]".to_string())
            }
        }
        "close" => {
            if parts.len() == 1 {
                Err("Usage: close <id> or close all".to_string())
            } else if parts[1].to_lowercase() == "all" {
                Ok(Command::CloseAll)
            } else {
                let id = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "Invalid connection ID".to_string())?;
                Ok(Command::Close(id))
            }
        }
        "list" | "ls" => Ok(Command::List),
        "send" | "s" => {
            if parts.len() < 3 {
                Err("Usage: send <id> <message>".to_string())
            } else {
                let id = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "Invalid connection ID".to_string())?;
                let message = parts[2..].join(" ");
                Ok(Command::Send(id, message))
            }
        }
        "help" | "h" => Ok(Command::Help),
        "quit" | "exit" | "q" => Ok(Command::Quit),
        _ => Err(format!("Unknown command: '{}'. Type 'help' for available commands", parts[0])),
    }
}

fn print_help() {
    println!("\n{}", "Available Commands:".bright_yellow().bold());
    println!("  {}  {}  - Create a new WebSocket connection", "connect".bright_cyan(), "[count]".dimmed());
    println!("  {}     {}  - Alias for connect", "c".bright_cyan(), "[count]".dimmed());
    println!("  {}    {}  - Close a connection (or 'all')", "close".bright_cyan(), "<id|all>".dimmed());
    println!("  {}          - List all active connections", "list".bright_cyan());
    println!("  {}            - Alias for list", "ls".bright_cyan());
    println!("  {} {} - Send a message to a connection", "send".bright_cyan(), "<id> <message>".dimmed());
    println!("  {}      {} - Alias for send", "s".bright_cyan(), "<id> <message>".dimmed());
    println!("  {}          - Show this help message", "help".bright_cyan());
    println!("  {}            - Alias for help", "h".bright_cyan());
    println!("  {}    - Quit the client", "quit".bright_cyan());
    println!("  {}    - Alias for quit", "exit".bright_cyan());
    println!("  {}      - Alias for quit", "q".bright_cyan());
    println!("\n{}", "Examples:".bright_yellow().bold());
    println!("  connect       - Create 1 connection");
    println!("  connect 5     - Create 5 connections");
    println!("  list          - Show all connections");
    println!("  send 1 hello  - Send 'hello' to connection #1");
    println!("  close 1       - Close connection #1");
    println!("  close all     - Close all connections");
    println!();
}
