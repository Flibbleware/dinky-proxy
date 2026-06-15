// This file is managed by DinkyProxy; it will be regenerated on startup from config.json
function FindProxyForURL(url, host) {
    const myProxy = "PROXY localhost:8888";

    if (host === "localhost" || host === "127.0.0.1") {
        return "DIRECT";
    }

    return "DIRECT";
}
