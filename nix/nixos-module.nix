{ config, lib, pkgs, ... }:

let
  cfg = config.services.wikimark;
in
{
  options.services.wikimark = {
    enable = lib.mkEnableOption "wikimark wiki server";

    package = lib.mkPackageOption pkgs "wikimark" { };

    port = lib.mkOption {
      type = lib.types.port;
      default = 3007;
      description = "Port to listen on.";
    };

    address = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Address to bind to.";
    };

    repoPath = lib.mkOption {
      type = lib.types.path;
      default = "/var/lib/wikimark/repo";
      description = "Path to the git repository for wiki content.";
    };

    commitUrlPrefix = lib.mkOption {
      type = lib.types.str;
      default = "";
      example = "https://github.com/user/wiki/commit/";
      description = "URL prefix for linking to commits.";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "wikimark";
      description = "User to run wikimark as.";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "wikimark";
      description = "Group to run wikimark as.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = ''
        Environment file for additional configuration.
        Can be used to set WIKIMARK_USER, WIKIMARK_LOG, etc.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.${cfg.user} = lib.mkIf (cfg.user == "wikimark") {
      isSystemUser = true;
      group = cfg.group;
      home = "/var/lib/wikimark";
      createHome = true;
    };

    users.groups.${cfg.group} = lib.mkIf (cfg.group == "wikimark") { };

    systemd.services.wikimark = {
      description = "Wikimark wiki server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = ''
          ${lib.getExe cfg.package} \
            --port ${toString cfg.port} \
            --address ${cfg.address} \
            --repo ${cfg.repoPath} \
            --commit-url-prefix "${cfg.commitUrlPrefix}"
        '';
        Restart = "on-failure";
        RestartSec = "5s";

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ReadWritePaths = [ cfg.repoPath ];
      } // lib.optionalAttrs (cfg.environmentFile != null) {
        EnvironmentFile = cfg.environmentFile;
      };
    };
  };
}
