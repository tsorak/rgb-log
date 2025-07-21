use std::sync::Arc;
use tokio::sync::RwLock;

pub use in_memory::InMemory;
pub use on_disk::OnDisk;
pub use smart::Smart;

pub trait LogBuffer: 'static + Send + Sync {
    //async fn push_line<S: Sized + AsRef<str> + ToString + Send>(&self, line: S);
    fn push_line(&self, line: String);
}

impl LogBuffer for Option<()> {
    fn push_line(&self, _line: String) {}
}

mod on_disk {
    use super::*;

    use std::path::Path;

    use tokio::{
        fs::File,
        io::{AsyncWriteExt, BufStream},
        sync::Mutex,
    };

    #[derive(Clone)]
    pub struct OnDisk {
        cursor: Arc<Mutex<BufStream<File>>>,
    }

    impl OnDisk {
        pub async fn new(p: impl AsRef<Path>) -> Result<Self, std::io::Error> {
            let f = File::options()
                .append(true)
                .create(true)
                .read(true)
                .open(p)
                .await?;

            let cursor = BufStream::new(f);

            Ok(Self {
                cursor: Arc::new(Mutex::new(cursor)),
            })
        }

        async fn push(&self, line: String) {
            let buffer = line.as_ref();

            let mut lock = self.cursor.lock().await;

            let _ = lock
                .write_all(buffer)
                .await
                .inspect_err(|err| eprintln!("rgb_log: ERROR WRITING OnDisk LOG: {err}"));
            let _ = lock
                .flush()
                .await
                .inspect_err(|err| eprintln!("rgb_log: ERROR FLUSHING OnDisk LOG: {err}"));
        }
    }

    impl LogBuffer for OnDisk {
        fn push_line(&self, line: String) {
            let buffer = self.clone();
            tokio::spawn(async move { buffer.push(line.to_string()).await });
        }
    }
}

mod in_memory {
    use super::*;

    #[derive(Clone)]
    pub struct InMemory {
        inner: Arc<RwLock<Vec<String>>>,
    }

    impl InMemory {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                inner: Arc::new(RwLock::new(vec![])),
            }
        }

        pub async fn push(&self, value: String) {
            self.inner.write().await.push(value);
        }
    }

    //impl LogBuffer for InMemory {
    //    async fn push_line<S: Sized + AsRef<str> + ToString>(&self, line: S) {
    //        self.push(line.to_string()).await;
    //    }
    //}
}

#[allow(unused)]
mod smart {
    use super::*;

    use std::{
        path::{Path, PathBuf},
        time::Duration,
    };

    use tokio::{
        fs::File,
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    };

    /// For lack of a better name...
    /// Keeps an in memory buffer, and offloads to disk when it hasn't been accessed for X amount
    /// of time.
    #[derive(Clone)]
    pub struct Smart {
        state: Arc<RwLock<State>>,
        loader: Loader,
    }

    impl Smart {
        pub fn new(log_path: impl Into<PathBuf>, in_memory_timeout: Option<Duration>) -> Self {
            let state = State {
                inner: vec![],
                status: Status::InMemory,
            };
            let state = Arc::new(RwLock::new(state));

            let loader = Loader::new(
                state.clone(),
                in_memory_timeout.unwrap_or(Duration::from_secs(60 * 30)),
                log_path.into(),
            );

            Smart { state, loader }
        }

        pub async fn push(&self, value: String) {
            let lock = self.state.read().await;

            todo!();

            if let Status::OnDisk = lock.status {
                drop(lock);

                // !!!!!! todo: wait for a message from loader when loaded
                // this is all dumb. what if a reader hops in between when the memory buffer is
                // still empty!?
                self.load();
            } else {
                drop(lock);
            }

            let mut lock = self.state.write().await;
            lock.inner.push(value);
            drop(lock);

            self.extend_timeout();
        }

        fn load(&self) {
            self.loader.tx.send(Msg::Load).unwrap();
        }

