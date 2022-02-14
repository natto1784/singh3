job "singh3" {
  region      = "global"
  datacenters = ["nazrin"]
  type        = "service"

  group "svc" {
    count = 1

    network {
      mode = "bridge"

      port "db" {
        static = 5454
        to     = 5432
      }
    }

    vault {
      policies = ["singh3-policy"]
    }

    service {
      name = "singh3-db"
      port = "db"
    }

    task "db" {
      template {
        data = <<EOF
{{with secret "kv/data/singh3/db"}}{{.Data.data.pass}}{{end}}
EOF

        destination = "${NOMAD_SECRETS_DIR}/db.pass"
      }

      driver = "docker"

      config {
        image   = "postgres:alpine"
        ports   = ["db"]
        volumes = ["/var/lib/nomad-st/postgres-singh3:/var/lib/postgresql/data"]
      }

      env {
        POSTGRES_USER          = "singh3"
        POSTGRES_PASSWORD_FILE = "${NOMAD_SECRETS_DIR}/db.pass"
        POSTGRES_DB            = "singh3"
      }

      resources {
        cpu    = 256
        memory = 128
      }
    }

    task "bot" {
      driver = "docker"

      config {
        image      = "natto17/singh3:latest"
        force_pull = true
      }

      template {
        data = <<EOF
{{with secret "kv/data/singh3/db"}}
DB_URL="postgresql://singh3:{{.Data.data.pass}}@localhost:5432/singh3"
{{end}}
{{with secret "kv/data/singh3/discord"}}
DISCORD_TOKEN="{{.Data.data.token}}"
{{end}}
RUST_BACKTRACE=1
EOF

        destination = "${NOMAD_SECRETS_DIR}/data.env"
        env         = true
      }
    }
  }
}
