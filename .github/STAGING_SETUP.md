# ステージング環境セットアップガイド

このガイドでは、本番環境とは別にステージング環境を構築する方法を説明します。

## 🎯 ステージング環境とは？

ステージング環境は、本番環境にデプロイする前にコードをテストするための環境です。

### メリット

- ✅ 本番データを壊す心配なくテストできる
- ✅ PRのレビュー時に実際の動作を確認できる
- ✅ 開発中の機能を安全に試せる
- ✅ 本番環境と同じ構成でテストできる

## 📊 環境構成の選択肢

### オプション1: 本番と同じデータベースを使う（簡単）

**推奨度:** ⭐⭐☆☆☆

**メリット:**
- セットアップが簡単
- 追加コストなし
- 小規模プロジェクトに最適

**デメリット:**
- 本番データとステージングデータが混在
- テストデータが本番に影響する可能性

**設定:**
```bash
# セットアップスクリプト実行時にステージング用DBをスキップ
./scripts/setup-github-actions.sh
# "ステージング環境用のデータベース接続文字列" でEnterを押す
```

### オプション2: Neonブランチを使う（推奨）

**推奨度:** ⭐⭐⭐⭐⭐

**メリット:**
- 本番データと完全分離
- 同じプロジェクト内で管理
- ブランチ間でデータをコピー可能
- Neonの無料枠内で利用可能

**デメリット:**
- 若干のセットアップが必要

**設定方法は下記参照**

### オプション3: 完全に別のNeonプロジェクトを作る

**推奨度:** ⭐⭐⭐☆☆

**メリット:**
- 完全に独立した環境
- 本番環境に一切影響しない
- 異なるリージョンも選択可能

**デメリット:**
- 管理が複雑
- 無料枠を2つ使う

## 🔧 推奨: Neonブランチでステージング環境を作成

### ステップ1: Neonでブランチを作成

