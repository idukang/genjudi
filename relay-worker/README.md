# 根据地 · 自建加密中转（Cloudflare Worker）

跨网络（不同 Wi-Fi、开了代理、两端不在同一局域网）同步账号与登录态时，需要一个
「只暂存密文」的中转点。**根据地不内置任何官方中转服务器，也不会自动连接任何第三方
地址**；你用自己的 Cloudflare 账号部署这个 Worker，再把它填进 App。

## 它做什么 / 不做什么

- App 每次同步都会在本地随机生成一次性 256-bit 密钥，用 AES-256-GCM 端到端加密整包数据。
  密钥只存在于二维码 / 同步串里（`genjudi-sync://relay/...#<密钥>.<取件令牌>`），**从不上传**。
- Worker 只按 ID 暂存一段密文（≤ 2 MB），持有取件令牌的设备才能取一次；3 分钟或取满 3 次
  后立即销毁；取件令牌 5 次错误也立即销毁。Worker 无法解密任何内容。
- Worker 不收集、不上报、不记录任何用户数据，没有分析统计、没有第三方调用（源码只有
  `src/index.js` 一个文件，可自行逐行审计）。

## 部署（全程免费额度内）

```bash
npm install -g wrangler
wrangler login
cd relay-worker
wrangler deploy
```

`wrangler deploy` 会输出一个 `https://genjudi-sync-relay.<你的子域>.workers.dev` 地址，
把它填进 App 的「同步 → 使用自建加密中转通道」。

如果用自己的域名，先在 Cloudflare 里把域名接入，再取消 `wrangler.toml` 里 `routes`
的注释并改成自己的域名，然后重新 `wrangler deploy`。

## 费用

Cloudflare Workers 免费额度：每天 10 万次请求，Durable Objects（SQLite 后端）免费额度
足够个人同步使用。密文暂存 3 分钟即销毁，不会长期占用存储。

## 接口（便于自建其它实现）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/v1/drop` | 上传密文：`{ id, nonce, ciphertext, tokenHash, deleteTokenHash, ttlSecs, maxDownloads }` |
| `GET` | `/v1/drop/{id}` | 取件，请求头 `x-relay-token: <取件令牌>` |
| `DELETE` | `/v1/drop/{id}` | 提前销毁，请求头 `x-relay-token: <销毁令牌>` |
| `GET` | `/health` | 健康检查 |

你完全可以用任何支持 HTTPS 的服务端实现这三个接口（例如自己的 VPS + Nginx），App 只要求
填入的地址是 HTTPS、且不带查询参数与认证凭据。
