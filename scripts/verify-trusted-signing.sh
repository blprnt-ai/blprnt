#!/usr/bin/env bash
set -euo pipefail

az account set --subscription "$AZURE_SUBSCRIPTION_ID" >/dev/null

echo "Subscription ID: $AZURE_SUBSCRIPTION_ID"

account_id="$(
  az resource show \
    --resource-group "$AZURE_RESOURCE_GROUP" \
    --name "$AZURE_TRUSTED_SIGNING_ACCOUNT_NAME" \
    --resource-type "Microsoft.CodeSigning/codeSigningAccounts" \
    --query "id" -o tsv
)"

echo "Account ID: $account_id"

if [[ -z "$account_id" ]]; then
  echo "ERROR: Trusted Signing account not found."
  exit 1
fi

echo "OK: Found Trusted Signing account:"
echo "  $account_id"

profiles="$(
  az rest \
    --method get \
    --url "https://management.azure.com${account_id}/certificateProfiles?api-version=2025-10-13" \
  | jq -r '.value[].name'
)"

if ! echo "$profiles" | grep -Fxq "$AZURE_TRUSTED_SIGNING_CERT_PROFILE_NAME"; then
  echo "ERROR: Certificate profile not found: $AZURE_TRUSTED_SIGNING_CERT_PROFILE_NAME"
  echo "Available profiles:"
  echo "$profiles" | sed 's/^/  - /'
  exit 1
fi

echo "OK: Certificate profile exists: $AZURE_TRUSTED_SIGNING_CERT_PROFILE_NAME"

expected_endpoint="$(
  az resource show \
    --ids "$account_id" \
    --query "properties.endpoint" -o tsv 2>/dev/null || true
)"

if [[ -n "$expected_endpoint" && "$expected_endpoint" != "$AZURE_TRUSTED_SIGNING_ENDPOINT" ]]; then
  echo "ERROR: Endpoint mismatch."
  echo "  Env var:  $AZURE_TRUSTED_SIGNING_ENDPOINT"
  echo "  Azure:    $expected_endpoint"
  exit 1
fi

echo "OK: Endpoint looks consistent."

sp_object_id="$(
  az ad sp show --id "$AZURE_CLIENT_ID" --query id -o tsv
)"

echo "Service principal object id: $sp_object_id"

echo "Role assignments scoped to account:"
az role assignment list \
  --assignee-object-id "$sp_object_id" \
  --scope "$account_id" \
  -o table || true

echo "Role assignments scoped to certificate profiles (if any):"
az role assignment list \
  --assignee-object-id "$sp_object_id" \
  --scope "${account_id}/certificateProfiles/${AZURE_TRUSTED_SIGNING_CERT_PROFILE_NAME}" \
  -o table || true

echo "DONE: Resource + profile exist; permissions listed above."
echo "NOTE: Actual signing can only be validated on Windows with signtool."
