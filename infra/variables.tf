variable "project_id" {
  type        = string
  description = "GCP project ID."
}

variable "region" {
  type        = string
  description = "Primary region."
  default     = "us-central1"
}

variable "service_name" {
  type        = string
  description = "Cloud Run service name."
  default     = "music-server"
}

variable "artifact_repo_id" {
  type        = string
  description = "Artifact Registry Docker repository ID."
  default     = "music"
}

variable "github_repo" {
  type        = string
  description = "GitHub repo allowed to deploy, as 'owner/name'. Used to scope Workload Identity Federation."
}

variable "server_image" {
  type        = string
  description = "Container image for Cloud Run. Defaults to Google's public hello image so the first apply succeeds before CI has pushed our image."
  default     = "us-docker.pkg.dev/cloudrun/container/hello"
}
