#[macro_use]
extern crate log;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;


pub struct ThreadPool {
    workers: Vec<Worker>,

    // 
    sender: mpsc::Sender<Job>,  
}



// このJobはthread::spawnに渡される。
// thread::spawn に渡す型から、 FnOnce() + Send + 'static を指定する。typeでJobという型名に。
// シングルスレッドサーバをマルチスレッド化する - The Rust Programming Language 日本語版 
// https://doc.rust-jp.rs/book-ja/ch20-02-multithreaded.html?highlight=FnOnce#%E3%82%B9%E3%83%AC%E3%83%83%E3%83%89%E3%83%97%E3%83%BC%E3%83%AB%E3%81%A7%E3%82%B9%E3%83%AB%E3%83%BC%E3%83%97%E3%83%83%E3%83%88%E3%82%92%E5%90%91%E4%B8%8A%E3%81%95%E3%81%9B%E3%82%8B
// 
// type Jobは type alias. Jobという短縮名で定義する。
// 高度な型 - The Rust Programming Language 日本語版
// https://doc.rust-jp.rs/book-ja/ch19-04-advanced-types.html?highlight=type%20alias#%E5%9E%8B%E3%82%A8%E3%82%A4%E3%83%AA%E3%82%A2%E3%82%B9%E3%81%A7%E5%9E%8B%E5%90%8C%E7%BE%A9%E8%AA%9E%E3%82%92%E7%94%9F%E6%88%90%E3%81%99%E3%82%8B

// FnOnce:Trait std::ops::FnOnc
// FnOnce in std::ops - Rust https://doc.rust-lang.org/std/ops/trait.FnOnce.html
// Rustのクロージャtraitについて調べた(FnOnce, FnMut, Fn) - Qiita https://qiita.com/shortheron/items/c1735dc4c7c78b0b55e9


// Sendでスレッド間の所有権の転送を許可する
// SyncとSendトレイトで拡張可能な並行性 - The Rust Programming Language 日本語版 https://doc.rust-jp.rs/book-ja/ch16-04-extensible-concurrency-sync-and-send.html

// Sync allows an object to to be used by two threads A and B at the same time. 
// Send allows an object to be used by two threads A and B at different times.

//  'staticは全プログラム期間で生存するLifeTime（ずっと参照可能なstatic。）
// ライフタイムで参照を検証する - The Rust Programming Language 日本語版 
// https://doc.rust-jp.rs/book-ja/ch10-03-lifetime-syntax.html?highlight=%27static#%E9%9D%99%E7%9A%84%E3%83%A9%E3%82%A4%E3%83%95%E3%82%BF%E3%82%A4%E3%83%A0


type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();   // Threadとメッセージをやりとりするチャネル。(tx,rx)が返る。

        // std::sync::Arc; スマートポインタ。参照カウント方式、スレッドセーフ。(同様のstd::rc::Rc はスレッドセーフではない。)
        // 
        // Arc::new(Mutex::new(receiver)) 
        // Arc<Mutex<T>>という形はデザインパターン - Rustコトハジメ https://rustforbeginners.hatenablog.com/entry/arc-mutex-design-pattern
        // 変数を複数のスレッドで共有する場合、
        //   参照を渡す(move)１つ目のmoveであとは無効でNG
        //   Arc::newし、cloneを渡す。参照のみOK
        //   Arc::new(Mutex::new(x)) のCloneを渡す。lockして変更可。
        let receiver = Arc::new(Mutex::new(receiver)); 

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));   
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);  // スマートポインタ。ヒープに割り当てられる。スコープを抜けると自動で解放される。

        self.sender.send(job).unwrap(); //  チャネル経由で、関数ポインタ(JOB)を送信する。
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}


impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let builder = thread::Builder::new()
            .name(format!("worker{}",id).into());

        let thread = builder.spawn(move || loop {       // loopで無限処理している。
            let job = receiver.lock().expect("🟥🟥🟥　lock erorr.").recv().expect("🟥🟥🟥　recv erorr."); // チャネルのレシーバーをlockし、recvする。

            debug!("Worker {} got a job; executing.", id);

            job();  // 関数ポインタ実行
        }).unwrap();
 

        Worker { id, thread }
    }
}
