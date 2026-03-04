// Auth Module - Authentication handling
// Manages login, logout, and session state

const Auth = (() => {
    let isLoggedIn = false;
    let currentUser = null;

    return {
        init() {
            const accountBtn = document.getElementById('accountBtn');
            const accountDropdown = document.getElementById('accountDropdown');
            const logoutBtn = document.getElementById('logoutBtn');
            const loginForm = document.getElementById('loginForm');

            // Account menu toggle
            accountBtn.addEventListener('click', () => {
                accountDropdown.style.display =
                    accountDropdown.style.display === 'none' ? 'block' : 'none';
            });

            // Logout
            logoutBtn.addEventListener('click', async () => {
                await this.logout();
            });

            // Login form
            if (loginForm) {
                loginForm.addEventListener('submit', async (e) => {
                    e.preventDefault();
                    const username = document.getElementById('username').value;
                    await this.login(username);
                });
            }

            // Check initial auth status
            this.checkStatus();
        },

        async checkStatus() {
            // Check if already logged in via session
            try {
                const response = await fetch('/api/auth/status');
                if (response.ok) {
                    const data = await response.json();
                    if (data.username) {
                        this.setLoggedIn(data.username);
                    }
                }
            } catch (error) {
                console.log('Not authenticated');
            }
        },

        async login(username) {
            try {
                // For now, use simple authentication
                // In production, this would handle PGP authentication
                const response = await fetch('/api/login', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ username, password: 'demo' })
                });

                if (response.ok) {
                    this.setLoggedIn(username);
                } else {
                    alert('Login failed');
                }
            } catch (error) {
                console.error('Login error:', error);
                alert('Login error. Check console.');
            }
        },

        async logout() {
            try {
                await API.logout();
                this.setLoggedOut();
            } catch (error) {
                console.error('Logout error:', error);
            }
        },

        setLoggedIn(username) {
            isLoggedIn = true;
            currentUser = username;
            document.getElementById('authViews').style.display = 'none';
            document.querySelector('.app-main > .chat-layout').style.display = 'flex';
            document.getElementById('accountBtn').textContent = username;
            Chat.init();
        },

        setLoggedOut() {
            isLoggedIn = false;
            currentUser = null;
            document.getElementById('authViews').style.display = 'flex';
            document.querySelector('.app-main > .chat-layout').style.display = 'none';
            document.getElementById('accountBtn').textContent = 'Account';
            Chat.clear();
        },

        isAuthenticated() {
            return isLoggedIn;
        },

        getUsername() {
            return currentUser;
        }
    };
})();
