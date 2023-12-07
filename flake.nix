{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-23.11";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    inherit (nixpkgs) lib;
    defaultSystems = [
      "x86_64-linux"
      "x86_64-darwin"
      "aarch64-linux"
      "aarch64-darwin"
    ];
    eachDefaultSystem = f:
      builtins.listToAttrs (map (system: {
          name = system;
          value = f (import nixpkgs {inherit system;});
        })
        defaultSystems);
  in {
    packages = eachDefaultSystem (pkgs: let
      inherit (fromTOML (builtins.readFile ./Cargo.toml)) package;
    in {
      default = pkgs.rustPlatform.buildRustPackage {
        pname = package.name;
        version = package.version;
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./src
          ];
        };
        cargoLock.lockFile = ./Cargo.lock;
        doCheck = false;
      };
    });

    nixosModules.default = {
      config,
      lib,
      pkgs,
      ...
    }: {
      options.services.distodon = with lib; {
        enable = mkEnableOption "distodon";
        debug = mkOption {
          type = types.bool;
          default = false;
        };
        interval = mkOption {
          type = types.ints.positive;
          default = 600;
        };
        chunkSize = mkOption {
          type = types.ints.between 1 10;
          default = 1;
        };
        links = mkOption {
          type = types.listOf (types.submodule {
            options = {
              mastodonServerUrl = mkOption {
                type = types.str;
              };
              mastodonUser = mkOption {
                type = types.str;
              };
              webhookUrl = mkOption {
                type = types.nullOr types.str;
                default = null;
              };
              webhookUrlFile = mkOption {
                type = types.nullOr types.path;
                default = null;
              };
            };
          });
        };
      };

      config = let
        cfg = config.services.distodon;
        links = builtins.genList (i: rec {
          inherit i;
          link = builtins.elemAt cfg.links i;
          secretName = "webhook-url-${toString i}";
        }) (builtins.length cfg.links);
        linksWithSecrets = builtins.filter ({link, ...}: link.webhookUrlFile != null) links;

        configFile = (pkgs.formats.toml {}).generate "distodon-config.toml" {
          interval = cfg.interval;
          chunk_size = cfg.chunkSize;
          links = map (link:
            {
              mastodon_server_url = link.mastodonServerUrl;
              mastodon_user = link.mastodonUser;
            }
            // (lib.optionalAttrs (link.webhookUrl != null) {
              webhook_url = link.webhookUrl;
            }))
          cfg.links;
        };
      in
        lib.mkIf cfg.enable {
          assertions = lib.flatten (map ({
              webhookUrl,
              webhookUrlFile,
              ...
            }: [
              {
                assertion = (webhookUrl != null) != (webhookUrlFile != null);
                message = "Exactly one of webhookUrl and webhookUrlFile must be set";
              }
            ])
            cfg.links);

          systemd.services.distodon = {
            wantedBy = ["multi-user.target"];
            serviceConfig = {
              User = "distodon";
              Group = "distodon";
              DynamicUser = true;
              WorkingDirectory = "/run/distodon";
              RuntimeDirectory = "distodon";
              StateDirectory = "distodon";
              LoadCredential = builtins.map ({
                link,
                secretName,
                ...
              }: "${secretName}:${link.webhookUrlFile}")
              linksWithSecrets;
            };
            environment = {
              RUST_LOG =
                if cfg.debug
                then "debug"
                else "info";
            };
            preStart = ''
              cd /run/distodon
              rm -f config.toml data
              ln -s /var/lib/distodon data
              cp ${configFile} config.toml
              chmod 600 config.toml

              ${builtins.concatStringsSep "\n" (map ({
                  i,
                  secretName,
                  ...
                }: ''
                  webhook_url=$(cat ''${CREDENTIALS_DIRECTORY}/${secretName})
                  ${pkgs.yq}/bin/tomlq -i -t --argjson i ${toString i} --arg webhook_url "$webhook_url" '.links[$ARGS.named.i].webhook_url = $ARGS.named.webhook_url' config.toml
                '')
                linksWithSecrets)}
            '';
            script = ''
              ${self.packages.${pkgs.system}.default}/bin/distodon
            '';
          };
        };
    };
  };
}
