import React, { useState, useEffect, useRef } from "react";
import { 
  Monitor, 
  Tv, 
  Terminal, 
  Settings, 
  Activity, 
  Copy, 
  Check, 
  RefreshCw, 
  ArrowRight, 
  Power, 
  ShieldAlert, 
  MousePointer, 
  SlidersHorizontal,
  Info,
  BookOpen,
  FileCode,
  FolderOpen,
  Globe
} from "lucide-react";
import VirtualDesktop from "./components/VirtualDesktop";
import { DeviceInfo, HostStatus, ViewerStatus, SessionMetrics, InputEvent } from "./types";

export default function App() {
  // Navigation tabs
  const [activeTab, setActiveTab] = useState<"workspace" | "host" | "viewer" | "specs">("workspace");

  // Host state
  const [hostStatus, setHostStatus] = useState<HostStatus>("idle");
  const [allowControl, setAllowControl] = useState(true);
  const [hostLogs, setHostLogs] = useState<string[]>(["[09:00:00] ArcLink 主机端服务已成功初始化。"]);
  const [hostIp, setHostIp] = useState("192.168.1.100");
  const [hostPort, setHostPort] = useState(8443);
  const [hostName, setHostName] = useState("WORKSTATION-WIN11");
  const [hostId, setHostId] = useState("ARC-HOST-7890");
  const [activeRequest, setActiveRequest] = useState<{
    sessionId: string;
    viewerName: string;
    viewerIp: string;
  } | null>(null);

  // Viewer state
  const [viewerStatus, setViewerStatus] = useState<ViewerStatus>("disconnected");
  const [targetIp, setTargetIp] = useState("192.168.1.100");
  const [targetPort, setTargetPort] = useState("8443");
  const [viewerRemark, setViewerRemark] = useState("我的工作站 A");
  const [viewerLogs, setViewerLogs] = useState<string[]>(["[09:00:00] ArcLink 控制端已成功初始化。"]);
  const [inputLocked, setInputLocked] = useState(false);
  const [qualityMode, setQualityMode] = useState("流畅 (60fps)");
  const [showPerfDrawer, setShowPerfDrawer] = useState(true);
  const [copiedText, setCopiedText] = useState(false);

  // Real-time telemetry states
  const [metrics, setMetrics] = useState<SessionMetrics>({
    latency_ms: 8.5,
    fps: 60,
    bitrate_kbps: 4500.5,
    resolution_width: 1920,
    resolution_height: 1080,
    active_duration_secs: 0,
    network: {
      rtt_ms: 1.5,
      jitter_ms: 0.2,
      packet_loss_rate: 0.0,
      sent_bytes_sec: 1524,
      rcv_bytes_sec: 582410
    }
  });

  // Chart values (sliding window for telemetry visualization)
  const [latencyHistory, setLatencyHistory] = useState<number[]>([8.5, 9.1, 8.2, 8.7, 7.9, 8.5, 8.8, 8.4, 9.2, 8.5]);
  const [fpsHistory, setFpsHistory] = useState<number[]>([60, 60, 59, 60, 60, 60, 58, 60, 60, 60]);

  // WebSocket reference for live cross-tab direct connectivity
  const wsRef = useRef<WebSocket | null>(null);
  const [isWsConnected, setIsWsConnected] = useState(false);

  // Latest screenshot frame stream (Data URL) from Host Virtual Desktop to Viewer App canvas
  const [latestFrame, setLatestFrame] = useState<string>("");
  // Input event buffer to inject into the Host desktop
  const [currentExternalInput, setCurrentExternalInput] = useState<any | null>(null);

  // Create simulated clock
  const [currentTime, setCurrentTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setCurrentTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  // Sync metrics simulation
  useEffect(() => {
    let timer: any;
    if (hostStatus === "occupied" && viewerStatus === "connected") {
      timer = setInterval(() => {
        setMetrics((prev) => {
          const randLat = +(prev.latency_ms + (Math.random() * 1.6 - 0.8)).toFixed(2);
          const randFps = Math.floor(prev.fps + (Math.random() * 4 - 2));
          const randKbps = +(prev.bitrate_kbps + (Math.random() * 100 - 50)).toFixed(1);
          
          // update charts
          setLatencyHistory((prevLat) => [...prevLat.slice(1), Math.max(2.0, randLat)]);
          setFpsHistory((prevFps) => [...prevFps.slice(1), Math.min(60, Math.max(30, randFps))]);

          return {
            ...prev,
            latency_ms: Math.max(2.0, randLat),
            fps: Math.min(60, Math.max(30, randFps)),
            bitrate_kbps: randKbps,
            active_duration_secs: prev.active_duration_secs + 1
          };
        });
      }, 1000);
    }
    return () => clearInterval(timer);
  }, [hostStatus, viewerStatus]);

  // Connect to backend WebSocket for real-time pairing
  useEffect(() => {
    const loc = window.location;
    const protocol = loc.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${loc.host}/api/remote-ws`;

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsWsConnected(true);
      // Auto register host as default to let standalone client work
      ws.send(JSON.stringify({
        type: "register-host",
        ip: hostIp,
        port: hostPort,
        name: hostName
      }));
      ws.send(JSON.stringify({ type: "register-viewer" }));
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        switch (msg.type) {
          case "registered":
            addHostLog(`设备注册成功。主机端正在运行于：${hostIp}:${hostPort}`);
            break;
          case "inbound-request":
            addHostLog(`收到来自 ${msg.viewerName} (${msg.viewerIp}) 的传入控制端配对请求`);
            setHostStatus("connecting");
            setActiveRequest({
              sessionId: msg.sessionId,
              viewerName: msg.viewerName,
              viewerIp: msg.viewerIp,
            });
            break;
          case "connection-accepted":
            addViewerLog(`主机端已接受远程控制请求。控制通道已成功建立！`);
            setViewerStatus("connected");
            break;
          case "connection-rejected":
            addViewerLog(`主机端拒绝配对申请。原因：${msg.reason}`);
            setViewerStatus("disconnected");
            break;
          case "render-frame":
            // Stream the JPEG frame data URL or svg code directly
            setLatestFrame(msg.frame);
            break;
          case "host-inject-input":
            // Trigger input on host's desktop
            setCurrentExternalInput(msg.event);
            break;
          case "disconnected":
            addHostLog(`配对设备已断开连接。`);
            addViewerLog(`远程连接终端主动终止了会话。`);
            setHostStatus("listening");
            setViewerStatus("disconnected");
            break;
        }
      } catch (err) {
        console.error("WS parse error:", err);
      }
    };

    ws.onclose = () => {
      setIsWsConnected(false);
    };

    return () => {
      ws.close();
    };
  }, []);

  const addHostLog = (msg: string) => {
    const time = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
    setHostLogs((prev) => [...prev, `[${time}] ${msg}`].slice(-15));
  };

  const addViewerLog = (msg: string) => {
    const time = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
    setViewerLogs((prev) => [...prev, `[${time}] ${msg}`].slice(-15));
  };

  // Action: Host clicks Accept Connection
  const handleHostAccept = () => {
    if (activeRequest) {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: "host-accept" }));
      }
      setHostStatus("occupied");
      addHostLog(`已向控制端 ${activeRequest.viewerIp} 授予远程控制特权`);
      setActiveRequest(null);
    }
  };

  // Action: Host clicks Reject Connection
  const handleHostReject = () => {
    if (activeRequest) {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: "host-reject", reason: "用户主动拒绝了连接申请。" }));
      }
      setHostStatus("listening");
      addHostLog(`已拒绝该远程控制接入申请。`);
      setActiveRequest(null);
    }
  };

  // Action: Viewer clicks Connect
  const handleViewerConnect = () => {
    setViewerStatus("connecting");
    addViewerLog(`正在尝试 Ping 目标主机 IP: ${targetIp}:${targetPort}...`);
    
    // Send socket request via express/ws
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: "viewer-connect",
        hostIp: targetIp,
        hostPort: targetPort,
        viewerIp: "192.168.1.150",
        viewerName: viewerRemark,
        sessionId: "SESSION-V542"
      }));
    } else {
      // Offline fallback: Simulation Mode
      setTimeout(() => {
        setHostStatus("connecting");
        setActiveRequest({
          sessionId: "SIM-8842",
          viewerName: viewerRemark,
          viewerIp: "192.168.1.150"
        });
      }, 600);
    }
  };

  // Action: Disconnect session
  const handleDisconnect = () => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type: "viewer-disconnect" }));
    }
    setViewerStatus("disconnected");
    setHostStatus("listening");
    addViewerLog("远程会话已被本地控制端主动断开。");
    addHostLog("远程会话已被控制端切断。");
  };

  // Capture screen frame from Host and send to Viewer via websocket
  const handleHostFrameCapture = (frameDataUrl: string) => {
    if (hostStatus === "occupied" && wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: "screen-frame",
        frame: frameDataUrl,
        metrics: metrics
      }));
    }
    // Set locally anyway for local loopback layout
    setLatestFrame(frameDataUrl);
  };

  // Capture pointer events on Viewer window and relay to Host
  const handleViewerPointerAction = (e: React.MouseEvent<HTMLDivElement>, type: string) => {
    if (viewerStatus !== "connected") return;

    const rect = e.currentTarget.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;

    const event: InputEvent = {
      type: type as any,
      x,
      y,
      button: e.button === 2 ? "right" : "left"
    };

    // Forward through websocket
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: "input-event",
        event
      }));
    } else {
      // Local loopback
      setCurrentExternalInput({
        type,
        x,
        y,
        button: e.button === 2 ? "right" : "left"
      });
    }
  };

  // Capture keydown events on Viewer and relay to Host
  const handleViewerKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (viewerStatus !== "connected" || inputLocked) return;
    
    // Prevent browser actions like backspace navigating
    e.preventDefault();

    const event: InputEvent = {
      type: "keydown",
      key: e.key,
      code: e.code,
      isDown: true
    } as any;

    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: "input-event",
        event
      }));
    } else {
      setCurrentExternalInput({
        type: "keydown",
        key: e.key,
        isDown: true
      });
    }
  };

  const copyConnectionInfo = () => {
    const text = `ArcLink Address: ${hostIp}:${hostPort} | Code: ${hostId}`;
    navigator.clipboard.writeText(text);
    setCopiedText(true);
    setTimeout(() => setCopiedText(false), 2000);
  };

  return (
    <div className="min-h-screen bg-[#E8EEF5] text-[#1D1D1F] flex flex-col antialiased relative overflow-hidden select-none font-sans">
      {/* Background Decorative Elements for 'Desktop' feel */}
      <div className="absolute top-[-20%] left-[-10%] w-[600px] h-[600px] bg-blue-200/40 rounded-full blur-[120px] pointer-events-none"></div>
      <div className="absolute bottom-[-10%] right-[-5%] w-[500px] h-[500px] bg-white/60 rounded-full blur-[100px] pointer-events-none"></div>

      {/* Liquid Glass Header */}
      <header className="sticky top-0 z-50 bg-white/40 backdrop-blur-3xl h-14 flex items-center justify-between px-6 border-b border-black/5 relative z-20">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-blue-500 flex items-center justify-center text-white font-semibold font-display shadow-lg shadow-blue-500/20">
            AL
          </div>
          <div>
            <h1 className="text-sm font-semibold tracking-tight font-display text-[#1D1D1F]">ArcLink 远程控制</h1>
            <p className="text-[10px] text-slate-500 font-mono">局域网极速远程桌面 MVP 操控台</p>
          </div>
        </div>

        {/* Real-time Indicator Panel */}
        <div className="flex items-center gap-4 text-xs font-medium">
          <div className="hidden sm:flex items-center gap-2 px-2.5 py-1 rounded-full bg-white/60 border border-white/40 shadow-sm">
            <span className={`w-1.5 h-1.5 rounded-full ${isWsConnected ? "bg-emerald-500 animate-pulse" : "bg-amber-400"}`} />
            <span className="text-[11px] text-slate-600 font-medium">
              {isWsConnected ? "Socket 双向信道已联机" : "本地独立模拟模式已激活"}
            </span>
          </div>
          <span className="text-slate-300">|</span>
          <div className="font-mono text-slate-600 bg-white/50 px-2.5 py-1 rounded-lg border border-white/40 text-[11px]">
            UTC {currentTime.toISOString().slice(11, 19)}
          </div>
        </div>
      </header>

      {/* Main Layout Workspace */}
      <div className="flex-1 flex flex-col lg:flex-row h-[calc(100vh-3.5rem)] overflow-hidden relative z-10">
        
        {/* Navigation Sidebar Drawer */}
        <nav className="w-full lg:w-56 bg-white/30 backdrop-blur-2xl border-r border-black/5 flex flex-row lg:flex-col justify-start lg:justify-between p-3 gap-1 overflow-x-auto lg:overflow-y-auto relative z-20">
          <div className="flex flex-row lg:flex-col gap-1 w-full">
            <button 
              onClick={() => setActiveTab("workspace")}
              className={`flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-xs font-semibold transition-all ${
                activeTab === "workspace" 
                  ? "bg-white/75 text-blue-600 border border-white/80 shadow-md shadow-slate-100" 
                  : "text-slate-600 hover:bg-white/40"
              }`}
            >
              <SlidersHorizontal className="w-4 h-4" />
              <span>双端联合工作区</span>
            </button>

            <button 
              onClick={() => setActiveTab("host")}
              className={`flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-xs font-semibold transition-all ${
                activeTab === "host" 
                  ? "bg-white/75 text-blue-600 border border-white/80 shadow-md shadow-slate-100" 
                  : "text-slate-600 hover:bg-white/40"
              }`}
            >
              <Tv className="w-4 h-4" />
              <span>ArcLink 被控端主机</span>
              {hostStatus === "occupied" && (
                <span className="ml-auto w-2 h-2 rounded-full bg-blue-500 animate-ping" />
              )}
            </button>

            <button 
              onClick={() => setActiveTab("viewer")}
              className={`flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-xs font-semibold transition-all ${
                activeTab === "viewer" 
                  ? "bg-white/75 text-blue-600 border border-white/80 shadow-md shadow-slate-100" 
                  : "text-slate-600 hover:bg-white/40"
              }`}
            >
              <Monitor className="w-4 h-4" />
              <span>ArcLink 主控端视图</span>
              {viewerStatus === "connected" && (
                <span className="ml-auto w-2 h-2 rounded-full bg-emerald-500" />
              )}
            </button>

            <div className="hidden lg:block my-2.5 border-t border-black/5" />

            <button 
              onClick={() => setActiveTab("specs")}
              className={`flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl text-xs font-semibold transition-all ${
                activeTab === "specs" 
                  ? "bg-white/75 text-blue-600 border border-white/80 shadow-md shadow-slate-100" 
                  : "text-slate-600 hover:bg-white/40"
              }`}
            >
              <FileCode className="w-4 h-4" />
              <span>Rust 项目源码结构</span>
            </button>
          </div>

          <div className="hidden lg:block text-[11px] text-slate-400 p-2.5 text-center bg-white/40 rounded-xl border border-white/40 font-mono">
            v1.0 演示版
          </div>
        </nav>

        {/* Primary Screen Stage Area */}
        <main className="flex-1 overflow-y-auto p-4 md:p-6 bg-transparent relative z-10">
          
          {/* TAB 1: CO-WORKSPACE PANEL */}
          {activeTab === "workspace" && (
            <div className="space-y-6">
              {/* Top Intro card */}
              <div className="editorial-card relative overflow-hidden">
                <div className="absolute right-[-20px] top-[-20px] w-40 h-40 bg-blue-100/30 rounded-full blur-3xl" />
                <h2 className="text-lg font-light tracking-tight text-[#1D1D1F] font-display">
                  ArcLink 双端全真画面及交互同步控制面板
                </h2>
                <p className="text-xs text-slate-600 max-w-2xl mt-2 leading-relaxed">
                  在此直接体验流畅的低延迟远程控制！在该双端联合控制台中，<strong>主机被控端 (Host PC)</strong> 与 <strong>主控端 (Viewer Client)</strong> 可以在此单页完美并排运行。您能够直接在右侧的控制端图像里点击、移动或使用键盘进行操作，并实时观测画笔笔触、资源管理器文件浏览以及诊断图表延迟指标的双向极速同步。
                </p>
              </div>

              {/* Grid with Host on Left, Viewer on Right */}
              <div className="grid grid-cols-1 xl:grid-cols-2 gap-6 items-start">
                
                {/* Simulated Host Widget Card */}
                <div className="glass-window flex flex-col">
                  {/* Window Title Bar */}
                  <div className="h-11 bg-white/40 flex items-center justify-between px-4 border-b border-black/5">
                    <div className="flex items-center gap-2">
                      <div className={`w-3 h-3 rounded-full flex items-center justify-center ${
                        hostStatus === "idle" ? "bg-slate-500/20" :
                        hostStatus === "listening" ? "bg-emerald-500/20 animate-pulse" :
                        hostStatus === "connecting" ? "bg-amber-500/20" : "bg-blue-500/20"
                      }`}>
                        <div className={`w-1.5 h-1.5 rounded-full ${
                          hostStatus === "idle" ? "bg-slate-500" :
                          hostStatus === "listening" ? "bg-emerald-500" :
                          hostStatus === "connecting" ? "bg-amber-500" : "bg-blue-500"
                        }`} />
                      </div>
                      <span className="text-[11px] font-bold uppercase tracking-widest text-slate-600 font-display">ArcLink 主机端 (Host)</span>
                    </div>
                    <div className="flex gap-4 opacity-40">
                      <div className="w-3 h-[1px] bg-black"></div>
                      <div className="w-3 h-3 border border-black rounded-[2px]"></div>
                      <div className="w-3 h-3 text-[10px] flex items-center justify-center font-bold">✕</div>
                    </div>
                  </div>

                  {/* Main content area of Window */}
                  <div className="p-6 flex flex-col gap-5 flex-1">
                    {/* Device Spec details inside window */}
                    <div className="bg-white/50 border border-white/60 rounded-xl p-4 space-y-4 shadow-sm">
                      <div className="grid grid-cols-2 gap-y-3 text-xs">
                        <div>
                          <span className="editorial-label">本地端点 Socket</span>
                          <p className="font-mono font-semibold text-[#1D1D1F] text-[13px]">{hostIp}:{hostPort}</p>
                        </div>
                        <div>
                          <span className="editorial-label">唯一配对 ID 标识符</span>
                          <p className="font-mono font-semibold text-[#1D1D1F] text-[13px]">{hostId}</p>
                        </div>
                      </div>

                      <div className="flex gap-2.5 pt-1">
                        <button 
                          onClick={copyConnectionInfo}
                          className="editorial-btn-secondary flex-1 py-2 flex items-center justify-center gap-1.5 text-xs font-semibold"
                        >
                          {copiedText ? <Check className="w-3.5 h-3.5 text-emerald-600" /> : <Copy className="w-3.5 h-3.5 text-slate-500" />}
                          <span>{copiedText ? "已复制" : "复制连接信息"}</span>
                        </button>

                        {hostStatus === "idle" ? (
                          <button 
                            onClick={() => {
                              setHostStatus("listening");
                              addHostLog("套接字服务绑定并监听成功。准备接收控制端会话请求。");
                            }}
                            className="editorial-btn flex-1 py-2 text-xs"
                          >
                            <Power className="w-3.5 h-3.5" />
                            <span>启动被控服务</span>
                          </button>
                        ) : (
                          <button 
                            onClick={() => {
                              setHostStatus("idle");
                              setViewerStatus("disconnected");
                              addHostLog("主机端被控服务已主动终止。");
                            }}
                            className="flex-1 flex items-center justify-center gap-1.5 py-2 rounded-xl text-xs font-semibold border border-rose-200 bg-rose-50/70 hover:bg-rose-100 text-rose-600 transition-all shadow-sm"
                          >
                            <Power className="w-3.5 h-3.5" />
                            <span>停止被控服务</span>
                          </button>
                        )}
                      </div>
                    </div>

                    {/* Incoming Handshake Consent Window popup */}
                    {hostStatus === "connecting" && activeRequest && (
                      <div className="bg-white/80 border border-blue-200 p-5 rounded-2xl relative overflow-hidden shadow-xl animate-in fade-in zoom-in-95 duration-200 ring-1 ring-blue-500/10">
                        <div className="absolute right-[-10px] top-[-10px] w-20 h-20 bg-blue-100/40 rounded-full blur-xl pointer-events-none" />
                        <div className="flex gap-3.5 relative z-10">
                          <div className="w-9 h-9 rounded-xl bg-blue-100/60 flex items-center justify-center text-blue-600 flex-shrink-0 border border-blue-200/50">
                            <ShieldAlert className="w-4.5 h-4.5" />
                          </div>
                          <div className="flex-1">
                            <h4 className="text-sm font-bold text-slate-800 font-display">收到新远程配对接入请求</h4>
                            <p className="text-[11px] text-slate-500 mt-0.5 leading-relaxed">
                              有来自远程控制台的连接申请。请仔细核对客户端认证标识：
                            </p>
                            <div className="mt-3 bg-white/60 border border-slate-100 p-3 rounded-xl text-[10px] font-mono space-y-1.5 shadow-inner">
                              <p><span className="text-slate-400">主控端名称：</span> {activeRequest.viewerName}</p>
                              <p><span className="text-slate-400">主控端 IP 地址：</span> {activeRequest.viewerIp}</p>
                              <p><span className="text-slate-400">连接类型：</span> 局域网直连 (LAN Direct Connection)</p>
                            </div>
                            <div className="flex gap-2.5 mt-4">
                              <button 
                                onClick={handleHostAccept}
                                className="editorial-btn py-2 text-xs font-semibold"
                              >
                                授予远程控制特权
                              </button>
                              <button 
                                onClick={handleHostReject}
                                className="editorial-btn-secondary py-2 text-xs font-semibold"
                              >
                                拒绝
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    )}

                    {/* Active screen viewer inside Host widget */}
                    <div className="space-y-2">
                      <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display">主机原生独立显示屏画面：</h4>
                      <div className="border border-white/50 rounded-xl overflow-hidden shadow-md">
                        <VirtualDesktop 
                          onFrameCapture={handleHostFrameCapture} 
                          externalInput={currentExternalInput}
                        />
                      </div>
                    </div>

                    {/* Host Logs Console */}
                    <div className="space-y-2 mt-2">
                      <div className="flex items-center justify-between">
                        <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display">主机端套接字及会话事件：</h4>
                        <button onClick={() => setHostLogs([])} className="text-[10px] text-slate-400 hover:text-blue-500 transition-colors font-mono uppercase tracking-wider">清空</button>
                      </div>
                      <div className="h-28 overflow-y-auto bg-[#1D1D1F] text-emerald-400 font-mono text-[10px] p-3.5 rounded-xl border border-white/10 mac-scrollbar shadow-inner leading-5">
                        {hostLogs.map((log, idx) => (
                          <div key={idx} className="opacity-90">{log}</div>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Simulated Viewer Widget Card */}
                <div className="glass-window flex flex-col">
                  {/* Window Title Bar */}
                  <div className="h-11 bg-white/40 flex items-center justify-between px-4 border-b border-black/5">
                    <div className="flex items-center gap-2">
                      <div className={`w-3 h-3 rounded-full flex items-center justify-center ${
                        viewerStatus === "disconnected" ? "bg-slate-500/20" :
                        viewerStatus === "connecting" ? "bg-amber-500/20 animate-pulse" :
                        "bg-emerald-500/20"
                      }`}>
                        <div className={`w-1.5 h-1.5 rounded-full ${
                          viewerStatus === "disconnected" ? "bg-slate-500" :
                          viewerStatus === "connecting" ? "bg-amber-500" :
                          "bg-emerald-500"
                        }`} />
                      </div>
                      <span className="text-[11px] font-bold uppercase tracking-widest text-slate-600 font-display">ArcLink 主控端 (Viewer)</span>
                    </div>
                    <div className="flex gap-4 opacity-40">
                      <div className="w-3 h-[1px] bg-black"></div>
                      <div className="w-3 h-3 border border-black rounded-[2px]"></div>
                      <div className="w-3 h-3 text-[10px] flex items-center justify-center font-bold">✕</div>
                    </div>
                  </div>

                  {/* Main Content Area */}
                  <div className="p-6 flex flex-col gap-5 flex-1">
                    {viewerStatus === "disconnected" ? (
                      /* Initial Dialer Menu */
                      <div className="bg-white/50 border border-white/60 rounded-xl p-5 space-y-4 shadow-sm">
                        <div className="space-y-1">
                          <h4 className="text-sm font-semibold text-slate-800 font-display">建立远程会话连接：</h4>
                          <p className="text-[11px] text-slate-500">请在下方输入正在运行中的主机端端点 IP 与端口：</p>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                          <div className="space-y-1.5">
                            <label className="editorial-label">主机 IP 地址</label>
                            <input 
                              type="text" 
                              value={targetIp} 
                              onChange={(e) => setTargetIp(e.target.value)}
                              className="editorial-input font-mono"
                            />
                          </div>
                          <div className="space-y-1.5">
                            <label className="editorial-label">主机监听端口</label>
                            <input 
                              type="number" 
                              value={targetPort} 
                              onChange={(e) => setTargetPort(e.target.value)}
                              className="editorial-input font-mono"
                            />
                          </div>
                        </div>

                        <div className="space-y-1.5">
                          <label className="editorial-label">主控端设备备注</label>
                          <input 
                            type="text" 
                            value={viewerRemark} 
                            onChange={(e) => setViewerRemark(e.target.value)}
                            className="editorial-input"
                          />
                        </div>

                        <button 
                          onClick={handleViewerConnect}
                          disabled={hostStatus === "idle"}
                          className={`w-full py-3 rounded-xl text-xs font-semibold font-display flex items-center justify-center gap-2 transition-all ${
                            hostStatus === "idle" 
                              ? "bg-slate-100/50 text-slate-400 cursor-not-allowed border border-slate-200/50" 
                              : "editorial-btn"
                          }`}
                        >
                          <span>连接到远程目标桌面</span>
                          <ArrowRight className="w-3.5 h-3.5" />
                        </button>

                        {hostStatus === "idle" && (
                          <div className="flex items-center gap-2 text-[10px] text-amber-600 bg-amber-50/60 p-3 rounded-xl border border-amber-200/30">
                            <Info className="w-4 h-4 flex-shrink-0" />
                            <span>请先在左侧被控端点击<strong>“启动被控服务”</strong>以部署配对侦听器。</span>
                          </div>
                        )}
                      </div>
                    ) : viewerStatus === "connecting" ? (
                      /* Waiting Spinner */
                      <div className="bg-white/50 border border-white/60 rounded-xl p-8 flex flex-col items-center justify-center text-center gap-4 shadow-sm">
                        <div className="w-10 h-10 rounded-full border-2 border-blue-100 border-t-blue-500 animate-spin" />
                        <div className="space-y-1">
                          <h4 className="text-xs font-semibold text-slate-800 font-display uppercase tracking-wider">正在传输会话握手数据包</h4>
                          <p className="text-[10px] text-slate-500">正在等待被控端主机的确认同意...</p>
                        </div>
                        <div className="flex gap-2.5">
                          <button 
                            onClick={handleHostAccept}
                            className="px-3.5 py-2 rounded-xl bg-blue-50 text-blue-600 border border-blue-100 hover:bg-blue-100/70 text-[10px] font-bold transition-all shadow-sm"
                          >
                            模拟主机端同意接入
                          </button>
                          <button 
                            onClick={() => setViewerStatus("disconnected")}
                            className="px-3.5 py-2 rounded-xl bg-slate-100 hover:bg-slate-200 text-slate-700 text-[10px] font-bold transition-all border border-slate-200/50 shadow-sm"
                          >
                            取消
                          </button>
                        </div>
                      </div>
                    ) : (
                      /* Viewer Connected Screen */
                      <div className="space-y-4">
                        {/* Top Toolbar panel */}
                        <div className="bg-[#1D1D1F] text-white rounded-xl p-3 flex items-center justify-between text-xs font-mono border border-white/10 shadow-md">
                          <div className="flex items-center gap-2">
                            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
                            <span className="font-bold text-slate-200">{viewerRemark}</span>
                          </div>
                          <div className="flex items-center gap-3 text-[10px] opacity-90">
                            <span className="text-emerald-400">往返延迟: {metrics.network.rtt_ms} ms</span>
                            <span className="text-blue-400">帧率: {metrics.fps}</span>
                            <span className="text-purple-400">带宽: {metrics.bitrate_kbps} kbps</span>
                          </div>
                          <button 
                            onClick={handleDisconnect}
                            className="px-2.5 py-1 rounded-lg bg-rose-600 hover:bg-rose-700 text-white text-[10px] font-sans font-bold transition-colors shadow-sm"
                          >
                            断开会话
                          </button>
                        </div>

                        {/* Interactive Control Screen Frame */}
                        <div className="space-y-1.5">
                          <p className="text-[10px] text-slate-500 uppercase tracking-widest font-bold">
                            交互式控制窗口。在此窗口内点击或使用键盘输入即可发送操作指令：
                          </p>
                          
                          <div 
                            onMouseMove={(e) => handleViewerPointerAction(e, "mousemove")}
                            onMouseDown={(e) => handleViewerPointerAction(e, "mousedown")}
                            onMouseUp={(e) => handleViewerPointerAction(e, "mouseup")}
                            onKeyDown={handleViewerKeyDown}
                            tabIndex={0}
                            className="relative border border-slate-300 rounded-xl overflow-hidden cursor-crosshair focus:ring-2 focus:ring-blue-500 outline-none select-none shadow-lg"
                          >
                            {/* We draw the synced frame here */}
                            {latestFrame ? (
                              <img 
                                src={latestFrame} 
                                alt="Remote Screen" 
                                referrerPolicy="no-referrer"
                                className="w-full h-auto block object-contain aspect-video"
                              />
                            ) : (
                              <div className="aspect-video w-full bg-slate-950 flex flex-col items-center justify-center text-slate-500 gap-2">
                                <RefreshCw className="w-6 h-6 animate-spin text-blue-500" />
                                <span className="text-xs">正在初始化桌面画面解码数据流管道...</span>
                              </div>
                            )}
                          </div>
                        </div>

                        {/* Diagnostic Graphs */}
                        <div className="grid grid-cols-2 gap-3.5">
                          <div className="bg-white/50 border border-white/60 p-3.5 rounded-xl shadow-sm">
                            <span className="editorial-label">控制输入延迟 (ms)</span>
                            <div className="text-sm font-bold font-mono text-blue-600 mt-1">{metrics.latency_ms} ms</div>
                            <div className="flex items-end h-8 gap-[3px] mt-2">
                              {latencyHistory.map((lat, idx) => (
                                <div 
                                  key={idx} 
                                  style={{ height: `${(lat / 15) * 100}%` }}
                                  className="bg-blue-400/60 hover:bg-blue-500 w-full rounded-sm transition-all"
                                />
                              ))}
                            </div>
                          </div>

                          <div className="bg-white/50 border border-white/60 p-3.5 rounded-xl shadow-sm">
                            <span className="editorial-label">画面解码帧率</span>
                            <div className="text-sm font-bold font-mono text-emerald-600 mt-1">{metrics.fps} FPS</div>
                            <div className="flex items-end h-8 gap-[3px] mt-2">
                              {fpsHistory.map((fps, idx) => (
                                <div 
                                  key={idx} 
                                  style={{ height: `${(fps / 60) * 100}%` }}
                                  className="bg-emerald-400/60 hover:bg-emerald-500 w-full rounded-sm transition-all"
                                />
                              ))}
                            </div>
                          </div>
                        </div>
                      </div>
                    )}

                    {/* Viewer Logs Console */}
                    <div className="space-y-2 mt-2">
                      <div className="flex items-center justify-between">
                        <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display">控制端会话与诊断日志：</h4>
                        <button onClick={() => setViewerLogs([])} className="text-[10px] text-slate-400 hover:text-blue-500 transition-colors font-mono uppercase tracking-wider">清空</button>
                      </div>
                      <div className="h-28 overflow-y-auto bg-[#1D1D1F] text-sky-400 font-mono text-[10px] p-3.5 rounded-xl border border-white/10 mac-scrollbar shadow-inner leading-5">
                        {viewerLogs.map((log, idx) => (
                          <div key={idx} className="opacity-90">{log}</div>
                        ))}
                      </div>
                    </div>
                  </div>
                </div>

              </div>
            </div>
          )}

          {/* TAB 2: ISOLATED ARC-LINK HOST VIEW */}
          {activeTab === "host" && (
            <div className="max-w-4xl mx-auto space-y-6">
              <div className="editorial-card flex flex-col md:flex-row justify-between items-start md:items-center gap-4 relative overflow-hidden">
                <div className="absolute right-[-20px] top-[-20px] w-36 h-36 bg-blue-100/20 rounded-full blur-2xl pointer-events-none" />
                <div className="flex items-center gap-3 relative z-10">
                  <div className="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center text-blue-600 border border-blue-100">
                    <Tv className="w-5 h-5" />
                  </div>
                  <div>
                    <h2 className="text-base font-semibold text-[#1D1D1F] font-display">ArcLink 被控端主机运行模式</h2>
                    <p className="text-xs text-slate-500">此窗口模拟真实 Windows 被控端桌面的画面捕获与指令接收。</p>
                  </div>
                </div>

                <div className="flex items-center gap-2 relative z-10 bg-white/60 px-3 py-1.5 rounded-xl border border-slate-100">
                  <span className={`w-2 h-2 rounded-full ${hostStatus === "occupied" ? "bg-blue-500 animate-pulse" : "bg-emerald-400"}`} />
                  <span className="text-[11px] font-bold uppercase tracking-wider text-slate-600 font-mono">
                    {hostStatus === "occupied" ? "远程控制端已接入" : "等待控制端配对接入"}
                  </span>
                </div>
              </div>

              <div className="glass-window flex flex-col">
                {/* Window title bar */}
                <div className="h-11 bg-white/40 flex items-center justify-between px-4 border-b border-black/5">
                  <span className="text-[10px] font-bold uppercase tracking-widest text-slate-500 font-mono">虚拟 Windows 被控桌面容器</span>
                  <div className="flex gap-4 opacity-40">
                    <div className="w-3 h-[1px] bg-black"></div>
                    <div className="w-3 h-3 border border-black rounded-[2px]"></div>
                    <div className="w-3 h-3 text-[10px] flex items-center justify-center font-bold">✕</div>
                  </div>
                </div>

                <div className="p-6 space-y-6">
                  <div className="border border-white/50 rounded-xl overflow-hidden shadow-md">
                    <VirtualDesktop 
                      onFrameCapture={handleHostFrameCapture} 
                      externalInput={currentExternalInput}
                    />
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
                    <div className="p-4 bg-white/40 rounded-xl border border-white/60 shadow-sm">
                      <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display mb-3">端口监听套接字绑定 (Socket Listeners)</h3>
                      <div className="space-y-2 text-xs text-slate-600 font-mono">
                        <p><span className="text-slate-400 block text-[10px] uppercase font-bold tracking-wider">TCP 连接握手控制端口</span> <span className="font-bold text-slate-800">{hostIp}:{hostPort}</span></p>
                        <p><span className="text-slate-400 block text-[10px] uppercase font-bold tracking-wider">UDP 画面传输数据流通道</span> <span className="font-bold text-slate-800">127.0.0.1:8444</span></p>
                        <p><span className="text-slate-400 block text-[10px] uppercase font-bold tracking-wider">UDP 操控输入指令通道</span> <span className="font-bold text-slate-800">127.0.0.1:8445</span></p>
                      </div>
                    </div>

                    <div className="p-4 bg-white/40 rounded-xl border border-white/60 shadow-sm flex flex-col justify-between">
                      <div>
                        <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display mb-1">活动控制端连接凭据</h3>
                        <p className="text-[11px] text-slate-500">仅接受经过验证的局域网客户端设备。</p>
                      </div>
                      {hostStatus === "occupied" ? (
                        <button 
                          onClick={() => {
                            setHostStatus("listening");
                            setViewerStatus("disconnected");
                          }}
                          className="w-full mt-4 py-2 text-xs font-semibold text-rose-600 bg-rose-50/80 hover:bg-rose-100 border border-rose-200/50 rounded-xl transition-all shadow-sm"
                        >
                          强行切断远程控制通道
                        </button>
                      ) : (
                        <p className="text-xs text-slate-400 italic mt-4">当前暂无主控端注册接入。</p>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: ISOLATED ARC-LINK VIEWER */}
          {activeTab === "viewer" && (
            <div className="max-w-4xl mx-auto space-y-6">
              <div className="editorial-card flex items-center justify-between relative overflow-hidden">
                <div className="absolute right-[-20px] top-[-20px] w-36 h-36 bg-blue-100/20 rounded-full blur-2xl pointer-events-none" />
                <div className="flex items-center gap-3 relative z-10">
                  <div className="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center text-blue-600 border border-blue-100">
                    <Monitor className="w-5 h-5" />
                  </div>
                  <div>
                    <h2 className="text-base font-semibold text-[#1D1D1F] font-display">ArcLink 主控端视图运行模式</h2>
                    <p className="text-xs text-slate-500">远程高精控制端专属客户端操控界面。</p>
                  </div>
                </div>

                {viewerStatus === "connected" && (
                  <button 
                    onClick={handleDisconnect}
                    className="px-4 py-2 rounded-xl border border-rose-200 bg-rose-50/80 text-rose-600 text-xs font-bold hover:bg-rose-100 transition-all shadow-sm relative z-10"
                  >
                    切断连接
                  </button>
                )}
              </div>

              {viewerStatus !== "connected" ? (
                <div className="bg-white/50 border border-white/60 rounded-2xl p-8 max-w-md mx-auto text-center space-y-6 shadow-md">
                  <div className="w-12 h-12 rounded-2xl bg-blue-50 flex items-center justify-center text-blue-600 mx-auto border border-blue-100">
                    <Monitor className="w-6 h-6" />
                  </div>
                  <div className="space-y-1.5">
                    <h3 className="text-sm font-bold text-[#1D1D1F] font-display">启动远程桌面控制通道</h3>
                    <p className="text-xs text-slate-500">在下方输入目标 ArcLink 被控主机的局域网端点地址即可建立极速控制链路。</p>
                  </div>

                  <div className="space-y-3 text-left">
                    <div className="space-y-1.5">
                      <label className="editorial-label">被控主机端点 Socket</label>
                      <input 
                        type="text" 
                        value={`${targetIp}:${targetPort}`}
                        disabled
                        className="editorial-input font-mono bg-slate-100/50 text-slate-400 cursor-not-allowed border-dashed"
                      />
                    </div>
                  </div>

                  <button 
                    onClick={handleViewerConnect}
                    className="w-full py-3 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-semibold font-display text-xs shadow-md transition-all flex items-center justify-center gap-1.5"
                  >
                    <span>连接目标设备</span>
                    <ArrowRight className="w-4 h-4" />
                  </button>
                </div>
              ) : (
                <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 items-start">
                  {/* Video render column */}
                  <div className="lg:col-span-3 bg-slate-950 p-4 rounded-2xl border border-slate-800 space-y-4 shadow-xl">
                    <div className="flex items-center justify-between text-xs font-mono text-slate-400">
                      <span>已映射的远程桌面画幅 (1920x1080)</span>
                      <span className="text-emerald-400">控制延迟: {metrics.latency_ms}ms</span>
                    </div>

                    <div 
                      onMouseMove={(e) => handleViewerPointerAction(e, "mousemove")}
                      onMouseDown={(e) => handleViewerPointerAction(e, "mousedown")}
                      onMouseUp={(e) => handleViewerPointerAction(e, "mouseup")}
                      onKeyDown={handleViewerKeyDown}
                      tabIndex={0}
                      className="relative rounded-xl overflow-hidden border border-slate-800 cursor-crosshair focus:ring-2 focus:ring-blue-500 outline-none select-none"
                    >
                      {latestFrame ? (
                        <img 
                          src={latestFrame} 
                          alt="Remote Screen" 
                          referrerPolicy="no-referrer"
                          className="w-full h-auto block object-contain aspect-video"
                        />
                      ) : (
                        <div className="aspect-video w-full bg-slate-950 flex flex-col items-center justify-center text-slate-500 gap-2">
                          <RefreshCw className="w-5 h-5 animate-spin text-blue-500" />
                          <span className="text-xs">正在建立流媒体解码通道...</span>
                        </div>
                      )}
                    </div>

                    {/* Bottom controls panel toolbar */}
                    <div className="bg-[#1D1D1F] border border-white/5 p-3 rounded-xl flex items-center justify-between gap-2 text-slate-200">
                      <div className="flex gap-2">
                        <button 
                          onClick={() => setInputLocked(!inputLocked)} 
                          className={`px-3 py-1.5 rounded-lg text-xs font-semibold font-sans transition-all ${
                            inputLocked ? "bg-rose-600 text-white" : "bg-white/10 hover:bg-white/15 text-slate-300"
                          }`}
                        >
                          {inputLocked ? "开启输入拦截发送" : "拦截并拦截按键发送"}
                        </button>
                      </div>

                      <div className="flex items-center gap-2 text-xs font-sans">
                        <span className="text-slate-400">画面清晰度:</span>
                        <select 
                          value={qualityMode} 
                          onChange={(e) => setQualityMode(e.target.value)}
                          className="bg-white/10 text-slate-200 px-2.5 py-1.5 rounded-lg text-xs border border-white/5 focus:outline-none"
                        >
                          <option className="bg-[#1D1D1F]">极速流畅 (60fps)</option>
                          <option className="bg-[#1D1D1F]">视网膜超清</option>
                          <option className="bg-[#1D1D1F]">省流低带宽模式</option>
                        </select>
                      </div>
                    </div>
                  </div>

                  {/* Right hand diagnostic sidebar drawer */}
                  {showPerfDrawer && (
                    <div className="bg-white/50 border border-white/60 rounded-2xl p-5 space-y-6 shadow-sm">
                      <div className="border-b border-black/5 pb-3">
                        <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display">诊断控制台面板</h3>
                        <p className="text-[10px] text-slate-500 mt-0.5">局域网实时遥测网络参数</p>
                      </div>

                      <div className="space-y-4">
                        <div className="space-y-1">
                          <span className="text-[10px] font-semibold text-slate-400">往返网络时延 (RTT)</span>
                          <div className="text-lg font-bold font-mono text-blue-600">{metrics.network.rtt_ms} ms</div>
                          <p className="text-[10px] text-slate-500">极佳局域网传输上限: &lt;2ms</p>
                        </div>

                        <div className="space-y-1">
                          <span className="text-[10px] font-semibold text-slate-400">桌面画面重绘率</span>
                          <div className="text-lg font-bold font-mono text-emerald-600">{metrics.fps} FPS</div>
                        </div>

                        <div className="space-y-1">
                          <span className="text-[10px] font-semibold text-slate-400">流媒体编解码码率</span>
                          <div className="text-lg font-bold font-mono text-purple-600">{(metrics.bitrate_kbps / 1000).toFixed(2)} Mbps</div>
                        </div>

                        <div className="space-y-1">
                          <span className="text-[10px] font-semibold text-slate-400">网络丢包率</span>
                          <div className="text-lg font-bold font-mono text-slate-700">0.000%</div>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

          {/* TAB 4: RUST PROJECT FILES AND DOCUMENTATION */}
          {activeTab === "specs" && (
            <div className="max-w-4xl mx-auto space-y-6">
              <div className="editorial-card relative overflow-hidden">
                <div className="absolute right-[-20px] top-[-20px] w-36 h-36 bg-blue-100/20 rounded-full blur-2xl pointer-events-none" />
                <div className="flex items-center gap-3 relative z-10">
                  <div className="w-10 h-10 rounded-xl bg-blue-50 flex items-center justify-center text-blue-600 border border-blue-100">
                    <FolderOpen className="w-5 h-5" />
                  </div>
                  <div>
                    <h2 className="text-base font-semibold text-[#1D1D1F] font-display">底层原生 Rust 工作区工程结构</h2>
                    <p className="text-xs text-slate-500">我们已经在该工作区中生成了标准、可直接编译的完整高性能 Rust 项目目录。</p>
                  </div>
                </div>
              </div>

              {/* Layout Files Grid */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                
                {/* File list */}
                <div className="bg-white/50 border border-white/60 rounded-2xl p-5 shadow-sm space-y-4">
                  <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display border-b border-black/5 pb-2">多包工程结构树 (Cargo Workspace)</h3>
                  
                  <div className="space-y-2.5 text-xs font-mono text-slate-600 leading-relaxed">
                    <div className="font-bold text-slate-800">📂 arclink-remote/</div>
                    <div className="pl-4">📄 Cargo.toml</div>
                    <div className="pl-4">📄 README.md</div>
                    <div className="pl-4 font-bold text-slate-800">📂 crates/</div>
                    <div className="pl-8">📂 arclink-common/</div>
                    <div className="pl-12 text-slate-500">└─ src/lib.rs (数据协议定义)</div>
                    <div className="pl-8">📂 arclink-host/</div>
                    <div className="pl-12 text-slate-500">└─ src/main.rs (画面采集与指令接收)</div>
                    <div className="pl-8">📂 arclink-viewer/</div>
                    <div className="pl-12 text-slate-500">└─ src/main.rs (界面解密与渲染客户端)</div>
                    <div className="pl-8">📂 arclink-protocol-test/</div>
                    <div className="pl-12 text-slate-500">└─ src/lib.rs (协议基准测试)</div>
                    <div className="pl-4 font-bold text-slate-800">📂 docs/</div>
                    <div className="pl-8 text-slate-500">📄 ARCHITECTURE.md (架构细节)</div>
                    <div className="pl-8 text-slate-500">📄 PROTOCOL.md (底层自定义包格式)</div>
                    <div className="pl-8 text-slate-500">📄 PERFORMANCE_PLAN.md (性能调优)</div>
                    <div className="pl-8 text-slate-500">📄 ROADMAP.md (第一阶段技术路线)</div>
                  </div>
                </div>

                {/* Compilation commands details */}
                <div className="bg-white/50 border border-white/60 rounded-2xl p-5 shadow-sm space-y-4 md:col-span-2">
                  <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider font-display border-b border-black/5 pb-2">本地 Windows 环境一键编译与部署指南</h3>
                  
                  <div className="space-y-4 text-xs">
                    <div className="space-y-1.5">
                      <span className="font-bold text-slate-800 block">1. 编译全部 Rust 目标程序：</span>
                      <pre className="bg-[#1D1D1F] text-slate-300 p-3 rounded-xl font-mono text-[11px] overflow-x-auto shadow-inner leading-relaxed">
                        cargo build --release
                      </pre>
                    </div>

                    <div className="space-y-1.5">
                      <span className="font-bold text-slate-800 block">2. 启动被控端电脑 A 上的 Host 程序：</span>
                      <pre className="bg-[#1D1D1F] text-slate-300 p-3 rounded-xl font-mono text-[11px] overflow-x-auto shadow-inner leading-relaxed">
                        cargo run --bin arclink-host
                      </pre>
                    </div>

                    <div className="space-y-1.5">
                      <span className="font-bold text-slate-800 block">3. 启动主控端电脑 B 上的 Viewer 软件：</span>
                      <pre className="bg-[#1D1D1F] text-slate-300 p-3 rounded-xl font-mono text-[11px] overflow-x-auto shadow-inner leading-relaxed">
                        cargo run --bin arclink-viewer
                      </pre>
                    </div>

                    <div className="space-y-1.5">
                      <span className="font-bold text-slate-800 block">4. 执行局域网远程协议环回测试集：</span>
                      <pre className="bg-[#1D1D1F] text-slate-300 p-3 rounded-xl font-mono text-[11px] overflow-x-auto shadow-inner leading-relaxed">
                        cargo test -p arclink-protocol-test
                      </pre>
                    </div>

                    <div className="flex items-start gap-3 text-amber-800 bg-amber-50/60 p-3.5 rounded-xl border border-amber-200/40 mt-3 shadow-sm">
                      <Info className="w-4 h-4 flex-shrink-0 mt-0.5 text-amber-600" />
                      <div>
                        <p className="font-bold">系统编译环境要求：必须具备 Windows SDK</p>
                        <p className="text-[11px] mt-0.5 leading-relaxed text-amber-900/80">
                          为了确保能成功编译，请确保您的被控 Windows 电脑上安装了 C++ 编译链。
                          本项目底层采用了高帧率 Direct3D11 截屏技术与 Windows 键盘/鼠标虚拟化注入接口。
                        </p>
                      </div>
                    </div>
                  </div>
                </div>

              </div>
            </div>
          )}

        </main>
      </div>
    </div>
  );
}
