// API Module - All API communication
// Handles requests to /api/jeebs and other endpoints

const API = (() => {
    const BASE_URL = '/api';

    return {
        // Chat endpoint
        async sendMessage(message) {
            try {
                const response = await fetch(`${BASE_URL}/jeebs`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ prompt: message })
                });

                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                return await response.json();
            } catch (error) {
                console.error('API Error (sendMessage):', error);
                throw error;
            }
        },

        // Chat history
        async getHistory(limit = 50) {
            try {
                const response = await fetch(`${BASE_URL}/chat/history?limit=${limit}`);
                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                return await response.json();
            } catch (error) {
                console.error('API Error (getHistory):', error);
                return [];
            }
        },

        // Brain status
        async getBrainStatus() {
            try {
                const response = await fetch(`${BASE_URL}/brain/status`);
                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                return await response.json();
            } catch (error) {
                console.error('API Error (getBrainStatus):', error);
                return { status: 'unknown' };
            }
        },

        // Mood/summary
        async getMood() {
            try {
                const response = await fetch(`${BASE_URL}/jeebs_mood`);
                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                return await response.text();
            } catch (error) {
                console.error('API Error (getMood):', error);
                return 'contemplative';
            }
        },

        // Login
        async login(username, password) {
            try {
                const response = await fetch(`${BASE_URL}/login`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ username, password })
                });
                if (!response.ok) throw new Error(`HTTP ${response.status}`);
                return await response.json();
            } catch (error) {
                console.error('API Error (login):', error);
                throw error;
            }
        },

        // Logout
        async logout() {
            try {
                await fetch(`${BASE_URL}/logout`, { method: 'POST' });
            } catch (error) {
                console.error('API Error (logout):', error);
            }
        }
    };
})();
