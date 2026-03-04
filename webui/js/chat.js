// Chat Module - Chat message handling and display
// Manages conversation flow and message rendering

const Chat = (() => {
    let messageCount = 0;
    let currentTopic = 'General';

    return {
        init() {
            const sendBtn = document.getElementById('sendBtn');
            const userInput = document.getElementById('userInput');

            sendBtn.addEventListener('click', () => this.sendMessage());
            userInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    this.sendMessage();
                }
            });
        },

        async sendMessage() {
            const userInput = document.getElementById('userInput');
            const message = userInput.value.trim();

            if (!message) return;

            // Display user message immediately
            this.addMessage(message, 'user');
            userInput.value = '';
            userInput.focus();

            try {
                // Show "thinking..." indicator
                this.showThinking();

                const response = await API.sendMessage(message);
                this.removeThinking();

                // Display AI response
                this.addMessage(response.response || response, 'assistant');
                messageCount++;
                this.updateMessageCount();
            } catch (error) {
                this.removeThinking();
                this.addMessage('Sorry, I encountered an error. Please try again.', 'assistant');
                console.error('Chat error:', error);
            }
        },

        addMessage(content, role) {
            const messagesContainer = document.getElementById('messages');
            const messageEl = document.createElement('div');
            messageEl.className = `message message-${role}`;
            messageEl.innerHTML = `<div class="message-content">${this.escapeHtml(content)}</div>`;
            messagesContainer.appendChild(messageEl);
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        },

        showThinking() {
            const messagesContainer = document.getElementById('messages');
            const thinkingEl = document.createElement('div');
            thinkingEl.id = 'thinkingIndicator';
            thinkingEl.className = 'message message-thinking';
            thinkingEl.innerHTML = '<div class="message-content"><span class="thinking-dots">Thinking</span></div>';
            messagesContainer.appendChild(thinkingEl);
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        },

        removeThinking() {
            const thinkingEl = document.getElementById('thinkingIndicator');
            if (thinkingEl) thinkingEl.remove();
        },

        updateMessageCount() {
            document.getElementById('messageCount').textContent = messageCount;
        },

        setTopic(topic) {
            currentTopic = topic || 'General';
            document.getElementById('currentTopic').textContent = currentTopic;
            document.getElementById('headerTopic').textContent = `Topic: ${currentTopic}`;
        },

        escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        },

        clear() {
            document.getElementById('messages').innerHTML = '';
            messageCount = 0;
            this.updateMessageCount();
        }
    };
})();
