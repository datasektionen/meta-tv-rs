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
        "traefik.http.routers.betting.rule=Host(`tv.betasektionen.se`)",
        "traefik.http.routers.betting.tls.certresolver=default",
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
ROCKET_OIDC={issuer_url="https://sso.datasektionen.se/op",client_id="{{ .oidc_client_id }}",client_secret="{{ .oidc_client_secret }}",redirect_url="http://localhost:8000/auth/oidc-callback"}
ROCKET_SECRET_KEY={{ .app_secret }}
{{ end }}
ROCKET_PORT={{ env "NOMAD_PORT_http" }}
ROCKET_ADDRESS=0.0.0.0
ROCKET_UPLOAD_DIR="./uploads"
ROCKET_LIMITS_FILE=50MiB
ROCKET_LIMITS_DATA_FORM=51MiB
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
  default = "ghcr.io/datasektionen/meta-tv:latest"
}
