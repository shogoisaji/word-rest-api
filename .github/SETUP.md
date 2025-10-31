# GitHub Actions デプロイセットアップガイド

このガイドでは、GitHub ActionsからGoogle Cloud Runへの自動デプロイを設定する方法を説明します。

## 前提条件

1. Google Cloudプロジェクトが作成されている
2. Neon PostgreSQLデータベースが作成されている
3. GitHubリポジトリが作成されている

## セットアップ手順

### 1. Google Cloud プロジェクトの準備

```bash
# プロジェクトIDを設定
export PROJECT_ID="your-project-id"
gcloud config set project $PROJECT_ID

# 必要なAPIを有効化
gcloud services enable run.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable secretmanager.googleapis.com
```

### 2. Artifact Registryリポジトリの作成

```bash
# Dockerリポジトリを作成
gcloud artifacts repositories create word-rest-api \
  --repository-format=docker \
  --location=asia-northeast1 \
  --description="Word REST API Docker images"
```

### 3. Secret Managerにデータベース接続文字列を保存

```bash
# Neonの接続文字列を保存
echo -n "postgresql://username:password@host:port/database?sslmode=require" | \
  gcloud secrets create database-url --data-file=-

# プロジェクト番号を取得
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")

# Cloud Runサービスアカウントにシークレットアクセスを付与
gcloud secrets add-iam-policy-binding database-url \
  --member="serviceAccount:${PROJECT_NUMBER}-compute@developer.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```

### 4. サービスアカウントの作成

GitHub Actions用のサービスアカウントを作成します：

```bash
# サービスアカウントを作成
gcloud iam service-accounts create github-actions \
  --display-name="GitHub Actions Deployment"

# 必要な権限を付与
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/run.admin"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/artifactregistry.writer"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser"

# サービスアカウントキーを作成
gcloud iam service-accounts keys create github-actions-key.json \
  --iam-account=github-actions@${PROJECT_ID}.iam.gserviceaccount.com
```

### 5. GitHubシークレットの設定

GitHubリポジトリの Settings > Secrets and variables > Actions に移動し、以下のシークレットを追加します：

#### `GCP_PROJECT_ID`
```
your-project-id
```

#### `GCP_SA_KEY`
```bash
# github-actions-key.jsonの内容をコピー
cat github-actions-key.json
```

JSONファイルの全内容をコピーしてGitHubシークレットに貼り付けます。

**重要**: キーファイルを作成したら、安全に保管し、バージョン管理にコミットしないでください：

```bash
# キーファイルを削除（GitHubに保存済み）
rm github-actions-key.json
```

### 6. デプロイのテスト

コードをmainブランチにプッシュすると、自動的にデプロイが開始されます：

```bash
git add .
git commit -m "Setup GitHub Actions deployment"
git push origin main
```

GitHubの Actions タブでデプロイの進行状況を確認できます。

### 7. デプロイの確認

デプロイが完了したら、サービスURLを取得：

```bash
gcloud run services describe word-rest-api \
  --region=asia-northeast1 \
  --format='value(status.url)'
```

ヘルスチェック：

```bash
curl https://your-service-url.run.app/health
```

## トラブルシューティング

### 権限エラー

サービスアカウントに必要な権限があることを確認：

```bash
gcloud projects get-iam-policy $PROJECT_ID \
  --flatten="bindings[].members" \
  --filter="bindings.members:serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com"
```

### Artifact Registryエラー

リポジトリが存在することを確認：

```bash
gcloud artifacts repositories list --location=asia-northeast1
```

### Secret Managerエラー

シークレットが存在し、アクセス可能であることを確認：

```bash
gcloud secrets list
gcloud secrets get-iam-policy database-url
```

## 手動デプロイ

GitHub Actionsを使わずに手動でデプロイする場合：

```bash
# ワークフローを手動でトリガー
# GitHubの Actions タブ > Deploy to Cloud Run > Run workflow
```

または、ローカルから：

```bash
gcloud run deploy word-rest-api \
  --source . \
  --region asia-northeast1 \
  --platform managed \
  --allow-unauthenticated \
  --set-secrets="DATABASE_URL=database-url:latest" \
  --set-env-vars="ENV=production,RUST_LOG=info"
```

## セキュリティのベストプラクティス

1. ✅ サービスアカウントキーを安全に保管
2. ✅ 最小権限の原則に従う
3. ✅ 定期的にキーをローテーション
4. ✅ シークレットをバージョン管理にコミットしない
5. ✅ GitHub Actionsログでシークレットが公開されていないか確認

## 次のステップ

- [ ] 本番環境用のカスタムドメインを設定
- [ ] Cloud Armorでレート制限を設定
- [ ] Cloud Monitoringでアラートを設定
- [ ] 自動バックアップを設定
- [ ] ステージング環境を作成

## 参考リンク

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Google Cloud Run Documentation](https://cloud.google.com/run/docs)
- [Artifact Registry Documentation](https://cloud.google.com/artifact-registry/docs)
- [Secret Manager Documentation](https://cloud.google.com/secret-manager/docs)
