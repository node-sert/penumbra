#!/bin/bash
WORKDIR=${WORKDIR:=$(pwd)/helm/pdcli}
IMAGE=${IMAGE:=ghcr.io/strangelove-ventures/heighliner/penumbra}
PENUMBRA_VERSION=${PENUMBRA_VERSION:=030-isonoe}
TENDERMINT_VERSION=${TENDERMINT_VERSION:=v0.34.21}
NVALS=${NVALS:=2}
NFULLNODES=${NFULLNODES:=1}
CONTAINERHOME=${CONTAINERHOME:=/root}
HELM_RELEASE=${HELM_RELEASE:=testnet}

# Use fresh working directory
sudo rm -rf ${WORKDIR}
mkdir -p "${WORKDIR}"

echo "Shutting down existing testnet if necessary..."
# Delete existing replication controllers
kubectl delete rc --all --wait=false 2>&1 > /dev/null
# Delete all existing PVCs so that fresh testnet is created
kubectl delete pvc --all 2>&1 > /dev/null

for i in $(seq $NVALS)
do
    I="$((i-1))"
    NODEDIR="node$I"
    mkdir -p "${WORKDIR}/$NODEDIR"
    # This will be overwritten by pd testnet generate.
    echo '{"identity_key": "penumbravalid1lr73zgd726gpk7rl45hvpg9f7r9wchgg8gpjhx2gqntx4md6gg9sser05u","consensus_key": "9OQ8HOy4YsryEPLbTtPKoKdmmjSqEJhzvS+x0WC8YoM=","name": "","website": "","description": "","enabled": false,"funding_streams": [{"address": "penumbrav2t1wz70yfqlgzfgwml5ne04vhnhahg8axmaupuv7x0gpuzesfhhz63y52cqffv93k7qvuuq6yqtgcj0z267v59qxpjuvc0hvfaynaaemgmqzyj38xhj8yjx7vcftnyq9q28exjrdj","rate_bps": 100}],"sequence_number": 0,"governance_key": "penumbragovern1lr73zgd726gpk7rl45hvpg9f7r9wchgg8gpjhx2gqntx4md6gg9sthagp6"}' > "${WORKDIR}/$NODEDIR/val.json"
done

find "$WORKDIR" -name "val.json" -exec cat {} + | jq -s > "$WORKDIR/vals.json"

echo "Generating new testnet files..."
docker run --user 0:0 \
-v "$WORKDIR":"$CONTAINERHOME" -it --rm \
--entrypoint pd \
$IMAGE:$PENUMBRA_VERSION \
testnet generate \
--validators-input-file "$CONTAINERHOME/vals.json" > /dev/null

sudo chown -R "$(whoami)" "$WORKDIR"

for i in $(seq $NVALS)
do
    I=$((i-1))
    NODE_ID=$(jq -r '.priv_key.value' ${WORKDIR}/.penumbra/testnet_data/node$I/tendermint/config/node_key.json | base64 --decode | tail -c 32 | sha256sum  | cut -c -40)
    for j in $(seq $NVALS)
    do
      J=$((j-1))
      if [ "$I" -ne "$J" ]; then
        PVAR=PERSISTENT_PEERS_$J
        if [ -z "${!PVAR}" ]; then
          declare PERSISTENT_PEERS_$J="$NODE_ID@p2p-$I:26656"
        else
          declare PERSISTENT_PEERS_$J="$PERSISTENT_PEERS,$NODE_ID@p2p-$I:26656"
        fi
      fi
    done
    if [ -z "$PERSISTENT_PEERS" ]; then
      PERSISTENT_PEERS="$NODE_ID@p2p-$I:26656"
      PRIVATE_PEERS="$NODE_ID"
    else
      PERSISTENT_PEERS="$PERSISTENT_PEERS,$NODE_ID@p2p-$I:26656"
      PRIVATE_PEERS="$PRIVATE_PEERS,$NODE_ID"
    fi
done

for i in $(seq $NVALS)
do
  I=$((i-1))
  PVAR=PERSISTENT_PEERS_$I
  echo "${!PVAR}" > $WORKDIR/persistent_peers_$I.txt
done

echo "$PERSISTENT_PEERS" > $WORKDIR/persistent_peers.txt
echo "$PRIVATE_PEERS" > $WORKDIR/private_peers.txt

helm get values $HELM_RELEASE 2>&1 > /dev/null
if [ "$?" -eq "0" ]; then
  HELM_CMD=upgrade
else
  HELM_CMD=install
fi

helm $HELM_CMD $HELM_RELEASE helm --set numValidators=$NVALS,numFullNodes=$NFULLNODES,penumbra.version=$PENUMBRA_VERSION,tendermint.version=$TENDERMINT_VERSION
