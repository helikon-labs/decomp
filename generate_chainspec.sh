#!/usr/bin/env bash
chain-spec-builder \
  --chain-spec-path ./chainspec/chainspec.json \
  create \
  --raw-storage \
  --relay-chain="rococo-local" \
  --para-id=1000 \
  --runtime="./target/production/wbuild/decomp-runtime/decomp_runtime.wasm" \
  --verify \
  --chain-name="Decomp Testnet" \
  --chain-id=decomp_testnet \
  -t=local \
  --properties tokenSymbol=DCMP,tokenDecimals=12,ss58Format=42,isEthereum=false \
  named-preset development