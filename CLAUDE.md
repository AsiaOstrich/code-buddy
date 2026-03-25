# Claude Code 專案指南
# 由 Universal Dev Standards CLI 生成
# https://github.com/AsiaOstrich/universal-dev-standards

## 對話語言 / Conversation Language
所有回覆必須使用**繁體中文 (Traditional Chinese)**。
AI 助手應以繁體中文回覆使用者的問題與請求。

## 核心標準使用規則 / Core Standards Usage Rule
> 當驗證標準、檢查程式碼或執行任務時，**優先**讀取 `core/` 中的精簡規則（例如 `core/testing-standards.md`）。
> 只有在被明確要求提供教育內容、詳細解釋或教學時，才讀取 `core/guides/` 或 `methodologies/guides/`。
> 這確保了 Token 效率和上下文聚焦。

---

<!-- UDS:STANDARDS:START -->
<!-- WARNING: This block is managed by UDS (universal-dev-standards). DO NOT manually edit. Use 'npx uds install' or 'npx uds update' to modify. -->
<!-- WARNING: This block is managed by UDS (universal-dev-standards). DO NOT manually edit. Use 'npx uds install' or 'npx uds update' to modify. -->
## Commit Message Language
Write commit messages in **bilingual** format (English + 繁體中文).
Format: `<type>(<scope>): <English>. <中文>.`
Body MUST be bilingual: English first → blank line → Chinese second. NEVER mix languages in one paragraph.

## Standards Compliance Instructions

**MUST follow** (每次都要遵守):
| Task | Standard | When |
|------|----------|------|
| Project context | [project-context-memory.ai.yaml](.standards/project-context-memory.ai.yaml) | Planning & Coding |
| Writing commits | [commit-message.ai.yaml](.standards/commit-message.ai.yaml) | Every commit |
| Workflow gates | [workflow-enforcement.ai.yaml](.standards/workflow-enforcement.ai.yaml) | Before any workflow phase |

**SHOULD follow** (相關任務時參考):
| Task | Standard | When |
|------|----------|------|
| Developer memory | [developer-memory.ai.yaml](.standards/developer-memory.ai.yaml) | Always (protocol) |
| Git workflow | [git-workflow.ai.yaml](.standards/git-workflow.ai.yaml) | Branch/merge decisions |
| Writing tests | [testing.ai.yaml](.standards/testing.ai.yaml) | When creating tests |


## Installed Standards Index

本專案採用 UDS 標準。所有規範位於 `.standards/`：

### Core (52 standards)
- `deployment-standards.ai.yaml` - deployment-standards.ai.yaml
- `documentation-writing-standards.ai.yaml` - documentation-writing-standards.ai.yaml
- `ai-agreement-standards.ai.yaml` - ai-agreement-standards.ai.yaml
- `virtual-organization-standards.ai.yaml` - virtual-organization-standards.ai.yaml
- `security-standards.ai.yaml` - security-standards.ai.yaml
- `performance-standards.ai.yaml` - performance-standards.ai.yaml
- `accessibility-standards.ai.yaml` - accessibility-standards.ai.yaml
- `developer-memory.ai.yaml` - 開發者持久記憶
- `project-context-memory.ai.yaml` - 專案情境記憶
- `anti-hallucination.ai.yaml` - anti-hallucination.ai.yaml
- `ai-friendly-architecture.ai.yaml` - ai-friendly-architecture.ai.yaml
- `commit-message.ai.yaml` - 提交訊息格式
- `checkin-standards.ai.yaml` - checkin-standards.ai.yaml
- `api-design-standards.ai.yaml` - api-design-standards.ai.yaml
- `database-standards.ai.yaml` - database-standards.ai.yaml
- `spec-driven-development.ai.yaml` - spec-driven-development.ai.yaml
- `code-review.ai.yaml` - code-review.ai.yaml
- `git-workflow.ai.yaml` - Git 工作流程
- `versioning.ai.yaml` - versioning.ai.yaml
- `changelog.ai.yaml` - changelog.ai.yaml
- `test-governance.ai.yaml` - test-governance.ai.yaml
- `structured-task-definition.ai.yaml` - structured-task-definition.ai.yaml
- `workflow-state-protocol.ai.yaml` - workflow-state-protocol.ai.yaml
- `workflow-enforcement.ai.yaml` - 工作流程強制執行
- `testing.ai.yaml` - 測試標準
- `documentation-structure.ai.yaml` - documentation-structure.ai.yaml
- `ai-instruction-standards.ai.yaml` - ai-instruction-standards.ai.yaml
- `project-structure.ai.yaml` - project-structure.ai.yaml
- `error-codes.ai.yaml` - error-codes.ai.yaml
- `logging.ai.yaml` - logging.ai.yaml
- `test-completeness-dimensions.ai.yaml` - test-completeness-dimensions.ai.yaml
- `test-driven-development.ai.yaml` - test-driven-development.ai.yaml
- `behavior-driven-development.ai.yaml` - behavior-driven-development.ai.yaml
- `acceptance-test-driven-development.ai.yaml` - acceptance-test-driven-development.ai.yaml
- `reverse-engineering-standards.ai.yaml` - reverse-engineering-standards.ai.yaml
- `forward-derivation-standards.ai.yaml` - forward-derivation-standards.ai.yaml
- `refactoring-standards.ai.yaml` - refactoring-standards.ai.yaml
- `requirement-engineering.ai.yaml` - requirement-engineering.ai.yaml
- `context-aware-loading.ai.yaml` - context-aware-loading.ai.yaml
- `requirement-checklist.md` - requirement-checklist.md
- `requirement-template.md` - requirement-template.md
- `requirement-document-template.md` - requirement-document-template.md
- `pipeline-integration-standards.ai.yaml` - pipeline-integration-standards.ai.yaml
- `acceptance-criteria-traceability.ai.yaml` - acceptance-criteria-traceability.ai.yaml
- `change-batching-standards.ai.yaml` - change-batching-standards.ai.yaml
- `systematic-debugging.ai.yaml` - systematic-debugging.ai.yaml
- `agent-dispatch.ai.yaml` - agent-dispatch.ai.yaml
- `model-selection.ai.yaml` - model-selection.ai.yaml
- `git-worktree.ai.yaml` - git-worktree.ai.yaml
- `branch-completion.ai.yaml` - branch-completion.ai.yaml
- `verification-evidence.ai.yaml` - verification-evidence.ai.yaml
- `ai-command-behavior.ai.yaml` - ai-command-behavior.ai.yaml
<!-- UDS:STANDARDS:END -->

---
