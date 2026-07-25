resource "google_cloud_run_v2_service" "server" {
  name     = var.service_name
  location = var.region
  # Public endpoint: browsers (on Tailscale) and agents reach it directly.
  # App-layer auth (Firebase JWT + agent tokens) is the real gate, added in M2.
  ingress = "INGRESS_TRAFFIC_ALL"

  template {
    service_account = google_service_account.runtime.email

    scaling {
      min_instance_count = 0 # scale to zero — cheap by default
      max_instance_count = 4
    }

    containers {
      image = var.server_image
      ports {
        container_port = 8080
      }
      # NOTE: /healthz + /readyz probes are wired in M2 once our own image is the
      # deployed one. The default hello image only serves "/", so adding an HTTP
      # probe now would fail the first apply.
    }
  }

  depends_on = [google_project_service.enabled]
}

# Publicly invokable (auth is enforced in the app, not by Cloud Run IAM).
resource "google_cloud_run_v2_service_iam_member" "public" {
  name     = google_cloud_run_v2_service.server.name
  location = google_cloud_run_v2_service.server.location
  role     = "roles/run.invoker"
  member   = "allUsers"
}
