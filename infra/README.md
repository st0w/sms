# infra

Terraform for the control-plane server on GCP: Artifact Registry, Cloud Run
(scale-to-zero, public), a least-privilege runtime service account, Secret
Manager containers, and **keyless** GitHub Actions deploy via Workload Identity
Federation. Neon (Postgres) and Firebase/Identity Platform are provisioned in M2;
their secrets already have containers here (`db-url`, `firebase-config`).

## First apply
```bash
cd infra
cp terraform.tfvars.example terraform.tfvars   # edit project_id + github_repo
terraform init
terraform apply
```
The first apply uses Google's public hello image, so `server_url` returns a live
HTTPS endpoint immediately. CI then builds and deploys the real image on pushes
to `main`.

## After apply — wire GitHub Actions
Add these repo variables (Settings → Secrets and variables → Actions → Variables):
- `GCP_PROJECT_ID`
- `GCP_REGION`
- `GCP_WIF_PROVIDER` = `terraform output -raw wif_provider`
- `GCP_DEPLOYER_SA` = `terraform output -raw deployer_sa_email`

## Populate secrets (out-of-band, never in git)
```bash
printf '%s' "$NEON_URL" | gcloud secrets versions add db-url --data-file=-
```

## Notes
- Remote state: see `backend.tf` to migrate to a GCS bucket.
- Cloud Run is public by design; auth is enforced in the app (M2).
