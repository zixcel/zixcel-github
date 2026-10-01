# zixcel-github

GitHub APIの型付きラッパーです。GitHubの観測正規化と、安全な操作集合の検証、
opaqueなconnection参照を使ったbackend呼び出しだけを担当します。呼出側固有のpackage、
runtime grant、ワークフロー、ユーザー認可の概念は公開契約にも実装にも含みません。

```bash
cargo run --offline -- doctor
cargo run --offline -- capabilities
cargo run --offline -- validate examples/config.toml
cargo run --offline -- plan examples/config.toml
cargo run --offline -- check-request examples/push-request.json
```

認証情報は受け付けず、opaqueな`connection_ref`だけを扱います。OAuth、
secret解決、HTTP/Git transportはCrowsi/Zixcel worker境界へ委譲します。Libraryの
`execute_api_request`は、検証済みの閉じたAPI requestだけを注入された
`GitHubBackend`へ渡します。呼出主体の権限検証は上位adapterの責務です。

このリポジトリは他のローカルリポジトリへパス依存しません。

Libraryからは `parse_config` で閉じた1 MiB以下のTOMLを検証し、`build_plan` で
Repository順序に依存しないPlanを生成できます。HTTPやSecret resolverをリンクせず、
操作backendもtrait越しに限定します。公開先は既存の `zixcel-private` レジストリです。今回の変更をレジストリへ発行していません。

書込操作はprivate repository作成、source snapshot反映、Issue/PR作成、Workflow dispatch、private repository削除です。

## 責務境界

- Zixcel: GitHub API request/receipt、操作入力の検証、backend port。
- Crowsi: connection解決、credential、transport、署名済み呼出境界。
- 呼出adapter: ユーザーやworkflow固有の認可をAPI requestへ変換。

このcrateは上位adapterの種類や製品を識別しません。

## 品質ゲート

以下はGitHubへ接続せず、このcrateだけで完結します。

```bash
# WONDERLAND_ROOT is the workspace checkout root.
"$WONDERLAND_ROOT/bin/verify-repositories" --rust --tier standard
```

## ローカル設定と反映準備

`allowed_actions` はローカルの書込上限です。省略または空配列なら読取操作だけを許可します。`ConnectorConfig::authorize_request` は connection、organization、repository を照合します。上位の署名済みgrantやGitHub自身の権限は別途必要です。

```sh
cargo run -- check-configured-request examples/write-config.toml examples/create-request.json
```

`execute_configured_request` は設定の検査をしてから注入backendを呼びます。実運用のCrowsi transportでもverify/execute両方に同じ設定検査を適用します。既存の `execute_api_request` は型検証だけの低水準portであり、呼出側が認可とローカル上限を適用する責務を持ちます。

作成許可 `create-private-repository`、反映許可 `push-repository-snapshot`、削除許可 `delete-repository` は独立しています。削除は `github-repository-administration/delete-resource`、期待repository ID、バックアップ宣言のSHA-256を要求します。

ローカル検査ツール `zixcel-repository-security prepare-snapshot` で検査済みの完全source artifactを作成します。artifact参照は `sha256:<64hex>`、digest欄はprefixなしの64hexです。Crowsi artifact storeへ `<64hex>.json` として置きます。反映先の現在commit SHA文字列をSHA-256化した値を `expected_remote: {state: exact, commit_sha256: ...}` に指定し、変更後は再観測して新しい期待値で実行します。force更新は行いません。

作成APIは個人とorganizationを判別し、organizationなら `/orgs/{owner}/repos` を使います。Git Data APIが空repositoryを扱えないため作成時に初期commitを作り、そのheadを観測してから完全snapshotを反映します。作成と反映は別の署名済み操作です。ローカルGitのcommit履歴はこのsource snapshot操作では転送しません。履歴を保持するpushはcredentialを扱うCrowsi Git transportと履歴検査が別途必要です。

削除用バックアップ宣言は次の形です。宣言のdigestとtargetは検証しますが、宣言だけでは実バックアップの存在・完全性を証明しません。削除権限を付与する運用者はGit bundleとLFS・Issue・設定等の必要なバックアップを別途検証し保管してください。

```json
{"schema":"zixcel://github/deletion-backup/v1","repository_id":"12345","owner":"example-org","repository":"example-api","bundle_digest_sha256":"sha256:<64hex>"}
```

今回の検証はローカルfixtureです。実接続・権限付与・remote作成・push・削除の受入試験は未実施です。

## License / ライセンス

The current distribution uses Apache-2.0; see LICENSE and NOTICE. Earlier permissions and third-party terms remain in effect. Private registration, credentials and runtime state are excluded.

現在の配布版はApache-2.0です。LICENSEとNOTICEを参照してください。従前の許諾・第三者条件は保持します。非公開登録データ・認証情報・実行時状態は配布対象外です。
