# Web Server Deployment Checklist (Single-Folder, New UI)

1. Copy all updated files to the VPS web root (overwrite old files):
   - index.html (should be the new, clean UI)
   - style.css (should be the new dark blue/clean design)
   - All JS files (auth.js, chat.js, api.js, ui.js, etc.)
   - Any new images or assets

2. Remove any old files or folders (like webui/ or src/) from the web root on the VPS.

3. Set correct permissions (if needed):
   sudo chown -R www-data:www-data /path/to/webroot
   sudo chmod -R 755 /path/to/webroot

4. Restart nginx (or your web server):
   sudo systemctl restart nginx

5. Hard refresh your browser (Ctrl+Shift+R or Cmd+Shift+R) to clear cache.

6. Verify:
   - The site loads with the new dark blue, clean UI
   - All features (login, chat, admin) work as expected
   - Admin pages are protected as required

---

# Example Deployment Script (run on VPS)

# Copy new files from your local machine or repo (adjust paths as needed)
scp index.html style.css auth.js chat.js api.js ui.js user@your-vps:/path/to/webroot/

# Remove old folders if they exist
ssh user@your-vps 'rm -rf /path/to/webroot/webui /path/to/webroot/src'

# Set permissions
ssh user@your-vps 'sudo chown -R www-data:www-data /path/to/webroot && sudo chmod -R 755 /path/to/webroot'

# Restart nginx
ssh user@your-vps 'sudo systemctl restart nginx'

# Done! Visit your site and verify the new UI is live.
