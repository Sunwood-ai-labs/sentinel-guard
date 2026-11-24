// Safe live TCP probe: opens a raw TCP socket to us.i.posthog.com:443 (sends ZERO bytes / ZERO user data)
// Holds the connection briefly so Sentinel Guard's live monitor can detect the active telemetry connection by PID.
import net from "node:net";

const HOST = "us.i.posthog.com";
const PORT = 443;
const HOLD_MS = 6000;

const socket = net.createConnection({ host: HOST, port: PORT }, () => {
  console.log(`CONNECTED ${socket.remoteAddress}:${socket.remotePort} pid=${process.pid}`);
});

socket.on("error", (err) => {
  console.error(`ERR ${err.message}`);
  process.exit(1);
});

setTimeout(() => {
  socket.destroy();
  process.exit(0);
}, HOLD_MS);
