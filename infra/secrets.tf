# Secret CONTAINERS only — values are added out-of-band (never in Terraform/git):
#   printf '%s' "$VALUE" | gcloud secrets versions add <name> --data-file=-
# The runtime SA is granted access to exactly these secrets (least privilege).
# Consumed by the server starting in M2.
locals {
  secret_ids = [
    "db-url",              # Neon connection string
    "token-signing-keys",  # hybrid Ed25519 + ML-DSA private keys (JSON)
    "firebase-config",     # Firebase/Identity Platform config
  ]
}

resource "google_secret_manager_secret" "app" {
  for_each  = toset(local.secret_ids)
  secret_id = each.value
  replication {
    auto {}
  }
  depends_on = [google_project_service.enabled]
}

resource "google_secret_manager_secret_iam_member" "runtime_access" {
  for_each  = google_secret_manager_secret.app
  secret_id = each.value.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.runtime.email}"
}
