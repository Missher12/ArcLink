import express from "express";
import path from "path";
import { createServer } from "http";
import { WebSocketServer, WebSocket } from "ws";
import { createServer as createViteServer } from "vite";

interface HostSession {
  hostSocket: WebSocket;
  viewerSocket: WebSocket | null;
  ip: string;
  port: string;
  name: string;
  status: "idle" | "connecting" | "occupied";
}

// Global sessions registry
const activeHosts = new Map<string, HostSession>();

async function startServer() {
  const app = express();
  const server = createServer(app);
  const wss = new WebSocketServer({ noServer: true });
  const PORT = 3000;

  app.use(express.json());

  // API Route: Get all active host sessions
  app.get("/api/hosts", (req, res) => {
    const list = Array.from(activeHosts.entries()).map(([id, session]) => ({
      id,
      ip: session.ip,
      port: session.port,
      name: session.name,
      status: session.status,
    }));
    res.json(list);
  });

  // Handle WebSocket Connection Upgrades
  server.on("upgrade", (request, socket, head) => {
    const { pathname } = new URL(request.url || "", `http://${request.headers.host}`);
    
    if (pathname === "/api/remote-ws") {
      wss.handleUpgrade(request, socket, head, (ws) => {
        wss.emit("connection", ws, request);
      });
    } else {
      socket.destroy();
    }
  });

  // WebSocket Server logic
  wss.on("connection", (ws: WebSocket) => {
    let clientType: "host" | "viewer" | null = null;
    let registeredId: string | null = null;
    let pairedId: string | null = null;

    ws.on("message", (message) => {
      try {
        const data = JSON.parse(message.toString());
        const type = data.type;

        switch (type) {
          case "register-host": {
            clientType = "host";
            registeredId = `${data.ip}:${data.port}`;
            
            // Clean up existing if any
            if (activeHosts.has(registeredId)) {
              try { activeHosts.get(registeredId)?.hostSocket.close(); } catch (e) {}
            }

            activeHosts.set(registeredId, {
              hostSocket: ws,
              viewerSocket: null,
              ip: data.ip,
              port: data.port,
              name: data.name || "Unknown Device",
              status: "idle",
            });

            ws.send(JSON.stringify({ type: "registered", id: registeredId }));
            broadcastHosts();
            break;
          }

          case "register-viewer": {
            clientType = "viewer";
            break;
          }

          case "viewer-connect": {
            const targetId = `${data.hostIp}:${data.hostPort}`;
            const hostSession = activeHosts.get(targetId);

            if (!hostSession) {
              ws.send(JSON.stringify({ type: "connect-error", message: "Host not found or offline." }));
              return;
            }

            if (hostSession.status === "occupied") {
              ws.send(JSON.stringify({ type: "connect-error", message: "Host is currently busy controlling another session." }));
              return;
            }

            // Lock session state to connecting
            hostSession.status = "connecting";
            hostSession.viewerSocket = ws;
            pairedId = targetId;

            // Forward connection request to the host
            hostSession.hostSocket.send(JSON.stringify({
              type: "inbound-request",
              viewerIp: data.viewerIp || "127.0.0.1",
              viewerName: data.viewerName || "Viewer Client",
              sessionId: data.sessionId,
            }));

            broadcastHosts();
            break;
          }

          case "host-accept": {
            if (clientType !== "host" || !registeredId) return;
            const session = activeHosts.get(registeredId);
            if (session && session.viewerSocket) {
              session.status = "occupied";
              session.viewerSocket.send(JSON.stringify({
                type: "connection-accepted",
                hostName: session.name,
              }));
              broadcastHosts();
            }
            break;
          }

          case "host-reject": {
            if (clientType !== "host" || !registeredId) return;
            const session = activeHosts.get(registeredId);
            if (session && session.viewerSocket) {
              session.status = "idle";
              session.viewerSocket.send(JSON.stringify({
                type: "connection-rejected",
                reason: data.reason || "Rejected by Host user.",
              }));
              session.viewerSocket = null;
              broadcastHosts();
            }
            break;
          }

          case "viewer-disconnect":
          case "host-disconnect": {
            handleSessionClose();
            break;
          }

          // Forward mouse and keyboard events from Viewer to Host
          case "input-event": {
            if (clientType !== "viewer" || !pairedId) return;
            const session = activeHosts.get(pairedId);
            if (session && session.hostSocket && session.status === "occupied") {
              session.hostSocket.send(JSON.stringify({
                type: "host-inject-input",
                event: data.event,
              }));
            }
            break;
          }

          // Forward screen frame stream from Host to Viewer
          case "screen-frame": {
            if (clientType !== "host" || !registeredId) return;
            const session = activeHosts.get(registeredId);
            if (session && session.viewerSocket && session.status === "occupied") {
              session.viewerSocket.send(JSON.stringify({
                type: "render-frame",
                frame: data.frame,
                metrics: data.metrics,
              }));
            }
            break;
          }

          // Relay stats or heartbeats
          case "ping": {
            ws.send(JSON.stringify({ type: "pong", timestamp: Date.now() }));
            break;
          }
        }
      } catch (err) {
        console.error("Error processing websocket message:", err);
      }
    });

    const handleSessionClose = () => {
      if (clientType === "host" && registeredId) {
        const session = activeHosts.get(registeredId);
        if (session) {
          if (session.viewerSocket) {
            session.viewerSocket.send(JSON.stringify({ type: "disconnected", reason: "Host disconnected." }));
          }
          activeHosts.delete(registeredId);
          broadcastHosts();
        }
      } else if (clientType === "viewer" && pairedId) {
        const session = activeHosts.get(pairedId);
        if (session) {
          session.status = "idle";
          session.viewerSocket = null;
          if (session.hostSocket) {
            session.hostSocket.send(JSON.stringify({ type: "disconnected", reason: "Viewer disconnected." }));
          }
          broadcastHosts();
        }
        pairedId = null;
      }
    };

    ws.on("close", () => {
      handleSessionClose();
    });

    ws.on("error", () => {
      handleSessionClose();
    });
  });

  // Helper: Broadcast active hosts list to all viewers
  function broadcastHosts() {
    const list = Array.from(activeHosts.entries()).map(([id, session]) => ({
      id,
      ip: session.ip,
      port: session.port,
      name: session.name,
      status: session.status,
    }));
    
    wss.clients.forEach((client) => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(JSON.stringify({ type: "hosts-list", list }));
      }
    });
  }

  // Vite middleware for development
  if (process.env.NODE_ENV !== "production") {
    const vite = await createViteServer({
      server: { middlewareMode: true },
      appType: "spa",
    });
    app.use(vite.middlewares);
  } else {
    const distPath = path.join(process.cwd(), "dist");
    app.use(express.static(distPath));
    app.get("*", (req, res) => {
      res.sendFile(path.join(distPath, "index.html"));
    });
  }

  server.listen(PORT, "0.0.0.0", () => {
    console.log(`[ArcLink Server] Running on http://localhost:${PORT}`);
  });
}

startServer();
