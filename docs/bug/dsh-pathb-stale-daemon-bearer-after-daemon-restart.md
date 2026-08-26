# dsh Path B「API key 无效」与 daemon 重启后 stale `DAEMON_BEARER`

- **状态**: 已复现；产品修复在 `P8-T16`（`cognitive dsh web` helper 检测 401 后重签发 management bearer）
- **环境**: `B01-Desktop-Linux-002`（`hal9001@192.168.123.160`），runtime `/home/hal9001/p8t13-owner-ops/runtime`
- **相关任务/能力**: P8-T10 / P8-T15 / P8-T16，dsh Path B，`cognitive dsh web`
- **发现日期**: 2026-08-26

## 症状

- Control Plane（`/ui/`）中 LongCat Provider 账户显示 **active**，模型 `LongCat-2.0` 已绑定到 `agent://personal/dsh`。
- 原生 dsh 控制面板（`http://127.0.0.1:3080/`）在 Models 或对话中提示 **API key 无效**（或等价凭据错误）。
- `cognitive doctor` 可能仍显示 provider 组件异常（例如遗留 `provider.json` 与 `selected-model` digest 不一致），但 **这不是 dsh Path B 的直接凭据来源**。

## 机制概览

dsh **Path B** 不把 LongCat API key 写在面板或 `.env` 里。聊天路径是：

```text
Cos dsh web (llm-deepseek)
  → apiKeyEnv: DAEMON_BEARER（读 $DSH_HOME/.credentials.yaml）
  → POST http://127.0.0.1:<daemon>/provider/v1/dsh/chat/completions
  → daemon 用 management session 鉴权
  → Provider Control Plane 绑定 + Secret Store 中的 LongCat key
  → https://api.longcat.chat/openai/v1
```

因此：

| 用户看到的 | 实际校验的对象 |
|-----------|----------------|
| Control Plane「LongCat 已配置」 | 账户 `secret_ref` 在 Secret Store 可解析；绑定存在 |
| dsh「API key 无效」 | **`DAEMON_BEARER` 管理会话 token 无效或过期**，daemon 在 Path B 入口返回 401 |

两者可以同时成立：上游 LongCat 密钥没问题，但 **loopback 上的 daemon 会话**已失效。

## 根因

### 1. `DAEMON_BEARER` 是会话 token，不是 Provider API key

`cognitive dsh web` 启动 helper（`dsh-real-process.mjs`）时：

1. 用 `local-bootstrap.secret` 向 daemon 申请 **management** 与 **task** 会话；
2. 将 management token 写入 `$DSH_HOME/.credentials.yaml` 的 `DAEMON_BEARER` 与 `DEEPSEEK_API_KEY` 引用；
3. 在 `settings.yaml` 中配置 `llm-deepseek.apiKeyEnv: DAEMON_BEARER`。

Cos 子进程 **禁止**在环境变量中注入 `BEARER`/`API_KEY` 形状的秘密；它通过 credentials 引用解析 `DAEMON_BEARER`。

### 2. daemon 重启会使旧 management session 失效

对 `kernel-server` 执行 `cognitive daemon stop` / `start` 或替换二进制并重启后：

- 进程内 `LocalSessionAuthority` 重新初始化；
- 此前签发的 `sess-...` management token **不再被接受**；
- `POST /provider/v1/dsh/chat/completions` 在缺少/错误 bearer 时返回：

```json
{
  "error": {
    "code": "LOCAL_SESSION_UNAUTHORIZED",
    "message": "authorization bearer required"
  }
}
```

### 3. 修复前：dsh web 凭据不会随 daemon 自动刷新

若仅重启 daemon 而 **不**重启 `cognitive dsh web`（或未成功执行 `cognitive dsh apply`）：

- `$DSH_HOME/.credentials.yaml` 仍保留旧 `sess-...`；
- 3080 上 Cos/node 可能仍为 **CRASHED** / `process_alive: false`，或存活但持旧凭据；
- dsh UI 将上游 401 呈现为「API key 无效」，容易误判为 LongCat Provider 未配置。

`P8-T16` 让仍在运行的 `cognitive dsh web` helper 探测 `GET /personal/dsh/runtime`：401
`LOCAL_SESSION_UNAUTHORIZED` / `LOCAL_SESSION_EXPIRED` 时从 bootstrap 重签发
management session、重写 `.credentials.yaml` 并重载 Cos。该 401 **不得**被当成
dsh 未绑定。会话仍只存在于 daemon 进程内，不落盘。

### 4. 与 Control Plane 显示不一致的常见混淆点

| 事实 | 说明 |
|------|------|
| `provider.json` 仍指向 `deepseek` | `cognitive init` 遗留；dsh Path B **不读**此文件 |
| `selected-model.json` 为 `LongCat-2.0` | 来自 Control Plane 绑定；与 Path B 路由一致 |
| LongCat 账户 `active` | Secret Store + 目录发现正常 |
| dsh 仍报错 | **loopback bearer** 失效，与 LongCat 账户状态无关 |

## 复现条件

1. runtime 上 LongCat 账户与 dsh 绑定已正确配置；
2. `cognitive dsh web` 曾成功启动并写入 `.credentials.yaml`；
3. **在不重启 dsh web 的情况下**重启 daemon（例如部署新 `kernel-server`、换 UI 包、修 lock 冲突后 `daemon start`）；
4. 在 dsh 面板发起对话或打开 Models 校验。

## 诊断步骤

在 guest 上（替换实际 runtime 路径）：

