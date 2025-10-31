#!/bin/bash

# GitHub Actions デプロイセットアップスクリプト
# このスクリプトは、GitHub ActionsからCloud Runへのデプロイに必要な
# Google Cloudリソースを自動的にセットアップします

set -e

echo "🚀 GitHub Actions デプロイセットアップを開始します"
echo ""

# プロジェクトIDの入力
read -p "Google Cloud プロジェクトID: " PROJECT_ID

if [ -z "$PROJECT_ID" ]; then
    echo "❌ プロジェクトIDが必要です"
    exit 1
fi

echo ""
echo "📋 プロジェクト: $PROJECT_ID"
echo ""

# プロジェクトを設定
gcloud config set project $PROJECT_ID

# 必要なAPIを有効化
echo "🔧 必要なAPIを有効化しています..."
gcloud services enable run.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable secretmanager.googleapis.com
echo "✅ APIの有効化が完了しました"
echo ""

# Artifact Registryリポジトリの作成
echo "📦 Artifact Registryリポジトリを作成しています..."
gcloud artifacts repositories create word-rest-api \
  --repository-format=docker \
  --location=asia-northeast1 \
  --description="Word REST API Docker images" \
  2>/dev/null || echo "ℹ️  リポジトリは既に存在します"
echo "✅ Artifact Registryリポジトリの準備が完了しました"
echo ""

# データベース接続文字列の入力
echo "🗄️  データベース設定"
read -p "Neon PostgreSQL接続文字列: " DATABASE_URL

if [ -z "$DATABASE_URL" ]; then
    echo "❌ データベース接続文字列が必要です"
    exit 1
fi

# Secret Managerにデータベース接続文字列を保存
echo "🔐 Secret Managerにデータベース接続文字列を保存しています..."
echo -n "$DATABASE_URL" | gcloud secrets create database-url --data-file=- 2>/dev/null || {
    echo "ℹ️  シークレットは既に存在します。更新しますか？ (y/n)"
    read -p "> " UPDATE_SECRET
    if [ "$UPDATE_SECRET" = "y" ]; then
        echo -n "$DATABASE_URL" | gcloud secrets versions add database-url --data-file=-
        echo "✅ シークレットを更新しました"
    fi
}
echo ""

# ステージング環境用のシークレット（オプション）
read -p "ステージング環境用のデータベース接続文字列（スキップする場合はEnter）: " DATABASE_URL_STAGING

if [ -n "$DATABASE_URL_STAGING" ]; then
    echo "🔐 ステージング環境用のシークレットを保存しています..."
    echo -n "$DATABASE_URL_STAGING" | gcloud secrets create database-url-staging --data-file=- 2>/dev/null || {
        echo -n "$DATABASE_URL_STAGING" | gcloud secrets versions add database-url-staging --data-file=-
    }
    echo "✅ ステージング環境用のシークレットを保存しました"
fi
echo ""

# プロジェクト番号を取得
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")

# Cloud Runサービスアカウントにシークレットアクセスを付与
echo "🔑 Cloud Runサービスアカウントに権限を付与しています..."
gcloud secrets add-iam-policy-binding database-url \
  --member="serviceAccount:${PROJECT_NUMBER}-compute@developer.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor" \
  --quiet

if [ -n "$DATABASE_URL_STAGING" ]; then
    gcloud secrets add-iam-policy-binding database-url-staging \
      --member="serviceAccount:${PROJECT_NUMBER}-compute@developer.gserviceaccount.com" \
      --role="roles/secretmanager.secretAccessor" \
      --quiet
fi
echo "✅ 権限の付与が完了しました"
echo ""

# GitHub Actions用のサービスアカウントを作成
echo "👤 GitHub Actions用のサービスアカウントを作成しています..."
gcloud iam service-accounts create github-actions \
  --display-name="GitHub Actions Deployment" \
  2>/dev/null || echo "ℹ️  サービスアカウントは既に存在します"

# 必要な権限を付与
echo "🔑 サービスアカウントに権限を付与しています..."
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/run.admin" \
  --quiet

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/artifactregistry.writer" \
  --quiet

gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:github-actions@${PROJECT_ID}.iam.gserviceaccount.com" \
  --role="roles/iam.serviceAccountUser" \
  --quiet

echo "✅ 権限の付与が完了しました"
echo ""

# サービスアカウントキーを作成
echo "🔑 サービスアカウントキーを作成しています..."
KEY_FILE="github-actions-key.json"

if [ -f "$KEY_FILE" ]; then
    echo "⚠️  既存のキーファイルが見つかりました。上書きしますか？ (y/n)"
    read -p "> " OVERWRITE
    if [ "$OVERWRITE" != "y" ]; then
        echo "ℹ️  キーファイルの作成をスキップしました"
        KEY_FILE=""
    fi
fi

if [ -n "$KEY_FILE" ]; then
    gcloud iam service-accounts keys create $KEY_FILE \
      --iam-account=github-actions@${PROJECT_ID}.iam.gserviceaccount.com
    echo "✅ サービスアカウントキーを作成しました: $KEY_FILE"
fi
echo ""

# 完了メッセージ
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ セットアップが完了しました！"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 次のステップ:"
echo ""
echo "1. GitHubリポジトリの Settings > Secrets and variables > Actions に移動"
echo ""
echo "2. 以下のシークレットを追加:"
echo ""
echo "   GCP_PROJECT_ID:"
echo "   $PROJECT_ID"
echo ""
if [ -f "$KEY_FILE" ]; then
    echo "   GCP_SA_KEY:"
    echo "   (以下のコマンドでキーの内容をコピー)"
    echo "   cat $KEY_FILE"
    echo ""
fi
echo "3. コードをpushしてデプロイをテスト:"
echo "   git add ."
echo "   git commit -m \"Setup GitHub Actions deployment\""
echo "   git push origin main"
echo ""
echo "4. GitHub ActionsのUIでデプロイの進行状況を確認"
echo ""
echo "⚠️  重要: キーファイルを安全に保管し、バージョン管理にコミットしないでください！"
if [ -f "$KEY_FILE" ]; then
    echo "   GitHubにシークレットを追加したら、以下のコマンドでキーファイルを削除してください:"
    echo "   rm $KEY_FILE"
fi
echo ""
echo "📚 詳細なドキュメント: .github/SETUP.md"
echo ""
