# Vaktmester-appen — nøyaktig klikkeliste for Jørn

Vaktmester-appen er en GitHub App som gir Studio 15 LIGHT repo-automatikk i
program-orgene: opprette prosjektrepoer, hente push-tokens til
arbeidsflatene og slette repoer i sletteregimet. Lærdommen fra Studio 15
gjelder: deploy keys er avslått som standard i nye orger, og app-tokens har
særregler — derfor app, ikke PAT. (PAT-en på raven dekker ikke KODE15AS.)

Alt under er manuelle GitHub-steg. Resten (nøkkelhåndtering, API-bruk)
automatiseres i repoet etterpå.

## 1. Opprett appen (én gang)

1. Logg inn på github.com med KODE15-kontoen.
2. Gå til **organisasjonen KODE15AS → Settings** (org-innstillingene, ikke
   dine egne): `https://github.com/organizations/KODE15AS/settings/apps`
3. Venstremeny: **Developer settings → GitHub Apps → New GitHub App**.
4. Fyll ut:
   - **GitHub App name:** `studio15-light-vaktmester`
   - **Homepage URL:** `https://github.com/KODE15AS/studio15-light`
   - **Webhook:** fjern haken **Active** (vi bruker ikke webhooks).
5. **Permissions → Repository permissions:**
   - **Administration:** Read and write (opprette/slette repoer)
   - **Contents:** Read and write (push-tokens til arbeidsflatene)
   - **Metadata:** Read-only (settes automatisk)
   - Alt annet: No access.
6. **Where can this GitHub App be installed?** → **Any account**
   (appen skal installeres i hver program-org).
7. Klikk **Create GitHub App**.

## 2. Nøkkel og ID-er (rett etter opprettingen)

1. På appens side: notér **App ID** (øverst).
2. Under **Private keys**: klikk **Generate a private key** — en `.pem`-fil
   lastes ned.
3. Lever `.pem`-fila og App ID til agenten på raven (eller legg innholdet i
   Bitwarden, KODE15-hvelvet, som «studio15-light-vaktmester»). Fila skal
   ende i stackens `.env`-regime på raven — aldri i git.

## 3. Installer appen i org-ene

For **hver** program-org (og gjerne KODE15AS selv):

1. Appens side → **Install App** (venstremeny).
2. Velg org-en → **Install**.
3. **Repository access:** **All repositories** (prosjektrepoer opprettes og
   slettes løpende — appen må se dem alle).

## 4. Ved nytt program senere

Nytt program = ny GitHub-org (manuelt, GitHub har ikke API for dette):

1. github.com → profilikonet → **Settings → Organizations →
   New organization** → Free.
2. Org-navn med KODE15-prefiks, f.eks. `KODE15-<program>`.
3. Installer vaktmester-appen i den nye org-en (punkt 3 over).
4. Legg org-navnet inn i `register/programmer.yaml` (`github_org:`).

## 5. Sletteregimets ene manuelle unntak

Repoer sletter vaktmesteren. **Org-sletting** finnes ikke i GitHubs API og
gjøres manuelt når et helt program legges ned:
org-en → Settings → nederst **Delete this organization**.
