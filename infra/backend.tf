# Remote state in GCS. Bootstrap the bucket ONCE, then uncomment and
# `terraform init -migrate-state`:
#
#   gcloud storage buckets create gs://<PROJECT>-tfstate --location=<REGION> \
#     --uniform-bucket-level-access --public-access-prevention
#
# terraform {
#   backend "gcs" {
#     bucket = "<PROJECT>-tfstate"
#     prefix = "music/infra"
#   }
# }
