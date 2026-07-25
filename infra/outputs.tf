output "server_url" {
  description = "Public HTTPS URL of the Cloud Run service."
  value       = google_cloud_run_v2_service.server.uri
}

output "artifact_registry" {
  description = "Docker repo path for pushing the server image."
  value       = "${var.region}-docker.pkg.dev/${var.project_id}/${var.artifact_repo_id}"
}

output "wif_provider" {
  description = "Full resource name of the WIF provider (for the GitHub Actions auth step)."
  value       = google_iam_workload_identity_pool_provider.github.name
}

output "deployer_sa_email" {
  description = "Deployer service account email (for the GitHub Actions auth step)."
  value       = google_service_account.deployer.email
}
