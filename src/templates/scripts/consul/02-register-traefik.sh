# Variables
CONSUL_HOST="http://10.1.0.101:8500"
SERVICE_NAME="traefik"
SERVICE_ID="traefik-1"
SERVICE_ADDRESS="10.1.0.10"
SERVICE_PORT=8080
HEALTH_CHECK_URL="http://10.1.0.10:8080/ping"
DESTINATION_SERVICE="*"

echo "===== register traefik ====="

# Step 1: Deregister the service (ignore errors if the service doesn't exist)
echo "[INFO] attempting to deregister service $SERVICE_ID..."
curl --silent --request PUT "$CONSUL_HOST/v1/agent/service/deregister/$SERVICE_ID" || true
echo "[INFO] service $SERVICE_ID deregistered if it existed, continuing..."

# Step 2: Register the service
echo "[INFO] registering service $SERVICE_NAME..."
curl  --silent --request PUT \
      --data "{
        \"Name\": \"$SERVICE_NAME\",
        \"ID\": \"$SERVICE_ID\",
        \"Address\": \"$SERVICE_ADDRESS\",
        \"Port\": $SERVICE_PORT,
        \"Connect\": {
          \"SidecarService\": {}
        },
        \"Check\": {
          \"HTTP\": \"$HEALTH_CHECK_URL\",
          \"Interval\": \"10s\",
          \"Timeout\": \"1s\"
        }
      }" \
      "$CONSUL_HOST/v1/agent/service/register" > /dev/null 2>&1
echo "[INFO] service $SERVICE_NAME registered."

# Step 3: Create or update the intention
echo "[INFO] creating/updating intention for $SERVICE_NAME to communicate with $DESTINATION_SERVICE..."
curl  --silent --request PUT \
      --data "{
        \"Action\": \"allow\"
      }" \
      "$CONSUL_HOST/v1/connect/intentions/exact?source=$SERVICE_NAME&destination=$DESTINATION_SERVICE" > /dev/null 2>&1
echo "[INFO] intention created/updated."
echo ""
