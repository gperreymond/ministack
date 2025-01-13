# Variables
CONSUL_HOST="http://10.1.0.101:8500"
SERVICE_NAME="default%2Fdefault%2F*"
DESTINATION_SERVICE="default%2Fdefault%2F*"

echo "===== apply default intension ====="

# Step 1: Create or update the intention
echo "[INFO] creating/updating intention for $SERVICE_NAME to communicate with $DESTINATION_SERVICE..."
curl  --silent --request PUT \
      --data "{
        \"Action\": \"deny\"
      }" \
      "$CONSUL_HOST/v1/connect/intentions/exact?source=$SERVICE_NAME&destination=$DESTINATION_SERVICE" > /dev/null 2>&1
echo "[INFO] intention created/updated."
echo ""
