import React, { useRef, useEffect, useState } from "react";

interface FileItem {
  name: string;
  type: "folder" | "file";
  size?: string;
}

interface VirtualDesktopProps {
  onFrameCapture?: (frameData: string) => void;
  // External inputs from the WebSocket / controller
  externalInput?: {
    type: string;
    x: number;
    y: number;
    button?: string;
    key?: string;
    isDown?: boolean;
  } | null;
}

export default function VirtualDesktop({ onFrameCapture, externalInput }: VirtualDesktopProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  
  // Desktop OS state
  const [activeWindow, setActiveWindow] = useState<string | null>("painter");
  const [winPos, setWinPos] = useState({ x: 80, y: 60, w: 420, h: 320 });
  const [paintColor, setPaintColor] = useState("#3B82F6"); // default blue brush
  const [brushSize, setBrushSize] = useState(5);
  const [notepadText, setNotepadText] = useState("从控制端键盘输入内容...\n\n支持的热键：\n- 按下 [Backspace] 退格键删除字符\n- 按下任意英文字母或数字输入");
  const [currentFolder, setCurrentFolder] = useState<string>("根目录");
  const [explorerHistory, setExplorerHistory] = useState<string[]>(["根目录"]);

  // Paint drawings state
  const drawingsRef = useRef<{ x: number; y: number; color: string; size: number; drag: boolean }[]>([]);
  // Last virtual cursor coords in remote screen resolution (1920x1080)
  const [cursorX, setCursorX] = useState(960);
  const [cursorY, setCursorY] = useState(540);
  const [isMouseDown, setIsMouseDown] = useState(false);

  // Hardcoded simulated files
  const files: Record<string, FileItem[]> = {
    "根目录": [
      { name: "我的文档", type: "folder" },
      { name: "系统配置System64", type: "folder" },
      { name: "ArcLink_配置.toml", type: "file", size: "1.2 KB" },
      { name: "性能日志.txt", type: "file", size: "45 KB" }
    ],
    "我的文档": [
      { name: "设计草稿", type: "folder" },
      { name: "远程会话私钥.pem", type: "file", size: "3.2 KB" },
      { name: "备忘录.md", type: "file", size: "124 B" }
    ],
    "设计草稿": [
      { name: "架构设计草图.png", type: "file", size: "4.2 MB" }
    ],
    "系统配置System64": [
      { name: "windows-rs-绑定.dll", type: "file", size: "8.9 MB" },
      { name: "dxgi_屏幕捕获器.exe", type: "file", size: "1.4 MB" }
    ]
  };

  // Run the render loop for the Windows 11 Desktop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animFrameId: number;

    const render = () => {
      const w = canvas.width;
      const h = canvas.height;

      // 1. Draw modern Windows 11 glass-tinted wallpaper background
      const gradient = ctx.createRadialGradient(w / 2, h / 2, 50, w / 2, h / 2, w * 0.8);
      gradient.addColorStop(0, "#2563EB"); // Royal blue core
      gradient.addColorStop(0.5, "#1E3A8A"); // Midnight deep blue
      gradient.addColorStop(1, "#0F172A"); // Slate dark background
      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, w, h);

      // Draw light decorative wave lines on desktop background (Mica style)
      ctx.beginPath();
      ctx.strokeStyle = "rgba(147, 197, 253, 0.08)";
      ctx.lineWidth = 4;
      ctx.moveTo(0, h * 0.3);
      ctx.bezierCurveTo(w * 0.3, h * 0.1, w * 0.6, h * 0.7, w, h * 0.4);
      ctx.stroke();

      ctx.beginPath();
      ctx.strokeStyle = "rgba(147, 197, 253, 0.04)";
      ctx.moveTo(0, h * 0.6);
      ctx.bezierCurveTo(w * 0.4, h * 0.9, w * 0.7, h * 0.2, w, h * 0.7);
      ctx.stroke();

      // 2. Render Desktop Shortcuts Icons
      const icons = [
        { id: "painter", name: "Arc画图工具", color: "#EC4899", icon: "🎨" },
        { id: "notepad", name: "备忘录记事本", color: "#F59E0B", icon: "📝" },
        { id: "explorer", name: "文件资源管理器", color: "#3B82F6", icon: "📁" }
      ];

      icons.forEach((ic, idx) => {
        const ix = 30;
        const iy = 40 + idx * 80;

        // Selection highlight if active
        if (activeWindow === ic.id) {
          ctx.fillStyle = "rgba(255, 255, 255, 0.15)";
          ctx.strokeStyle = "rgba(255, 255, 255, 0.25)";
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.roundRect(ix - 20, iy - 10, 80, 70, 8);
          ctx.fill();
          ctx.stroke();
        }

        ctx.font = "24px Inter";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(ic.icon, ix + 20, iy + 15);

        ctx.font = "11px Inter";
        ctx.fillStyle = "#FFFFFF";
        ctx.fillText(ic.name, ix + 20, iy + 48);
      });

      // 3. Render the Active Draggable Application Window
      if (activeWindow) {
        const { x, y, w: winW, h: winH } = winPos;

        // Window shadow
        ctx.shadowColor = "rgba(0, 0, 0, 0.3)";
        ctx.shadowBlur = 20;
        ctx.shadowOffsetX = 0;
        ctx.shadowOffsetY = 8;

        // Window Frame (frosted acrylic glass)
        ctx.fillStyle = "rgba(255, 255, 255, 0.95)";
        ctx.beginPath();
        ctx.roundRect(x, y, winW, winH, 10);
        ctx.fill();

        ctx.shadowColor = "transparent"; // Reset shadow

        // Window Titlebar (light gray acrylic)
        ctx.fillStyle = "rgba(241, 245, 249, 0.9)";
        ctx.beginPath();
        ctx.roundRect(x, y, winW, 36, [10, 10, 0, 0]);
        ctx.fill();

        // Title text
        ctx.font = "bold 12px Inter";
        ctx.fillStyle = "#1E293B";
        ctx.textAlign = "left";
        ctx.textBaseline = "middle";
        let title = "Arc画图工具 v1.0";
        if (activeWindow === "notepad") title = "Arc备忘录记事本.txt";
        if (activeWindow === "explorer") title = `文件资源管理器 - ${currentFolder}`;
        ctx.fillText(title, x + 16, y + 18);

        // Window Close Button (red tint on hover)
        ctx.fillStyle = "rgba(239, 68, 68, 0.1)";
        ctx.beginPath();
        ctx.arc(x + winW - 20, y + 18, 8, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = "#EF4444";
        ctx.font = "9px Inter";
        ctx.textAlign = "center";
        ctx.fillText("×", x + winW - 20, y + 18);

        // Window Content Area
        if (activeWindow === "painter") {
          // Paint Canvas background
          ctx.fillStyle = "#F8FAFC";
          ctx.fillRect(x + 10, y + 46, winW - 20, winH - 96);

          // Render brush palette options inside Painter Window
          ctx.fillStyle = "rgba(226, 232, 240, 0.8)";
          ctx.fillRect(x + 10, y + winH - 44, winW - 20, 36);

          // Palette Labels
          ctx.font = "11px Inter";
          ctx.fillStyle = "#475569";
          ctx.textAlign = "left";
          ctx.fillText("画笔颜色：", x + 20, y + winH - 26);

          const colors = ["#3B82F6", "#EF4444", "#10B981", "#F59E0B", "#8B5CF6", "#000000"];
          colors.forEach((col, cidx) => {
            const cx = x + 110 + cidx * 30;
            const cy = y + winH - 26;

            ctx.fillStyle = col;
            ctx.beginPath();
            ctx.arc(cx, cy, 8, 0, Math.PI * 2);
            ctx.fill();

            if (paintColor === col) {
              ctx.strokeStyle = "#475569";
              ctx.lineWidth = 1.5;
              ctx.beginPath();
              ctx.arc(cx, cy, 11, 0, Math.PI * 2);
              ctx.stroke();
            }
          });

          // Draw the canvas drawings
          ctx.save();
          // Clip drawing to window viewport
          ctx.beginPath();
          ctx.rect(x + 10, y + 46, winW - 20, winH - 96);
          ctx.clip();

          drawingsRef.current.forEach((pt, pidx) => {
            // Translate drawings relative to the painter canvas area
            if (pt.drag && pidx > 0) {
              ctx.strokeStyle = pt.color;
              ctx.lineWidth = pt.size;
              ctx.lineCap = "round";
              ctx.lineJoin = "round";
              ctx.beginPath();
              ctx.moveTo(x + 10 + drawingsRef.current[pidx - 1].x, y + 46 + drawingsRef.current[pidx - 1].y);
              ctx.lineTo(x + 10 + pt.x, y + 46 + pt.y);
              ctx.stroke();
            } else {
              ctx.fillStyle = pt.color;
              ctx.beginPath();
              ctx.arc(x + 10 + pt.x, y + 46 + pt.y, pt.size / 2, 0, Math.PI * 2);
              ctx.fill();
            }
          });

          ctx.restore();
        } else if (activeWindow === "notepad") {
          // Notepad page
          ctx.fillStyle = "#FFFFFF";
          ctx.fillRect(x + 10, y + 46, winW - 20, winH - 56);

          ctx.fillStyle = "#1E293B";
          ctx.font = "12px JetBrains Mono";
          ctx.textAlign = "left";
          ctx.textBaseline = "top";

          // Multiline text wrapping
          const lines = notepadText.split("\n");
          lines.forEach((line, lidx) => {
            ctx.fillText(line, x + 20, y + 56 + lidx * 18);
          });

          // Render a pulsing keyboard cursor
          if (Math.floor(Date.now() / 500) % 2 === 0) {
            const lastLine = lines[lines.length - 1];
            const tw = ctx.measureText(lastLine).width;
            ctx.fillStyle = "#2563EB";
            ctx.fillRect(x + 20 + tw, y + 56 + (lines.length - 1) * 18, 2, 14);
          }
        } else if (activeWindow === "explorer") {
          // File Explorer columns layout
          // Sidebar (gray glass)
          ctx.fillStyle = "#F1F5F9";
          ctx.fillRect(x + 10, y + 46, 120, winH - 56);

          // Content area
          ctx.fillStyle = "#FFFFFF";
          ctx.fillRect(x + 130, y + 46, winW - 140, winH - 56);

          // Sidebar Navigation Link
          ctx.font = "bold 11px Inter";
          ctx.fillStyle = "#475569";
          ctx.textAlign = "left";
          ctx.fillText("⭐ 快速访问", x + 20, y + 66);

          const sidebarItems = ["根目录", "我的文档", "系统配置System64"];
          sidebarItems.forEach((folder, fidx) => {
            const sx = x + 24;
            const sy = y + 86 + fidx * 24;

            if (currentFolder === folder) {
              ctx.fillStyle = "rgba(59, 130, 246, 0.15)";
              ctx.beginPath();
              ctx.roundRect(sx - 10, sy - 6, 100, 20, 4);
              ctx.fill();
              ctx.fillStyle = "#2563EB";
            } else {
              ctx.fillStyle = "#1E293B";
            }

            ctx.font = "11px Inter";
            ctx.fillText(`📁 ${folder}`, sx, sy + 8);
          });

          // Main Folder Content Grid
          const currentFiles = files[currentFolder] || [];
          ctx.fillStyle = "#1E293B";
          ctx.font = "bold 11px Inter";
          ctx.fillText("名称", x + 146, y + 66);
          ctx.fillText("类型", x + 266, y + 66);
          ctx.fillText("大小", x + 346, y + 66);

          ctx.strokeStyle = "#E2E8F0";
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(x + 140, y + 76);
          ctx.lineTo(x + winW - 20, y + 76);
          ctx.stroke();

          currentFiles.forEach((file, fidx) => {
            const fx = x + 146;
            const fy = y + 90 + fidx * 24;

            ctx.font = "11px Inter";
            ctx.fillStyle = "#1E293B";
            ctx.fillText(file.type === "folder" ? `📁 ${file.name}` : `📄 ${file.name}`, fx, fy);
            ctx.fillStyle = "#64748B";
            ctx.fillText(file.type === "folder" ? "文件夹" : "文档与配置文件", x + 266, fy);
            ctx.fillText(file.size || "--", x + 346, fy);
          });
        }
      }

      // 4. Windows 11 Taskbar
      ctx.fillStyle = "rgba(15, 23, 42, 0.92)";
      ctx.fillRect(0, h - 36, w, 36);

      // Taskbar Center Icons
      const startBtnX = w / 2 - 40;
      ctx.font = "16px Inter";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText("🔷", startBtnX, h - 18); // Start Logo
      ctx.fillText("🎨", startBtnX + 24, h - 18); // Paint Shortcut
      ctx.fillText("📝", startBtnX + 48, h - 18); // Notepad Shortcut
      ctx.fillText("📁", startBtnX + 72, h - 18); // Explorer Shortcut

      // Draw active indicators under icons
      ctx.fillStyle = "#3B82F6";
      if (activeWindow === "painter") ctx.fillRect(startBtnX + 16, h - 3, 16, 2);
      if (activeWindow === "notepad") ctx.fillRect(startBtnX + 40, h - 3, 16, 2);
      if (activeWindow === "explorer") ctx.fillRect(startBtnX + 64, h - 3, 16, 2);

      // Clock and system tray on right corner
      ctx.font = "11px Inter";
      ctx.fillStyle = "#94A3B8";
      ctx.textAlign = "right";
      const now = new Date();
      const timeStr = now.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
      const dateStr = now.toLocaleDateString([], { month: "short", day: "numeric" });
      ctx.fillText(timeStr, w - 16, h - 23);
      ctx.fillText(dateStr, w - 16, h - 10);

      // 5. Draw Mapped Physical Cursor Dot (Visual Confirmation)
      ctx.shadowColor = "rgba(0,0,0,0.4)";
      ctx.shadowBlur = 8;
      ctx.fillStyle = "#FFFFFF";
      ctx.strokeStyle = "#000000";
      ctx.lineWidth = 1.5;

      ctx.beginPath();
      // Draw cursor arrow path
      ctx.moveTo(cursorX, cursorY);
      ctx.lineTo(cursorX + 10, cursorY + 14);
      ctx.lineTo(cursorX + 4, cursorY + 14);
      ctx.lineTo(cursorX, cursorY + 20);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();

      ctx.shadowColor = "transparent"; // Reset

      // 6. Frame Callback for remote stream emulation
      if (onFrameCapture) {
        onFrameCapture(canvas.toDataURL("image/jpeg", 0.75));
      }

      animFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animFrameId);
    };
  }, [activeWindow, winPos, paintColor, brushSize, notepadText, currentFolder, cursorX, cursorY, drawingsRef.current]);

  // Handle Incoming Keyboard Inputs and Mouse Move events from the Web Controller
  useEffect(() => {
    if (!externalInput) return;

    const { type, x, y, key, isDown, button } = externalInput;

    // Convert normalized coordinates (0.0 - 1.0) to local resolution (1920x1080)
    const localX = Math.round(x * 960);
    const localY = Math.round(y * 540);

    setCursorX(localX);
    setCursorY(localY);

    if (type === "mousemove") {
      // If pointer is dragging inside Painter Window Canvas area, draw!
      if (isMouseDown && activeWindow === "painter") {
        const px = localX - (winPos.x + 10);
        const py = localY - (winPos.y + 46);
        const innerW = winPos.w - 20;
        const innerH = winPos.h - 96;

        if (px >= 0 && px <= innerW && py >= 0 && py <= innerH) {
          drawingsRef.current.push({
            x: px,
            y: py,
            color: paintColor,
            size: brushSize,
            drag: true,
          });
        }
      }
    }

    if (type === "mousedown") {
      setIsMouseDown(true);

      // 1. Check if clicking Window Close Button
      const closeX = winPos.x + winPos.w - 20;
      const closeY = winPos.y + 18;
      const distClose = Math.hypot(localX - closeX, localY - closeY);
      if (distClose < 12) {
        setActiveWindow(null);
        return;
      }

      // 2. Check Palette clicks inside painter
      if (activeWindow === "painter") {
        const paletteY = winPos.y + winPos.h - 26;
        const colors = ["#3B82F6", "#EF4444", "#10B981", "#F59E0B", "#8B5CF6", "#000000"];
        colors.forEach((col, cidx) => {
          const cx = winPos.x + 110 + cidx * 30;
          if (Math.hypot(localX - cx, localY - paletteY) < 10) {
            setPaintColor(col);
          }
        });
      }

      // 3. Check File Explorer sidebar clicks or folder clicks
      if (activeWindow === "explorer") {
        // Sidebar folder selection
        const folders = ["根目录", "我的文档", "系统配置System64"];
        folders.forEach((f, fidx) => {
          const fy = winPos.y + 86 + fidx * 24;
          if (localX >= winPos.x + 14 && localX <= winPos.x + 114 && localY >= fy - 4 && localY <= fy + 16) {
            setCurrentFolder(f);
          }
        });

        // Content double click simulation or list click
        const currentFiles = files[currentFolder] || [];
        currentFiles.forEach((file, fidx) => {
          const fy = winPos.y + 90 + fidx * 24;
          if (localX >= winPos.x + 140 && localX <= winPos.x + winPos.w - 20 && localY >= fy - 10 && localY <= fy + 12) {
            if (file.type === "folder") {
              setCurrentFolder(file.name);
            }
          }
        });
      }

      // 4. Check Desktop icon clicks
      const icons = [
        { id: "painter", idx: 0 },
        { id: "notepad", idx: 1 },
        { id: "explorer", idx: 2 }
      ];
      icons.forEach((ic) => {
        const ix = 30;
        const iy = 40 + ic.idx * 80;
        if (localX >= ix - 20 && localX <= ix + 60 && localY >= iy - 10 && localY <= iy + 60) {
          setActiveWindow(ic.id);
        }
      });

      // 5. Check Taskbar shortcut clicks
      const startBtnX = 960 / 2 - 40;
      if (localY >= 540 - 36 && localY <= 540) {
        if (localX >= startBtnX + 12 && localX <= startBtnX + 36) setActiveWindow("painter");
        if (localX >= startBtnX + 36 && localX <= startBtnX + 60) setActiveWindow("notepad");
        if (localX >= startBtnX + 60 && localX <= startBtnX + 84) setActiveWindow("explorer");
      }

      // If clicked inside painter Canvas, register initial dot
      if (activeWindow === "painter") {
        const px = localX - (winPos.x + 10);
        const py = localY - (winPos.y + 46);
        const innerW = winPos.w - 20;
        const innerH = winPos.h - 96;

        if (px >= 0 && px <= innerW && py >= 0 && py <= innerH) {
          drawingsRef.current.push({
            x: px,
            y: py,
            color: paintColor,
            size: brushSize,
            drag: false,
          });
        }
      }
    }

    if (type === "mouseup") {
      setIsMouseDown(false);
    }

    // Handle character typing inside notepad text file
    if (type === "keydown" && activeWindow === "notepad" && isDown) {
      if (key === "Backspace") {
        setNotepadText((prev) => prev.slice(0, -1));
      } else if (key === "Enter") {
        setNotepadText((prev) => prev + "\n");
      } else if (key && key.length === 1) {
        setNotepadText((prev) => prev + key);
      }
    }

  }, [externalInput]);

  return (
    <div className="relative border border-white/50 rounded-xl overflow-hidden shadow-sm aspect-video bg-black max-w-full">
      <canvas
        ref={canvasRef}
        width={960}
        height={540}
        className="w-full h-full block object-contain"
      />
    </div>
  );
}
