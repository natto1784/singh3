job "singh3" {
  region      = "global"
  datacenters = ["nazrin"]
  type        = "service"

  group "svc" {
    count = 1

    network {
      mode = "bridge"

      port "db" {
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

      lifecycle {
        hook = "prestart"
        sidecar = true
      }

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
    }

    task "bot" {
      driver = "raw_exec"

      config {
        command = "/bin/sh"
        args = [ "-c", <<EOF
/run/current-system/sw/bin/nix-store --realise {{+.storepath+}}
{{+.storepath+}}/bin/{{+.binary+}}
EOF
]
      }

      template {
        data = <<EOF
{{with secret "kv/data/singh3/db"}}
DB_URL="postgresql://singh3:{{.Data.data.pass}}@localhost:{{env "NOMAD_PORT_db"}}/singh3"
{{end}}
{{with secret "kv/data/singh3/discord"}}
DISCORD_TOKEN="{{.Data.data.token}}"
{{end}}
RUST_BACKTRACE=full
EOF

        destination = "${NOMAD_SECRETS_DIR}/data.env"
        env         = true
      }
    }
  }
}
