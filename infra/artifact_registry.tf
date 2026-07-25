resource "google_artifact_registry_repository" "docker" {
  location      = var.region
  repository_id = var.artifact_repo_id
  format        = "DOCKER"
  description   = "Container images for the music control-plane server."

  depends_on = [google_project_service.enabled]
}