1. [Neon Console](https://console.neon.tech) にログイン
2. 本番環境で使用しているプロジェクトを選択
3. 左メニューの **Branches** をクリック
4. **Create Branch** ボタンをクリック

5. ブランチ設定を入力:
   ```
   Branch name: staging
   Parent branch: main (本番環境のブランチ)
   ```

6. **Create** をクリック

7. 作成された `staging` ブランチをクリック

8. **Connection Details** セクションで接続文字列をコピー
   - **Pooled connection** を選択（推奨）
   - 接続文字列の形式:
     ```
     postgresql://username:password@ep-staging-xxxxx.region.aws.neon.tech/neondb?sslmode=require
     ```

### ステップ2: Google Secret Managerに保存

```bash
# ステージング用の接続文字列を保存
echo -n "postgresql://username:password@ep-staging-xxxxx.region.aws.neon.tech/neondb?sslmode=require" | \
  gcloud secrets create database-url-staging --data-file=-

# プロジェクト番号を取得
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")

# Cloud Runサービスアカウントに権限を付与
gcloud secrets add-iam-policy-binding database-url-staging \
  --member="serviceAccount:${PROJECT_NUMBER}-compute@developer.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```

または、セットアップスクリプトを使用:

```bash
./scripts/setup-github-actions.sh
```

スクリプト実行中に、ステージング用の接続文字列を入力します。

### ステップ3: 確認

```bash
# シークレットが作成されたことを確認
gcloud secrets list

# 出力例:
# NAME                    CREATED              REPLICATION_POLICY  LOCATIONS
# database-url            2024-01-15T10:00:00  automatic           -
# database-url-staging    2024-01-15T10:05:00  automatic           -
```

## 🚀 デプロイ

### 本番環境にデプロイ

```bash
git push origin main
```

- サービス名: `word-rest-api`
- データベース: 本番用（`database-url`）
- URL: `https://word-rest-api-xxxxx.run.app`

### ステージング環境にデプロイ

```bash
git push origin develop
```

- サービス名: `word-rest-api-staging`
- データベース: ステージング用（`database-url-staging`）
- URL: `https://word-rest-api-staging-xxxxx.run.app`

## 📋 環境の比較

| 項目 | 本番環境 | ステージング環境 |
|------|---------|----------------|
| ブランチ | `main` / `master` | `develop` / `staging` |
| サービス名 | `word-rest-api` | `word-rest-api-staging` |
| データベース | `database-url` | `database-url-staging` |
| 最大インスタンス | 10 | 5 |
| ログレベル | `info` | `debug` |
| 環境変数 | `ENV=production` | `ENV=staging` |

## 🔄 ワークフロー例

### 開発フロー

1. **機能開発**
   ```bash
   git checkout -b feature/new-feature
   # コードを書く
   git commit -m "Add new feature"
   ```

2. **ステージングにデプロイ**
   ```bash
   git push origin develop
   # GitHub Actionsが自動的にステージング環境にデプロイ
   ```

3. **ステージング環境でテスト**
   ```bash
   # PRのコメントにステージングURLが表示される
   curl https://word-rest-api-staging-xxxxx.run.app/health
   ```

4. **本番環境にデプロイ**
   ```bash
   # PRをmainにマージ
   git checkout main
   git merge develop
   git push origin main
   # GitHub Actionsが自動的に本番環境にデプロイ
   ```

## 🧪 ステージング環境のテスト

### 自動テスト

ステージング環境へのデプロイ時に、自動的に以下のテストが実行されます：

```yaml
# .github/workflows/deploy-staging.yml
- name: Run Integration Tests
  run: |
    # ヘルスチェック
    curl -f $SERVICE_URL/health
    
    # API エンドポイントのテスト
    curl -f $SERVICE_URL/api/vocabulary
    
    # データ作成テスト
    curl -X POST $SERVICE_URL/api/vocabulary \
      -H "Content-Type: application/json" \
      -d '{"en_word":"test","ja_word":"テスト",...}'
```

### 手動テスト

```bash
# ステージング環境のURLを取得
STAGING_URL="https://word-rest-api-staging-xxxxx.run.app"

# ヘルスチェック
curl $STAGING_URL/health

# 語彙一覧を取得
curl $STAGING_URL/api/vocabulary

# 語彙を作成
curl -X POST $STAGING_URL/api/vocabulary \
  -H "Content-Type: application/json" \
  -d '{
    "en_word": "test",
    "ja_word": "テスト",
    "en_example": "This is a test.",
    "ja_example": "これはテストです。"
  }'
```

## 🔄 データの同期

### 本番データをステージングにコピー

Neonブランチを使用している場合、本番データをステージングにコピーできます：

1. Neon Consoleで `staging` ブランチを選択
2. **Reset from parent** をクリック
3. 確認して実行

**注意:** ステージング環境のデータは上書きされます。

### コマンドラインでコピー

```bash
# 本番データをエクスポート
pg_dump "$(gcloud secrets versions access latest --secret=database-url)" > production-backup.sql

# ステージングにインポート
psql "$(gcloud secrets versions access latest --secret=database-url-staging)" < production-backup.sql
```

## 🗑️ ステージング環境の削除

### Cloud Runサービスを削除

```bash
gcloud run services delete word-rest-api-staging --region=asia-northeast1
```

### Neonブランチを削除

1. Neon Consoleで `staging` ブランチを選択
2. **Settings** タブをクリック
3. **Delete branch** をクリック

### Secret Managerのシークレットを削除

```bash
gcloud secrets delete database-url-staging
```

## 💡 ベストプラクティス

### 1. 定期的にステージングをリセット

```bash
# 週に1回、本番データでステージングをリフレッシュ
# Neon Consoleで "Reset from parent" を実行
```

### 2. ステージング環境でのみテストデータを使用

```bash
# テストデータには明確なプレフィックスを付ける
{
  "en_word": "TEST_apple",
  "ja_word": "TEST_りんご"
}
```

### 3. ステージング環境のログレベルを上げる

```yaml
# .github/workflows/deploy-staging.yml
--set-env-vars="ENV=staging,RUST_LOG=debug"
```

### 4. PRにステージングURLを自動コメント

```yaml
# .github/workflows/deploy-staging.yml
- name: Comment PR with Staging URL
  if: github.event_name == 'pull_request'
  uses: actions/github-script@v7
  with:
    script: |
      github.rest.issues.createComment({
        issue_number: context.issue.number,
        owner: context.repo.owner,
        repo: context.repo.repo,
        body: `🚀 Staging deployment successful!\n\n**Service URL:** ${{ steps.get-url.outputs.url }}`
      })
```

## 🆘 トラブルシューティング

### Q: ステージング環境が本番データベースに接続している

**A:** Secret Managerのシークレットを確認:

```bash
# ステージング用のシークレットが存在するか確認
gcloud secrets list | grep staging

# シークレットの値を確認（最初の数文字のみ表示）
gcloud secrets versions access latest --secret=database-url-staging | head -c 50
```

### Q: Neonブランチが作成できない

**A:** Neonの無料プランでは、ブランチ数に制限があります。不要なブランチを削除してください。

### Q: ステージング環境のデプロイが失敗する

**A:** GitHub Actionsのログを確認:

```bash
# GitHubの Actions タブで最新のワークフロー実行を確認
# エラーメッセージを確認して対処
```

## 📚 参考リンク

- [Neon Branching Documentation](https://neon.tech/docs/introduction/branching)
- [GitHub Actions Environments](https://docs.github.com/en/actions/deployment/targeting-different-environments)
- [Cloud Run Multiple Environments](https://cloud.google.com/run/docs/multiple-environments)

---

ステージング環境を活用して、安全に開発を進めましょう！🚀
