// BDD Step 定義骨架
// TODO: v0.3.0 — 實作完整的 step 定義
//
// 本檔案為 Cucumber step 定義的進入點。
// 每個 step 對應 .feature 檔案中的 Given/When/Then。
// 目前僅提供範例骨架，未定義的 step 會被 cucumber-rs 標示為 skipped。

use cucumber::given;

use crate::CodeBuddyWorld;

#[given("Code Buddy 應用已啟動")]
fn app_started(_world: &mut CodeBuddyWorld) {
    // 骨架：實際整合測試需啟動 Axum test server
}
