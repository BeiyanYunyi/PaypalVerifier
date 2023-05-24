# PayPal Verifier

一个全栈项目，通过使用 PayPal 支付的方式，来确保用户是唯一且真实的。等待接手。

## 服务端开发

```bash
cd server
cargo run
```

## 客户端开发

```bash
cd client
pnpm i
pnpm dev
```

## 生产构建

```bash
cd client
pnpm build
cd ../server
cargo build -r
```

此后，将 `server/target/release/webserver` 复制至自己希望的目录下，并配置好环境变量。服务端会监听 `http://localhost:8080` 暂时懒得加上环境变量配置，所以需要修改监听端口或地址的话请直接修改[此处](server/src/main.rs#L71)并重新构建。值得注意的是，服务端会将客户端的静态文件打包进二进制文件中，因此，不需要额外部署客户端。

## 技术栈

- [Actix](https://actix.rs/)
- [Solid.js](https://www.solidjs.com/)

## 环境变量

请参考 `server/env-example` 文件，如果希望采用 `.env` 的方式配置，则请将其重命名为 `.env`。其中 `RUST_LOG` 项选填，其它项必填。

- `RUST_LOG`：日志等级，可选项：`trace`、`debug`、`info`、`warn`、`error`。
- `CLIENT_ID`：PayPal Client ID。
- `SECRET`：PayPal Secret。
- `PAYPAL_ENV`：PayPal 环境，可选项：`sandbox`、`live`。前者为测试环境，后者为生产环境。
- `DATABASE_URL`：目前可用的数据库有：`postgres`、`sqlite`。前者的格式为 `postgres://<user>:<password>@<host>:<port>/<database>`，后者的格式为 `sqlite://<path>`。

## 结构

该项目在计划中将作为一个微服务而运行。计划中的结构如下：

1. 对注册用户：通过让用户支付 0.01 美元来确保用户的真实性，此后，向 PayPal API 请求前端付款完成后提交的 `order_id`，记录并验证 PayPal 返回的 `payer_id` 字段来确保用户的唯一性。这之后，对此前的支付进行退款处理，以保证注册过程是免费的。当 `payer_id` 不存在记录，或对应的 `used` 为 `false` 时，返回 `payer_id`，否则返回 409。（后端[已完成](server/src/routes/v1.rs#L35)，前端待完善交互）
2. 对服务提供方：提供一个接口，当以 `{"payer_id": "xxxxx"}` POST 请求该接口时，会将数据库内该 `payer_id` 对应行的 `used` 字段置为 `true`，并返回 200，否则返回 409。（未完成）

具体接入：以“接入 flarum 的用户系统”为例：需要一个 flarum 的插件，该插件在用户注册时，将用户定向到 1. 所述页面，前述页面返回 `payer_id` 给用户，用户在注册表单中填写之。此后，flarum 插件请求 2. 所述接口，以确保用户的真实性和唯一性。（未完成）

## 技术指标

Windows 系统，使用 SQLite 作为数据库，服务端使用生产模式构建，进行单次支付测试，全程内存占用小于 5MB。高并发测试未完成。

时间仓促，有大量未完成事项，甚至可能并未在此文档中列举。还请接手者多多包涵。
