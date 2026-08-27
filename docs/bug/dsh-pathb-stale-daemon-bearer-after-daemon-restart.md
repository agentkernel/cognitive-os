# dsh Path B「API key 无效」与 daemon 重启后 stale `DAEMON_BEARER`

- **跟踪 ID**: `DSH-PATHB-STALE-DAEMON-BEARER-01`
- **状态**: 已复现；产品缺陷保持 open；已有经验证的运维恢复，尚无产品代码修复
- **登记任务**: `P10-T01/D03`（文档正式登记，不授权实现）
- **环境**: `B01-Desktop-Linux-002`（`hal9001@192.168.123.160`），runtime `/home/hal9001/p8t13-owner-ops/runtime`
- **相关任务/能力**: P8-T10 / P8-T15，dsh Path B，`cognitive dsh web`
- **发现日期**: 2026-08-26
- **当前处置**: daemon 重启/替换后必须重启 `cognitive dsh web`；`cognitive dsh apply` 仅用于 daemon 未重启且 dsh runtime 已为 `ACTIVE` 时同步受支持的 binding/model overlay

## 症状

- Control Plane（`/ui/`）中 LongCat Provider 账户显示持久化状态 **active**，模型 `LongCat-2.0` 已绑定到 `agent://personal/dsh`。
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
| Control Plane「LongCat active」 | 持久化账户状态与 binding；**不证明**当前 Secret Store 可实时解析 |
| daemon 重启后 dsh「API key 无效」 | 旧 **`DAEMON_BEARER`** management session 不再被新 daemon 接受，Path B 入口返回 401 |

两者可以同时成立，但不能从 `active` 推断上游 LongCat 密钥当前可用。实时 SecretStore
解析只发生在 discovery/proxy 使用期间；锁定、变化或不可用的 store 仍是独立可能原因。
本 bug 的判别依据是 daemon 重启时间线、旧 session 的 401，以及新 daemon 将 dsh
runtime 投影为 `INACTIVE`。

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

### 3. dsh web 进程与凭据文件不会随 daemon 自动刷新

若仅重启 daemon 而 **不**重启 `cognitive dsh web`：

- `$DSH_HOME/.credentials.yaml` 仍保留旧 `sess-...`；
- 新 daemon 没有原 dsh runtime 的进程内登记，`cognitive dsh status` 报 `INACTIVE`；
- 3080 上旧 Cos/node OS 进程可能仍存活，但它持有旧凭据且不属于新 daemon 的活动 runtime；
- dsh UI 将上游 401 呈现为「API key 无效」，容易误判为 LongCat Provider 未配置。

此时 `cognitive dsh apply` 会因 runtime 非 `ACTIVE` 被拒绝，不能刷新 stale session。

### 4. 与 Control Plane 显示不一致的常见混淆点

| 事实 | 说明 |
|------|------|
| `provider.json` 仍指向 `deepseek` | `cognitive init` 遗留；dsh Path B **不读**此文件 |
| `selected-model.json` 为 `LongCat-2.0` | 来自 Control Plane 绑定；与 Path B 路由一致 |
| LongCat 账户 `active` | 仅持久化状态；最近一次 discovery 可能曾成功，但不证明当前 Secret Store 可解析 |
| daemon 重启后 dsh 报错且 status `INACTIVE` | 符合 stale loopback session；必须重启 dsh web |

## 复现条件

1. runtime 上 LongCat 账户与 dsh 绑定已正确配置；
2. `cognitive dsh web` 曾成功启动并写入 `.credentials.yaml`；
3. **在不重启 dsh web 的情况下**重启 daemon（例如部署新 `kernel-server`、换 UI 包、修 lock 冲突后 `daemon start`）；
4. 在 dsh 面板发起对话或打开 Models 校验。

## 诊断步骤

在 guest 上只检查脱敏状态和文件元数据（替换实际 runtime 路径）：

```bash
RUNTIME=/home/hal9001/p8t13-owner-ops/runtime
BINDIR=/home/hal9001/dshfix-53ea437f/bin   # 或当前 cognitive 二进制路径

# 1. Provider 侧（Control Plane）
$BINDIR/cognitive provider account list --runtime-root "$RUNTIME"
$BINDIR/cognitive agent binding list --runtime-root "$RUNTIME"

# 2. dsh 运行时
$BINDIR/cognitive dsh status --runtime-root "$RUNTIME"
# daemon 重启后的本 bug 关注 state: INACTIVE；apply 在此状态会被拒绝

# 3. Path B 文件只看存在性/权限/大小/修改时间；不要读取内容
SETTINGS="$RUNTIME/cognitiveos/dsh-web-home/settings.yaml"
CREDENTIALS="$RUNTIME/cognitiveos/dsh-web-home/.credentials.yaml"
if test -f "$SETTINGS"; then
  stat --format='%n mode=%a size=%s modified=%y' "$SETTINGS"
else
  echo settings_missing
fi
if test -f "$CREDENTIALS"; then
  stat --format='%n mode=%a size=%s modified=%y' "$CREDENTIALS"
else
  echo credentials_missing
fi
```

