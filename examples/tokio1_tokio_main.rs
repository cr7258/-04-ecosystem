use std::{thread, time::Duration};

use tokio::{
    fs,
    // time::sleep,
};

fn expensive_blocking_task(s: String) -> String {
    thread::sleep(Duration::from_millis(8000));
    blake3::hash(s.as_bytes()).to_string()
}

#[tokio::main]
async fn main() {
    let handler1 = tokio::spawn(async {
        println!("future 1");
        let content = fs::read("Cargo.toml").await.unwrap();
        println!("content: {:?}", content.len());
    });

    let handler2 = tokio::spawn(async {
        println!("future 2");
        let result = expensive_blocking_task("hello".to_string());
        println!("result: {}", result);
    });

    // sleep 的时候主线程会暂时让出控制权，而运行时中的其他线程会继续执行异步任务。
    // 因此，即使主线程的 sleep 时间很短，异步任务仍然可以在后台完成执行。
    // sleep(Duration::from_millis(1)).await;

    // 但是更好的方式是使用 tokio::join! 来等待所有的异步任务完成。
    let (res1, res2) = tokio::join!(handler1, handler2);

    if let Err(e) = res1 {
        println!("Error in handle1: {:?}", e);
    }
    if let Err(e) = res2 {
        println!("Error in handle2: {:?}", e);
    }
}