        fn extend_timeout(&self) {
            self.loader.tx.send(Msg::ExtendTimeout).unwrap();
        }
    }

    enum Status {
        InMemory,
        OnDisk,
    }

    struct State {
        pub inner: Vec<String>,
        pub status: Status,
    }

    //
    enum Msg {
        Load,
        ExtendTimeout,
        Stop,
    }

    #[derive(Clone)]
    struct Loader {
        state: Arc<RwLock<State>>,
        tx: UnboundedSender<Msg>,
    }

    impl Loader {
        pub fn new(state: Arc<RwLock<State>>, timeout: Duration, path: PathBuf) -> Self {
            let (tx, rx) = unbounded_channel();

            let _handle = Self::main_loop(state.clone(), rx, timeout, path);

            Self { state, tx }
        }

        fn main_loop(
            state: Arc<RwLock<State>>,
            mut rx: UnboundedReceiver<Msg>,
            timeout: Duration,
            path: PathBuf,
        ) -> tokio::task::JoinHandle<()> {
            tokio::spawn(async move {
                let timeout = async || {
                    tokio::time::sleep(timeout).await;
                };

                loop {
                    tokio::select! {
                        _ = timeout() => {
                            Self::offload(state.clone(), &path).await;
                        }
                        Some(msg) = rx.recv() => {
                            match msg {
                                Msg::Load => Self::load(state.clone(), &path).await,
                                Msg::ExtendTimeout => continue,
                                Msg::Stop => break,
                            }
                        }
                    }
                }
            })
        }

        async fn load(state: Arc<RwLock<State>>, p: impl AsRef<Path>) {
            let mut lock = state.write().await;

            if let Status::InMemory = lock.status {
                eprintln!(
                    "rgb_log: ATTEMPT TO OVERWRITE IN-MEMORY LOGS CANCELLED. LINES IN MEMORY: {}",
                    lock.inner.len()
                );
                return;
            }

            match File::options().read(true).open(p.as_ref()).await {
                Ok(f) => match get_file_lines(f).await {
                    Ok(vec) => {
                        lock.inner = vec;
                    }
                    Err(ioerr) => {
                        eprintln!(
                            "rgb_log: FAILED TO READ LOG LINES '{}', ERROR: {ioerr}",
                            p.as_ref().to_string_lossy(),
                        );
                    }
                },
                Err(err) => {
                    eprintln!(
                        "rgb_log: FAILED TO OPEN LOG FOR READING {}, ERROR: {err}",
                        p.as_ref().to_string_lossy(),
                    );
                }
            };
        }

        async fn offload(state: Arc<RwLock<State>>, p: impl AsRef<Path>) {
            let mut lock = state.write().await;

            match File::options()
                .write(true)
                .truncate(true)
                .open(p.as_ref())
                .await
            {
                Ok(mut f) => {
                    let offloaded = std::mem::take(&mut lock.inner);

                    let bytes = offloaded
                        .clone()
                        .into_iter()
                        .map(|mut s| {
                            s.push('\n');
                            s.into_bytes()
                        })
                        .flatten()
                        .collect::<Vec<u8>>();

                    match f.write_all(&bytes).await {
                        Ok(_) => lock.status = Status::OnDisk,
                        Err(_) => {
                            lock.inner = offloaded;
                            eprintln!(
                                "rgb_log: FAILED TO OFFLOAD WRITE LOG {}",
                                p.as_ref().to_string_lossy()
                            );
                        }
                    }
                }
                Err(err) => {
                    eprintln!(
                        "rgb_log: FAILED TO OPEN LOG FOR WRITING {}, ERROR: {err}",
                        p.as_ref().to_string_lossy(),
                    );
                }
            };
        }
    }

    async fn get_file_lines(f: File) -> Result<Vec<String>, std::io::Error> {
        let reader = BufReader::new(f);
        let mut lines = reader.lines();

        let mut vec = vec![];

        while let Some(line) = lines.next_line().await? {
            vec.push(line);
        }

        Ok(vec)
    }
}
