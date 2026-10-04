#!/usr/bin/env bash
# DNS-01-hook for lego (EXEC-provider): setter/fjerner ACME-TXT-posten via
# GitOps-flyten i deploi-dns (git er fasit, applikatoren utfører mot
# Deploi-API-et ved push — webhook, sekunder). Brukes av
# hent-lobby-sertifikat.sh og forny-lobby-sertifikat.sh.
#
# Kalles av lego som:  <hook> present <fqdn> <verdi>   og
#                      <hook> cleanup <fqdn> <verdi>
set -euo pipefail

AKSJON="$1"; shift
[ "${1:-}" = "--" ] && shift        # lego setter inn «--» i default-modus
FQDN="${1%.}"                       # f.eks. _acme-challenge.lobby.studio15.cloud
VERDI="${2:-}"
DNS_REPO="${DNS_REPO:-$HOME/dev/deploi-dns}"
SONE="$DNS_REPO/zones/studio15.cloud.yaml"
HOST="${FQDN%.studio15.cloud}"      # f.eks. _acme-challenge.lobby

cd "$DNS_REPO"
git pull -q --rebase                # applikatoren kan ha committet sync-state

python3 - "$SONE" "$AKSJON" "$HOST" "$VERDI" <<'PY'
import sys, yaml
sone, aksjon, host, verdi = sys.argv[1:5]
with open(sone) as f:
    d = yaml.safe_load(f)
rec = {"type": "TXT", "host": host, "value": verdi, "ttl": 300,
       "priority": 0, "weight": 0, "port": 0, "protocol": ""}
poster = d["records"]
if aksjon == "present":
    if rec not in poster:
        poster.append(rec)
else:  # cleanup: fjern alle ACME-TXT for hosten
    d["records"] = [r for r in poster
                    if not (r["type"] == "TXT" and r["host"] == host)]
with open(sone, "w") as f:
    yaml.safe_dump(d, f, allow_unicode=True, sort_keys=False)
PY

if ! git diff --quiet; then
  git add "$SONE"
  git commit -q -m "ACME DNS-01 ($AKSJON): $HOST (lobby-sertifikat, automatisk)"
  git push -q
fi
