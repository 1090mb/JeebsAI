// UI Module - UI updates and visual management
// Handles status bar, brain status, and overall UI state

const UI = (() => {
    return {
        init() {
            this.updateBrainStatus();
            this.updateStatusBar();
            // Refresh status every 10 seconds
            setInterval(() => this.updateBrainStatus(), 10000);
        },

        async updateBrainStatus() {
            try {
                const mood = await API.getMood();
                const statusIndicator = document.querySelector('.status-indicator');
                const statusText = document.querySelector('.status-text');

                statusIndicator.textContent = '●';
                statusText.textContent = 'Awake';

                document.getElementById('statusCenter').textContent = `Mood: ${mood}`;
            } catch (error) {
                console.error('Status update error:', error);
            }
        },

        updateStatusBar() {
            const now = new Date();
            const time = now.toLocaleTimeString();
            document.getElementById('statusLeft').textContent = time;
        },

        showNotification(message, type = 'info') {
            // Simple notification system
            console.log(`[${type.toUpperCase()}] ${message}`);
            // Could be enhanced with toast notifications
        },

        enableChat() {
            document.getElementById('userInput').disabled = false;
            document.getElementById('sendBtn').disabled = false;
        },

        disableChat() {
            document.getElementById('userInput').disabled = true;
            document.getElementById('sendBtn').disabled = true;
        },

        showLoading() {
            Chat.showThinking();
        },

        hideLoading() {
            Chat.removeThinking();
        }
    };
})();

// Initialize app
document.addEventListener('DOMContentLoaded', () => {
    // Try anonymous first (for /api/jeebs support)
    // If auth is needed, Auth module will show login view
    Auth.init();
    UI.init();

    // Allow immediate chat if not authed (anonymous mode)
    const chatLayout = document.querySelector('.app-main > .chat-layout');
    if (chatLayout && chatLayout.style.display !== 'none') {
        Chat.init();
    }
});