```bash
RUNTIME=/home/hal9001/p8t13-owner-ops/runtime
BINDIR=/home/hal9001/dshfix-53ea437f/bin   # 或当前 cognitive 二进制路径

# 1. Provider 侧（Control Plane）
$BINDIR/cognitive provider account list --runtime-root "$RUNTIME"
$BINDIR/cognitive agent binding list --runtime-root "$RUNTIME"

# 2. dsh 运行时
$BINDIR/cognitive dsh status --runtime-root "$RUNTIME"
# 关注 state: CRASHED / process_alive: false

# 3. Path B 配置面（无 secret 内容）
cat "$RUNTIME/cognitiveos/dsh-web-home/settings.yaml"
# 期望 llm-deepseek.baseURL → http://127.0.0.1:<port>/provider/v1/dsh
# 期望 apiKeyEnv: DAEMON_BEARER

test -f "$RUNTIME/cognitiveos/dsh-web-home/.credentials.yaml" && echo credentials_present

# 4. 用 credentials 中的 bearer 探测 Path B（勿把 token 记入日志/聊天）
TOKEN=$(grep DAEMON_BEARER "$RUNTIME/cognitiveos/dsh-web-home/.credentials.yaml" | head -1 | sed 's/.*: //' | tr -d '"')
curl -s -m 60 -X POST "http://127.0.0.1:48681/provider/v1/dsh/chat/completions" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"model":"LongCat-2.0","messages":[{"role":"user","content":"ping"}],"max_tokens":5}'
# 401 LOCAL_SESSION_UNAUTHORIZED → stale bearer（本 bug）
# 200 + model LongCat-2.0 → Path B 正常
```

## 恢复步骤

**产品修复（P8-T16）后**：只要 `cognitive dsh web` helper 仍在运行，daemon 重启后
helper 会自行重签发 bearer 并重载 Cos。刷新浏览器 `http://127.0.0.1:3080/` 后重试对话。

**若 helper 已退出**（`cognitive dsh status` 无 ACTIVE 进程），在同一 runtime 上重新启动 web：

```bash
RUNTIME=/home/hal9001/p8t13-owner-ops/runtime
BINDIR=/path/to/cognitive

fuser -k 3080/tcp 2>/dev/null || true
$BINDIR/cognitive dsh web --runtime-root "$RUNTIME" \
  --host 127.0.0.1 --port 3080 --no-open

$BINDIR/cognitive dsh status --runtime-root "$RUNTIME"
# 期望 state: ACTIVE, process_alive: true
```

## 预防与运维约定

1. **运维顺序**：`daemon stop/start` 之后，仍在运行的 `dsh web` helper 应自动刷新 Path B bearer。Control Plane 绑定 alone 仍不够解释面板「API key 无效」。
2. **勿在 dsh `.env` 填 LongCat key**：Path B 设计为 Secret Store + daemon 代理；第二份 key 不解决 bearer 失效。
3. **勿用 `local-bootstrap.secret` 冒充 Path B bearer**：bootstrap 用于签发会话；`/provider/v1/dsh` 需要 **management session** token（形如 `sess-...`），不是 bootstrap 文件内容本身。
4. **doctor 中 provider blocked**（如 `selected_model_digest_mismatch`）影响 Pi/`cognitive init` 路径，**不**阻止 dsh Path B；dsh 仍依赖上述 bearer。

## 相关实现引用

| 组件 | 路径 |
|------|------|
| dsh web helper | `personal/packages/dsh-akp-adapter/scripts/dsh-real-process.mjs`（`runWebPathB`，签发 token 写 `.credentials.yaml`；探测 stale 后重签发） |
| Path B 凭据/设置 | `personal/packages/dsh-akp-adapter/scripts/dsh-web-preflight.mjs`（`classifyPathBManagementProbe`，`pathBWebCredentialsYaml`，`apiKeyEnv: DAEMON_BEARER`） |
| daemon Path B 路由 | `personal/apps/kernel-server/src/personal/server.rs`（`POST /provider/v1/dsh/chat/completions`） |
| 会话鉴权错误码 | `personal/apps/kernel-server/src/personal/auth.rs`（`LOCAL_SESSION_UNAUTHORIZED`） |
| 操作说明 | `personal/handbook/en/user/cli-basics.md`（`cognitive dsh web` / `apply`） |

## 非本 bug 的相似现象

- LongCat `secret_ref did not resolve` → 账户 **degraded**；需在 Control Plane 重新 `provider key set`。
- 绑定模型与账户 endpoint 不匹配（如 grok on DeepSeek host）→ Path B `PERSONAL_PROVIDER_BINDING_MISMATCH`。
- 未执行 `cognitive dsh configure` / 缺少 `apps/web/dist` → `dsh web` 启动失败，与凭据 stale 不同。

## 证据摘要（2026-08-26，linux-002）

- LongCat 账户 `acct-01a03174-...`：**active**，目录含 `LongCat-2.0`。
- dsh 绑定：`agent://personal/dsh` → `LongCat-2.0`，revision 17。
- daemon 重启后：旧 bearer → `401 LOCAL_SESSION_UNAUTHORIZED`；dsh **CRASHED**。
- `cognitive dsh web` 重启后：新 `sess-...` 写入 `.credentials.yaml`；Path B 返回 `model: LongCat-2.0` 正常 completion。
- 产品修复：`P8-T16` helper 在仍运行时对 401 重签发 bearer（linux-002 自动刷新验证见任务报告）。
