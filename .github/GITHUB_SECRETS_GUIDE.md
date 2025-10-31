# GitHubシークレット設定ガイド

このガイドでは、GitHub ActionsでGoogle Cloudにデプロイするために必要なシークレットの設定方法を詳しく説明します。

## 📋 必要なシークレット

以下の2つのシークレットが必要です：

1. **GCP_PROJECT_ID** - Google CloudプロジェクトID
2. **GCP_SA_KEY** - サービスアカウントキー（JSON）

---

## 🔑 1. サービスアカウントキーの取得

### オプションA: セットアップスクリプトを使用（推奨）

```bash
# プロジェクトのルートディレクトリで実行
./scripts/setup-github-actions.sh
```

このスクリプトが以下を自動的に実行します：
- サービスアカウントの作成
- 必要な権限の付与
- キーファイル `github-actions-key.json` の生成

### オプションB: Google Cloud Consoleから手動で作成

1. [Google Cloud Console](https://console.cloud.google.com/) にアクセス
2. プロジェクトを選択
3. **IAM & Admin** > **Service Accounts** に移動
4. **CREATE SERVICE ACCOUNT** をクリック
5. サービスアカウント名を入力（例: `github-actions`）
6. 以下のロールを付与：
   - Cloud Run Admin
   - Artifact Registry Writer
   - Service Account User
7. **DONE** をクリック
8. 作成したサービスアカウントをクリック
9. **KEYS** タブに移動
10. **ADD KEY** > **Create new key** をクリック
11. **JSON** を選択して **CREATE** をクリック
12. JSONファイルがダウンロードされます

### オプションC: gcloudコマンドで作成

```bash
# 1. プロジェクトIDを設定
export PROJECT_ID="your-project-id"
gcloud config set project $PROJECT_ID

# 2. サービスアカウントを作成
gcloud iam service-accounts create github-actions \
  --display-name="GitHub Actions Deployment"

# 3. 必要な権限を付与
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/run.admin"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/artifactregistry.writer"

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser"

# 4. キーファイルを作成
gcloud iam service-accounts keys create github-actions-key.json \
  --iam-account=github-actions@${PROJECT_ID}.iam.gserviceaccount.com

echo "✅ キーファイルが作成されました: github-actions-key.json"
```

---

## 📄 2. キーファイルの内容を確認

### キーファイルの構造

`github-actions-key.json` は以下のような構造のJSONファイルです：

```json
{
  "type": "service_account",
  "project_id": "your-project-id",
  "private_key_id": "abc123...",
  "private_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n",
  "client_email": "github-actions@your-project-id.iam.gserviceaccount.com",
  "client_id": "123456789...",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/..."
}
```

### キーファイルの内容を表示

```bash
# ファイルの内容を表示
cat github-actions-key.json

# macOSの場合、クリップボードにコピー
cat github-actions-key.json | pbcopy

# Linuxの場合（xclipがインストールされている場合）
cat github-actions-key.json | xclip -selection clipboard

# Windowsの場合（PowerShell）
Get-Content github-actions-key.json | Set-Clipboard
```

---

## 🔐 3. GitHubシークレットの設定

### ステップ1: GitHubリポジトリにアクセス

1. ブラウザでGitHubリポジトリを開く
2. リポジトリのトップページに移動

### ステップ2: Settings に移動

1. リポジトリの上部メニューから **Settings** タブをクリック
2. 左サイドバーの **Secrets and variables** をクリック
3. **Actions** をクリック

### ステップ3: GCP_PROJECT_ID を追加

1. **New repository secret** ボタンをクリック
2. 以下を入力：
   - **Name**: `GCP_PROJECT_ID`
   - **Secret**: Google CloudプロジェクトID（例: `my-project-12345`）
3. **Add secret** ボタンをクリック

**プロジェクトIDの確認方法：**
```bash
gcloud config get-value project
```

### ステップ4: GCP_SA_KEY を追加

1. 再度 **New repository secret** ボタンをクリック
2. 以下を入力：
   - **Name**: `GCP_SA_KEY`
   - **Secret**: `github-actions-key.json` の**全内容**を貼り付け
     - ファイルの内容をそのままコピー＆ペースト
     - 改行や空白も含めて全て
     - `{` から `}` まで全て
3. **Add secret** ボタンをクリック

**重要なポイント：**
- ✅ JSONファイルの**全内容**をコピー
- ✅ 最初の `{` から最後の `}` まで全て含める
- ✅ 改行や空白もそのまま保持
- ❌ ファイルパスやファイル名は含めない
- ❌ 余計な文字を追加しない

### ステップ5: 確認

シークレットが正しく追加されると、以下のように表示されます：

```
Repository secrets

GCP_PROJECT_ID
Updated X minutes ago

GCP_SA_KEY
Updated X minutes ago
```

---

## ✅ 4. 設定の確認

### テストデプロイを実行

```bash
# コードをコミット
git add .
git commit -m "Setup GitHub Actions deployment"

# mainブランチにプッシュ（本番環境にデプロイ）
git push origin main
```

### GitHub Actionsの確認

1. GitHubリポジトリの **Actions** タブをクリック
2. 最新のワークフロー実行を確認
3. 各ステップが緑色のチェックマークになっていることを確認

### デプロイが成功した場合

```
✅ Checkout code
✅ Authenticate to Google Cloud
✅ Set up Cloud SDK
✅ Configure Docker for Artifact Registry
✅ Build Docker image
✅ Push Docker image to Artifact Registry
✅ Deploy to Cloud Run
✅ Get Service URL
✅ Health Check
✅ Test API Endpoints
```

### エラーが発生した場合

**認証エラー:**
```
Error: google-github-actions/auth failed with: failed to generate Google Cloud access token
```

**解決方法:**
- `GCP_SA_KEY` の内容が正しいか確認
- JSONファイルの全内容がコピーされているか確認
- 余計な文字が含まれていないか確認

**権限エラー:**
```
Error: Permission denied on resource
```

**解決方法:**
- サービスアカウントに必要な権限が付与されているか確認
- 以下のコマンドで権限を確認：
```bash
gcloud projects get-iam-policy $PROJECT_ID \
  --flatten="bindings[].members" \
  --filter="bindings.members:serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com"
```

---

## 🔒 5. セキュリティのベストプラクティス

### ✅ やるべきこと

1. **キーファイルを安全に保管**
   ```bash
   # GitHubにシークレットを追加したら、ローカルのキーファイルを削除
   rm github-actions-key.json
   ```

2. **.gitignoreに追加**
   ```bash
   # .gitignoreに以下を追加（既に含まれています）
   *.json
   github-actions-key.json
   ```

3. **最小権限の原則**
   - 必要な権限のみを付与
   - 定期的に権限を見直す

4. **キーのローテーション**
   ```bash
   # 古いキーを削除
   gcloud iam service-accounts keys list \
     --iam-account=github-actions@${PROJECT_ID}.iam.gserviceaccount.com
   
   gcloud iam service-accounts keys delete KEY_ID \
     --iam-account=github-actions@${PROJECT_ID}.iam.gserviceaccount.com
   
   # 新しいキーを作成
   gcloud iam service-accounts keys create github-actions-key-new.json \
     --iam-account=github-actions@${PROJECT_ID}.iam.gserviceaccount.com
   ```

### ❌ やってはいけないこと

1. **キーファイルをGitにコミット**
   ```bash
   # 絶対にやらないこと！
   git add github-actions-key.json  # ❌
   ```

2. **キーファイルを公開リポジトリに含める**
   - 公開リポジトリでは特に注意

3. **キーファイルをSlackやメールで共有**
   - 安全な方法で共有する

4. **キーファイルをログに出力**
   - GitHub Actionsのログにシークレットが表示されないように注意

---

## 🆘 トラブルシューティング

### Q: キーファイルが見つからない

**A:** キーファイルの場所を確認：
```bash
# 現在のディレクトリを確認
pwd

# キーファイルを検索
find . -name "github-actions-key.json"

# ダウンロードフォルダを確認（ブラウザからダウンロードした場合）
ls ~/Downloads/github-actions-key*.json
```

### Q: シークレットが正しく設定されているか確認したい

**A:** GitHub Actionsのログで確認：
```yaml
# ワークフローに以下を追加（デバッグ用）
- name: Debug
  run: |
    echo "Project ID: ${{ secrets.GCP_PROJECT_ID }}"
    echo "SA Key length: ${#GCP_SA_KEY}"
  env:
    GCP_SA_KEY: ${{ secrets.GCP_SA_KEY }}
```

**注意:** 実際のキーの内容は絶対にログに出力しないでください！

### Q: 複数の環境（本番、ステージング）で異なるキーを使いたい

**A:** Environment secretsを使用：
1. Settings > Environments で環境を作成
2. 各環境ごとにシークレットを設定
3. ワークフローで環境を指定：
```yaml
jobs:
  deploy:
    environment: production  # または staging
```

---

## 📚 参考リンク

- [GitHub Actions Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Google Cloud Service Accounts](https://cloud.google.com/iam/docs/service-accounts)
- [google-github-actions/auth](https://github.com/google-github-actions/auth)
- [Cloud Run Documentation](https://cloud.google.com/run/docs)

---

## 📝 チェックリスト

デプロイ前に以下を確認してください：

- [ ] Google Cloudプロジェクトが作成されている
- [ ] 必要なAPIが有効化されている
- [ ] サービスアカウントが作成されている
- [ ] サービスアカウントに必要な権限が付与されている
- [ ] キーファイルが作成されている
- [ ] GitHubシークレット `GCP_PROJECT_ID` が設定されている
- [ ] GitHubシークレット `GCP_SA_KEY` が設定されている
- [ ] ローカルのキーファイルが削除されている
- [ ] .gitignoreにキーファイルが含まれている
- [ ] データベース接続文字列がSecret Managerに保存されている

すべてチェックが完了したら、デプロイの準備完了です！🚀
