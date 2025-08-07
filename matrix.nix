{ pkgs, lib, config, ... }:
{
    networking.firewall.allowedTCPPorts = [ 80 443 ];

    services.postgresql = {
      enable = true;
      ensureDatabases = [ "synapse" ];
      ensureUsers = [
        {
            name = "synapse";
            ensureDBOwnership = true;
        }
      ];
    };

    services.matrix-synapse = {
        enable = true;

        settings = {
            server_name = "localhost";
            public_baseurl = "http://localhost:8008/";

            tls = false;

            enable_registration = false;
            allow_guest_access = false;
            registration_requires_token = false;
            suppress_key_server_warning = true;

            registration_shared_secret = "some-secret-value";

            listeners = [
              {
                port = 8008;
                bind_addresses = [ "127.0.0.1" ];
                type = "http";
                tls = false;
                resources = [
                  {
                    names = [ "client" "federation" ];
                    compress = false;
                  }
                ];
              }
            ];
            database = {
                name = "psycopg2";
                args = {
                  user = "synapse";
                  password = "mysupersecretpassword";
                  database = "synapse";
                  host = "localhost";
                };
            };
        };
    };
}
