use tokio::sync::mpsc::{channel, Receiver};
use notify::{Event, PollWatcher, RecursiveMode, Watcher, Config};
use std::{path::{Path, PathBuf}, time::Duration};
use std::future::Future;

const POLL_SECONDS: f64 = 2.0;

/// Directly taken from the notify async watcher example with slight modifications
fn async_watcher() -> notify::Result<(PollWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);


    let config = Config::default()
    .with_poll_interval(Duration::from_secs_f64(POLL_SECONDS))
    .with_compare_contents(true);

    let rt = tokio::runtime::Runtime::new().unwrap();

    let watcher = PollWatcher::new(move |res| {
        rt.block_on(async {
            tx.send(res).await.unwrap();
        })
    }, config)?;
    Ok((watcher, rx))
}

async fn async_watch<'a, P, F, Fut>(path: P, callback: F) -> notify::Result<()> 
where 
    P: AsRef<Path>,
    F: Fn(Event) -> Fut,
    Fut: Future<Output = ()>
{
    let (mut watcher, mut rx) = async_watcher()?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(event) = rx.recv().await {
        callback(event?).await
    }

    Ok(())
}

fn is_json(path: &PathBuf) -> bool {

    if path.extension().is_none() {
        return false
    } 

    if path.extension().unwrap() == "json" {
        return true
    }

    return false
}

async fn print_event(e: Event) {
    for path in &e.paths {
        if is_json(path) {
            if e.kind.is_create() || e.kind.is_modify() {
                println!("Redeploying...")
            } else {
                println!("Not Redeploying...")

            }
        }
    }
}

#[tokio::main]
pub async fn main() -> notify::Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    println!("watching {}", path);
    if let Err(e) = async_watch(path, print_event).await {
        println!("error: {:?}", e)
    }
    Ok(())
}