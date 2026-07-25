# ---------- Runtime identity for the Cloud Run service (least privilege) ----------
resource "google_service_account" "runtime" {
  account_id   = "music-server-runtime"
  display_name = "Cloud Run runtime SA for the music server"
}

# ---------- Keyless CI/CD via Workload Identity Federation ----------
# GitHub Actions impersonates the deployer SA using a short-lived OIDC token.
# No service-account JSON keys are ever created or stored.
resource "google_iam_workload_identity_pool" "github" {
  workload_identity_pool_id = "github-pool"
  display_name              = "GitHub Actions"
  depends_on                = [google_project_service.enabled]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github-provider"
  display_name                       = "GitHub OIDC"

  attribute_mapping = {
    "google.subject"             = "assertion.sub"
    "attribute.repository"       = "assertion.repository"
    "attribute.repository_owner" = "assertion.repository_owner"
  }

  # Restrict federation to THIS repository only. Without a condition the provider
  # is rejected; this also prevents any other repo from minting tokens.
  attribute_condition = "assertion.repository == \"${var.github_repo}\""

  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

resource "google_service_account" "deployer" {
  account_id   = "music-ci-deployer"
  display_name = "GitHub Actions deployer"
}

# Only workflows from the configured repo may impersonate the deployer SA.
resource "google_service_account_iam_member" "deployer_wif" {
  service_account_id = google_service_account.deployer.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository/${var.github_repo}"
}

# Deployer permissions: deploy Cloud Run, push images, and act as the runtime SA.
resource "google_project_iam_member" "deployer_run" {
  project = var.project_id
  role    = "roles/run.developer"
  member  = "serviceAccount:${google_service_account.deployer.email}"
}

resource "google_project_iam_member" "deployer_ar" {
  project = var.project_id
  role    = "roles/artifactregistry.writer"
  member  = "serviceAccount:${google_service_account.deployer.email}"
}

# Scoped to the runtime SA only (not project-wide) — least privilege.
resource "google_service_account_iam_member" "deployer_actas_runtime" {
  service_account_id = google_service_account.runtime.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.deployer.email}"
}
