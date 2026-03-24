// BDD Cucumber runner 入口
// 讀取 tests/bdd/features/*.feature 並執行已定義的 step
//
// 執行方式：cargo test --test bdd
// 注意：未定義 step 的場景會被標示為 skipped（不會失敗）

mod steps;

use cucumber::World;

#[derive(Debug, Default, World)]
pub struct CodeBuddyWorld {
    /// HTTP 回應狀態碼
    pub response_status: Option<u16>,
    /// HTTP 回應 body（JSON string）
    pub response_body: Option<String>,
    /// 目前追蹤的 session ID
    pub session_id: Option<String>,
}

fn main() {
    // 指向專案根目錄的 BDD feature 檔案
    let features_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/bdd/features");

    futures::executor::block_on(
        CodeBuddyWorld::cucumber()
            .max_concurrent_scenarios(1)
            .run(features_path),
    );
}
