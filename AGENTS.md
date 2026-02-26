# AGENTS.md - OpenCode 規則
# 由 Universal Dev Standards CLI 生成
# https://github.com/AsiaOstrich/universal-dev-standards

您是專業的軟體工程助手。請遵循以下專案標準。

## 對話語言 / Conversation Language
所有回覆必須使用**繁體中文 (Traditional Chinese)**。

## 核心標準使用規則 / Core Standards Usage Rule
> 當驗證標準、檢查程式碼或執行任務時，**優先**讀取 `core/` 中的精簡規則（例如 `core/testing-standards.md`）。
> 只有在被明確要求提供教育內容、詳細解釋或教學時，才讀取 `core/guides/` 或 `methodologies/guides/`。
> 這確保了 Token 效率和上下文聚焦。

---

<!-- UDS:STANDARDS:START -->
## 提交訊息語言
使用**雙語**格式撰寫提交訊息（英文 + 繁體中文）。
格式：`<type>(<scope>): <English>. <中文>.`

## Standards Compliance Instructions

**MUST follow** (每次都要遵守):
| Task | Standard | When |
|------|----------|------|
| Project context | [project-context-memory.ai.yaml](.standards/project-context-memory.ai.yaml) | Planning & Coding |

**SHOULD follow** (相關任務時參考):
| Task | Standard | When |
|------|----------|------|
| Developer memory | [developer-memory.ai.yaml](.standards/developer-memory.ai.yaml) | Always (protocol) |


## Installed Standards Index

本專案採用 **Level 3** 標準。所有規範位於 `.standards/`：

### Core (9 standards)
- `deployment-standards.ai.yaml` - deployment-standards.ai.yaml
- `documentation-writing-standards.ai.yaml` - documentation-writing-standards.ai.yaml
- `ai-agreement-standards.ai.yaml` - ai-agreement-standards.ai.yaml
- `virtual-organization-standards.ai.yaml` - virtual-organization-standards.ai.yaml
- `security-standards.ai.yaml` - security-standards.ai.yaml
- `performance-standards.ai.yaml` - performance-standards.ai.yaml
- `accessibility-standards.ai.yaml` - accessibility-standards.ai.yaml
- `developer-memory.ai.yaml` - 開發者持久記憶
- `project-context-memory.ai.yaml` - 專案情境記憶

<!-- UDS:STANDARDS:END -->

---
