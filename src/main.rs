use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt, Future,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::{path::Path, time::Duration};

use std::error::Error;

/// Directly taken from the notify async watcher example with slight modifications
fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);


    let watcher = RecommendedWatcher::new(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        })
    }, Config::default())?;

    Ok((watcher, rx))
}

async fn async_watch<'a, P, F, Fut>(path: P, callback: F) -> notify::Result<()> 
where 
    P: AsRef<Path>,
    F: Fn(Event) -> Fut,
    Fut: Future<Output = ()>
{
    let config = Config::default()
    .with_poll_interval(Duration::from_secs(65))
    .with_compare_contents(false);

    let (mut watcher, mut rx) = async_watcher()?;
    watcher.configure(config).unwrap();
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(event) = rx.next().await {
        callback(event?).await
    }

    Ok(())
}

async fn print_event(e: Event) {
    if e.kind.is_modify() {
        for path in e.paths {
            println!("File at path {} modified", path.display())
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");
    println!("watching {}", path);
    if let Err(e) = async_watch(path, print_event).await {
        println!("error: {:?}", e)
    }
    Ok(())
}