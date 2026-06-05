use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum UciOutput {
    BestMove(String),
    Info { depth: u32, score: i32, pv: Option<String> },
    Ready,
}

pub struct UciEngine {
    stdin_tx: Sender<String>,
    pub output_rx: Receiver<UciOutput>,
    _child: Child,
}

impl UciEngine {
    pub fn launch(path: &str) -> anyhow::Result<Self> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let (stdin_tx, stdin_rx) = mpsc::channel::<String>(32);
        let (output_tx, output_rx) = mpsc::channel::<UciOutput>(64);

        tokio::spawn(stdin_writer(stdin, stdin_rx));
        tokio::spawn(stdout_reader(stdout, output_tx));

        Ok(Self { stdin_tx, output_rx, _child: child })
    }

    pub fn send(&self, cmd: &str) {
        let _ = self.stdin_tx.try_send(cmd.to_string());
    }

    pub fn init(&self) {
        self.send("uci");
        self.send("isready");
    }

    pub fn new_game(&self) {
        self.send("ucinewgame");
        self.send("isready");
    }

    pub fn go(&self, position: &str, depth: u32) {
        self.send(position);
        self.send(&format!("go depth {depth}"));
    }

    pub fn stop(&self) {
        self.send("stop");
    }
}

async fn stdin_writer(mut stdin: ChildStdin, mut rx: Receiver<String>) {
    while let Some(cmd) = rx.recv().await {
        let line = format!("{cmd}\n");
        if stdin.write_all(line.as_bytes()).await.is_err() {
            break;
        }
    }
}

async fn stdout_reader(stdout: ChildStdout, tx: Sender<UciOutput>) {
    let mut reader = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = reader.next_line().await {
        let parsed = parse_line(&line);
        if let Some(msg) = parsed {
            if tx.send(msg).await.is_err() {
                break;
            }
        }
    }
}

fn parse_line(line: &str) -> Option<UciOutput> {
    if line == "readyok" {
        return Some(UciOutput::Ready);
    }
    if let Some(rest) = line.strip_prefix("bestmove ") {
        let mv = rest.split_whitespace().next().unwrap_or("0000").to_string();
        return Some(UciOutput::BestMove(mv));
    }
    if line.starts_with("info") {
        let depth = extract_u32(line, "depth");
        let score = extract_i32(line, "cp");
        let pv = extract_after(line, "pv");
        if depth.is_some() || score.is_some() {
            return Some(UciOutput::Info {
                depth: depth.unwrap_or(0),
                score: score.unwrap_or(0),
                pv,
            });
        }
    }
    None
}

fn extract_u32(s: &str, key: &str) -> Option<u32> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let idx = tokens.iter().position(|&t| t == key)?;
    tokens.get(idx + 1)?.parse().ok()
}

fn extract_i32(s: &str, key: &str) -> Option<i32> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let idx = tokens.iter().position(|&t| t == key)?;
    tokens.get(idx + 1)?.parse().ok()
}

fn extract_after(s: &str, key: &str) -> Option<String> {
    let tokens: Vec<&str> = s.split_whitespace().collect();
    let idx = tokens.iter().position(|&t| t == key)?;
    let rest: Vec<&str> = tokens[idx + 1..].to_vec();
    if rest.is_empty() { None } else { Some(rest.join(" ")) }
}