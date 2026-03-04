/* ═════════════════════════════════════════════════════════
   JeebsAI Chat Logic Module for Dashboard
   ═════════════════════════════════════════════════════════ */

const chatState = {
    loggedIn: false,
    username: "",
    isAdmin: false,
    isTrainer: false,
    token: ""
};

const JEEBS_ROOT_ADMIN = "1090mb";

let chatFeed;
let promptInput;
let chatPanel;
let chatMeta;
let authViews;

// ── Initialize Elements ──────────────────────────────────
function initChatElements() {
    chatFeed = document.getElementById("chatFeed");
    promptInput = document.getElementById("promptInput");
    chatPanel = document.getElementById("chatPanel");
    chatMeta = document.getElementById("chatMeta");
    authViews = document.getElementById("authViews");
}

// ── Helper Functions ──────────────────────────────────────
function isRootAdminSession() {
    return chatState.isAdmin && chatState.username === JEEBS_ROOT_ADMIN;
}

function setStatus(msg, type) {
    let toast = document.getElementById("statusToast");
    if (!toast) {
        toast = document.createElement("div");
        toast.id = "statusToast";
        toast.style.cssText = "position:fixed;bottom:60px;left:50%;transform:translateX(-50%);padding:10px 20px;border-radius:8px;font-size:0.9rem;z-index:10000;transition:opacity 0.3s;pointer-events:none;";
        document.body.appendChild(toast);
    }
    toast.textContent = msg;
    toast.style.background = type === "good" ? "rgba(80,200,120,0.9)" : "rgba(220,80,80,0.9)";
    toast.style.color = "#fff";
    toast.style.opacity = "1";
    clearTimeout(toast._timer);
    toast._timer = setTimeout(() => { toast.style.opacity = "0"; }, 3000);
}

function setView(view) {
    document.querySelectorAll("#authViews .view").forEach((node) => {
        node.classList.toggle("active", node.id === `${view}View`);
    });
}

function renderAuthState() {
    if (authViews && authViews.style.display === "none") {
        chatPanel.style.display = "flex";
        chatMeta.textContent = !chatState.loggedIn
            ? "Guest User (Unregistered)"
            : `${chatState.username}${isRootAdminSession() ? " (super-admin)" : chatState.isAdmin ? " (admin)" : ""}`;
        promptInput.focus();

        const adminColumn = document.getElementById("adminColumn");
        if (adminColumn) {
            adminColumn.style.display = isRootAdminSession() ? "flex" : "none";
        }
    }
}

function updateUnsignedMessage() {
    const elem = document.getElementById("loginUsername");
    if (!elem) return;
    const username = elem.value.trim();
    const ts = Math.floor(Date.now() / 1000);
    const value = `LOGIN:${username || "username"}:${ts}`;
    const unsignedElem = document.getElementById("unsignedMessage");
    if (unsignedElem) unsignedElem.value = value;
}

// ── Message Handling ──────────────────────────────────────
function appendMessage(who, text, thought = null) {
    const bubble = document.createElement("div");
    bubble.className = `bubble ${who}`;

    if (who === "jeebs" && thought) {
        const details = document.createElement("details");
        details.className = "thought-process";
        const summary = document.createElement("summary");
        summary.className = "thought-summary";
        summary.innerHTML = "<span>🧠 Thought Process</span>";
        const content = document.createElement("div");
        content.className = "thought-content";
        content.textContent = thought.internal_monologue || "";
        details.appendChild(summary);
        details.appendChild(content);
        bubble.appendChild(details);
    }

    const label = document.createElement("small");
    label.textContent = who === "user" ? "You" : "JeebsAI";
    const body = document.createElement("div");
    body.textContent = text;
    bubble.append(label, body);

    chatFeed.appendChild(bubble);
    chatFeed.scrollTop = chatFeed.scrollHeight;
}

function showTyping() {
    const indicator = document.createElement("div");
    indicator.className = "typing-indicator";
    indicator.id = "typingIndicator";
    indicator.innerHTML = '<span>JeebsAI is typing</span><div class="dot"></div><div class="dot"></div><div class="dot"></div>';
    chatFeed.appendChild(indicator);
    chatFeed.scrollTop = chatFeed.scrollHeight;
}

function hideTyping() {
    const indicator = document.getElementById("typingIndicator");
    if (indicator) indicator.remove();
}

// ── API Calls ────────────────────────────────────────────
async function requestJson(url, options = {}) {
    const headers = new Headers(options.headers || {});
    if (options.body && !headers.has("Content-Type")) {
        headers.set("Content-Type", "application/json");
    }
    if (chatState.token && !headers.has("Authorization")) {
        headers.set("Authorization", `Bearer ${chatState.token}`);
    }

    const response = await safeFetch(url, {
        ...options,
        headers,
        credentials: "same-origin",
    });

    let payload = {};
    try {
        payload = await response.json();
    } catch {
        payload = {};
    }

    if (!response.ok) {
        throw new Error(payload.error || `Request failed (${response.status})`);
    }

    return payload;
}

async function bootstrapAuth() {
    try {
        const data = await requestJson("/api/auth/status");
        chatState.loggedIn = Boolean(data.logged_in);
        chatState.username = data.username || "";
        chatState.isAdmin = Boolean(data.is_admin);
        chatState.isTrainer = Boolean(data.is_trainer);
        if (data.token) {
            chatState.token = data.token;
            if (typeof jeebsSetToken === 'function') jeebsSetToken(chatState.token);
        }
        renderAuthState();
        if (chatState.loggedIn) {
            let greeting = `Welcome, ${chatState.username}! `;
            if (chatState.username === "1090mb") {
                greeting += "🔐 You have super admin access. ";
            }
            greeting += "Ask me anything.";
            appendMessage("jeebs", greeting);
        }
    } catch (error) {
        chatState.loggedIn = false;
        renderAuthState();
    }
}

// ── Event Listeners ──────────────────────────────────────
function attachChatEventListeners() {
    const sendBtn = document.getElementById("chatForm");
    if (!sendBtn) return;

    sendBtn.addEventListener("submit", async (event) => {
        event.preventDefault();

        const prompt = promptInput.value.trim();
        if (!prompt) return;

        appendMessage("user", prompt);
        promptInput.value = "";
        showTyping();

        try {
            const data = await requestJson("/api/jeebs", {
                method: "POST",
                body: JSON.stringify({ prompt: prompt }),
            });
            hideTyping();
            appendMessage("jeebs", data.response || "(No response)", data.thought);
        } catch (error) {
            hideTyping();
            appendMessage("jeebs", `Error: ${error.message}`);
        }
    });

    promptInput.addEventListener("keydown", (event) => {
        if ((event.shiftKey || event.ctrlKey) && event.key === "Enter") {
            event.preventDefault();
            document.getElementById("chatForm").dispatchEvent(new Event('submit'));
        }
    });

    const logoutBtn = document.getElementById("logoutBtn");
    if (logoutBtn) {
        logoutBtn.addEventListener("click", async () => {
            try {
                await requestJson("/api/logout", { method: "POST" });
            } catch { }
            chatState.loggedIn = false;
            chatState.username = "";
            chatFeed.innerHTML = "";
            setStatus("Logged out.", "good");
            location.reload();
        });
    }
}

// ── Initialize ──────────────────────────────────────────
async function initializeChat() {
    initChatElements();
    updateUnsignedMessage();
    attachChatEventListeners();
    await bootstrapAuth();
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initializeChat);
} else {
    initializeChat();
}
