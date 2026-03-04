/* ═════════════════════════════════════════════════════════
   JeebsAI Admin Unified Panel - Dashboard Mini Panel
   ═════════════════════════════════════════════════════════ */

const AdminUnified = (() => {
    let updateInterval = null;
    let activityLog = [];

    async function fetchStatus() {
        try {
            const res = await safeFetch('/api/admin/status', {
                credentials: 'same-origin',
                headers: typeof authHeaders === 'function' ? authHeaders() : {}
            });
            if (!res.ok) return null;
            return await res.json();
        } catch (e) {
            return null;
        }
    }

    async function fetchSessions() {
        try {
            const res = await safeFetch('/api/admin/sessions', {
                credentials: 'same-origin',
                headers: typeof authHeaders === 'function' ? authHeaders() : {}
            });
            if (!res.ok) return [];
            return await res.json();
        } catch (e) {
            return [];
        }
    }

    async function fetchInternet() {
        try {
            const res = await safeFetch('/api/admin/internet/status', {
                credentials: 'same-origin',
                headers: typeof authHeaders === 'function' ? authHeaders() : {}
            });
            if (!res.ok) return { enabled: false };
            return await res.json();
        } catch (e) {
            return { enabled: false };
        }
    }

    async function fetchTraining() {
        try {
            const res = await safeFetch('/api/admin/training/status', {
                credentials: 'same-origin',
                headers: typeof authHeaders === 'function' ? authHeaders() : {}
            });
            if (!res.ok) return { training: { enabled: false } };
            return await res.json();
        } catch (e) {
            return { training: { enabled: false } };
        }
    }

    async function toggleFeature(feature, enabled) {
        try {
            const endpoint = feature === 'internet'
                ? '/api/admin/internet/set'
                : '/api/admin/training/mode';

            const res = await safeFetch(endpoint, {
                method: 'POST',
                credentials: 'same-origin',
                headers: {
                    'Content-Type': 'application/json',
                    ...(typeof authHeaders === 'function' ? authHeaders(true) : {})
                },
                body: JSON.stringify({ enabled })
            });
            return res.ok;
        } catch (e) {
            return false;
        }
    }

    function updateUI(status, sessions, internet, training) {
        // Update server status
        const srvStatus = document.getElementById('srvStatusMini');
        if (srvStatus) {
            srvStatus.textContent = status && status.uptime ? '🟢' : '🔴';
        }

        // Update session count
        const sessionCount = document.getElementById('sessionCountMini');
        if (sessionCount) {
            sessionCount.textContent = sessions ? sessions.length : '0';
        }

        // Update toggles
        const internetToggle = document.getElementById('internetToggleMini');
        if (internetToggle) {
            internetToggle.checked = internet.enabled;
            const dot = document.querySelector('.toggle-item:nth-child(1) .status-dot');
            if (dot) {
                dot.className = internet.enabled ? 'status-dot on' : 'status-dot off';
            }
        }

        const trainingToggle = document.getElementById('trainingToggleMini');
        if (trainingToggle && training.training) {
            trainingToggle.checked = training.training.enabled;
            const dot = document.querySelector('.toggle-item:nth-child(2) .status-dot');
            if (dot) {
                dot.className = training.training.enabled ? 'status-dot on' : 'status-dot off';
            }
        }

        // Update activity feed
        updateActivityFeed(status, sessions);
    }

    function updateActivityFeed(status, sessions) {
        const feedMini = document.getElementById('activityFeedMini');
        if (!feedMini) return;

        let html = '';

        if (sessions && sessions.length > 0) {
            html += `<div class="activity-item"><strong>${sessions.length} active session${sessions.length > 1 ? 's' : ''}</strong></div>`;
            sessions.slice(0, 2).forEach(s => {
                html += `<div class="activity-item" style="font-size:0.7rem;color:var(--muted);">${s.username}</div>`;
            });
        }

        if (status && status.uptime) {
            html += `<div class="activity-item" style="font-size:0.7rem;">Uptime: ${status.uptime}</div>`;
        }

        if (!html) {
            html = '<div class="activity-item text-muted">No activity</div>';
        }

        feedMini.innerHTML = html;
    }

    async function refreshAdmin() {
        const status = await fetchStatus();
        const sessions = await fetchSessions();
        const internet = await fetchInternet();
        const training = await fetchTraining();
        updateUI(status, sessions, internet, training);
    }

    return {
        async init() {
            // Only initialize if user is admin
            const adminColumn = document.getElementById('adminColumn');
            if (!adminColumn || typeof chatState === 'undefined' || !chatState.isAdmin) {
                return;
            }

            // Attach toggle listeners
            window.onInternetToggleMini = async function() {
                const toggle = document.getElementById('internetToggleMini');
                toggle.disabled = true;
                const success = await toggleFeature('internet', toggle.checked);
                toggle.disabled = false;
                if (!success) toggle.checked = !toggle.checked;
                await refreshAdmin();
            };

            window.onTrainingToggleMini = async function() {
                const toggle = document.getElementById('trainingToggleMini');
                toggle.disabled = true;
                const success = await toggleFeature('training', toggle.checked);
                toggle.disabled = false;
                if (!success) toggle.checked = !toggle.checked;
                await refreshAdmin();
            };

            // Initial load and set interval
            await refreshAdmin();
            updateInterval = setInterval(refreshAdmin, 10000); // Update every 10s
        },

        destroy() {
            if (updateInterval) {
                clearInterval(updateInterval);
            }
        }
    };
})();

// Initialize when page loads
if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
        setTimeout(() => AdminUnified.init(), 500);
    });
} else {
    setTimeout(() => AdminUnified.init(), 500);
}
