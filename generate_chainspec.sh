#!/usr/bin/env bash
chain-spec-builder \
  --chain-spec-path ./chainspec/chainspec.json \
  create \
  --raw-storage \
  --relay-chain="rococo-local" \
  --para-id=1000 \
  --runtime="./target/production/wbuild/decomp-runtime/decomp_runtime.wasm" \
  --verify \
  --chain-name="Decomp Recoleta Testnet" \
  --chain-id=decomp_recoleta \
  --protocol-id=decomp_recoleta \
  -t=local \
  --properties tokenSymbol=DCMP,tokenDecimals=12,ss58Format=42,isEthereum=false \
  named-preset development