当前没有批准的、端到端 non-logging 的 direct-bearer probe 调用方，因此直接 bearer
探测记为 **not available**。不得从 `.credentials.yaml` 提取 token 到 shell 变量，
也不得把 `Authorization` bearer 放进 `curl` 或其他进程 argv。持久化账户/binding
存在、dsh 为 `INACTIVE` 且时间线紧随 daemon 重启时，直接执行下述 `dsh web` 重启，
再用 `cognitive dsh status` 与面板对话验证。若重启后仍失败，再把 SecretStore
锁定/变化/不可用作为独立原因检查；不要用持久化 `active` 状态排除它。

## 恢复步骤

**每次 daemon 重启或替换 `kernel-server` 后**，必须在同一 runtime 上重启 dsh web。
新 daemon 将 dsh runtime 投影为 `INACTIVE`，此状态不接受 `apply`：

```bash
RUNTIME=/home/hal9001/p8t13-owner-ops/runtime
BINDIR=/path/to/cognitive

# daemon 重启后：清理可能残留的旧 3080 进程并完整重启 web
fuser -k 3080/tcp 2>/dev/null || true
$BINDIR/cognitive dsh web --runtime-root "$RUNTIME" \
  --host 127.0.0.1 --port 3080 --no-open

# 确认
$BINDIR/cognitive dsh status --runtime-root "$RUNTIME"
# 期望 state: ACTIVE, process_alive: true
```

刷新浏览器 `http://127.0.0.1:3080/` 后重试对话。

`cognitive dsh apply` 只保留给**未发生 daemon 重启**、`cognitive dsh status` 已为
`ACTIVE` 的 runtime，用于同步受支持的 binding/model overlay；它可能按需重启 Cos
对话进程。它不是 daemon 重启后的 stale-session 恢复命令。

## 预防与运维约定

1. **运维顺序**：`daemon stop/start` → **必须重启** `cognitive dsh web`；不可用 `dsh apply` 代替，也不可假设 Control Plane binding alone 足够。`apply` 只服务仍为 `ACTIVE` 的未重启 runtime overlay 同步。
2. **勿在 dsh `.env` 填 LongCat key**：Path B 设计为 Secret Store + daemon 代理；第二份 key 不解决 bearer 失效。
3. **勿用 `local-bootstrap.secret` 冒充 Path B bearer**：bootstrap 用于签发会话；`/provider/v1/dsh` 需要 **management session** token（形如 `sess-...`），不是 bootstrap 文件内容本身。
4. **doctor 中 provider blocked**（如 `selected_model_digest_mismatch`）影响 Pi/`cognitive init` 路径，**不**阻止 dsh Path B；dsh 仍依赖上述 bearer 刷新。

## 相关实现引用

| 组件 | 路径 |
|------|------|
| dsh web helper | `personal/packages/dsh-akp-adapter/scripts/dsh-real-process.mjs`（`runWebPathB`，签发 token 写 `.credentials.yaml`） |
| Path B 凭据/设置 | `personal/packages/dsh-akp-adapter/scripts/dsh-web-preflight.mjs`（`pathBWebCredentialsYaml`，`apiKeyEnv: DAEMON_BEARER`） |
| daemon Path B 路由 | `personal/apps/kernel-server/src/personal/server.rs`（`POST /provider/v1/dsh/chat/completions`） |
| 会话鉴权错误码 | `personal/apps/kernel-server/src/personal/auth.rs`（`LOCAL_SESSION_UNAUTHORIZED`） |
| 操作说明 | `personal/handbook/en/user/cli-basics.md`（`cognitive dsh web` / `apply`） |

## 非本 bug 的相似现象

- live discovery/proxy 返回 LongCat `secret_ref did not resolve` → 当前 SecretStore 无法解析；持久化 `active` 不排除此原因，需解锁/恢复 store 或在 Control Plane 重新 `provider key set`。
- 绑定模型与账户 endpoint 不匹配（如 grok on DeepSeek host）→ Path B `PERSONAL_PROVIDER_BINDING_MISMATCH`。
- 未执行 `cognitive dsh configure` / 缺少 `apps/web/dist` → `dsh web` 启动失败，与凭据 stale 不同。

## 证据摘要（2026-08-26，linux-002）

- LongCat 账户 `acct-01a03174-...`：持久化状态 **active**，目录含 `LongCat-2.0`；该状态本身不证明当下 SecretStore 可解析。
- dsh 绑定：`agent://personal/dsh` → `LongCat-2.0`，revision 17。
- daemon 重启后：旧 bearer → `401 LOCAL_SESSION_UNAUTHORIZED`；新 daemon 投影 dsh 为 `INACTIVE`，`apply` 被拒绝。
- `cognitive dsh web` 重启后：新的 management session 写入凭据文件；Path B 返回 `model: LongCat-2.0` 正常 completion。证据不保留 session 值。
