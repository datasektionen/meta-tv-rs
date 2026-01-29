job "meta-tv" {
  type = "service"

  group "meta-tv" {
    network {
      port "http" { }
    }

    service {
      name     = "meta-tv"
      port     = "http"
      provider = "nomad"
      tags = [
        "traefik.enable=true",
        "traefik.http.routers.meta-tv.rule=Host(`tv.datasektionen.se`)",
        "traefik.http.routers.meta-tv.tls.certresolver=default",
      ]
    }

    task "meta-tv" {
      driver = "docker"

      config {
        image = var.image_tag
        ports = ["http"]
      }

      template {
        data        = <<ENV
{{ with nomadVar "nomad/jobs/meta-tv" }}
ROCKET_DATABASES={sea_orm={url="postgresql://metatv:{{ .db_password }}@postgres.dsekt.internal:5432/metatv"}}
ROCKET_OIDC={issuer_url="https://sso.datasektionen.se/op",client_id="{{ .oidc_client_id }}",client_secret="{{ .oidc_client_secret }}",redirect_url="https://tv.datasektionen.se/auth/oidc-callback"}
ROCKET_HIVE={url="https://hive.datasektionen.se/api/v1",secret="{{ .hive_secret }}"}
ROCKET_SECRET_KEY={{ .app_secret }}
ROCKET_S3={url="{{ .aws_url }}", bucket="meta-tv"}
AWS_ACCESS_KEY_ID="{{ .aws_access_key_id }}"
AWS_SECRET_ACCESS_KEY="{{ .aws_secret_access_key }}"
{{ end }}
ROCKET_PORT={{ env "NOMAD_PORT_http" }}
ROCKET_ADDRESS=0.0.0.0
ROCKET_LIMITS={file="50MiB", data-form="51MiB"}
FEED_ENTRY_DURATION=30000
ENV
        destination = "local/.env"
        env         = true
      }

      resources {
        memory = 120
      }
    }
  }
}

variable "image_tag" {
  type = string
  default = "ghcr.io/datasektionen/meta-tv-rs:latest"
}
