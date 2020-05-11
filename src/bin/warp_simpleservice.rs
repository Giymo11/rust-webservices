use warp::Filter;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let hi = warp::path!("hello" / String)
        .and(warp::header("user-agent"))
        .map(|param: String, agent: String| format!("Hello {}, whose agent is {}", param, agent));

    warp::serve(hi).run(([127, 0, 0, 1], 3031)).await;
}
