import os
import re

html_files = [f for f in os.listdir("webui") if f.endswith(".html") and not f.startswith("._") and not f.startswith(".DS")]

for filename in html_files:
    path = os.path.join("webui", filename)
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
        
    original = content
        
    def nav_replace(match):
        attrs = match.group(1) if match.group(1) else ""
        return f'<div id="app-layout">\n  <aside id="sidebar-container"{attrs}></aside>\n  <main class="main-content">'
        
    content = re.sub(r'<nav\s+id="topnav"([^>]*)></nav>', nav_replace, content)
    content = re.sub(r'(class="shell"[^>]*)style="padding-top:\s*60px;?\s*', r'\1style="', content)
    content = re.sub(r'(class="shell"[^>]*)style="padding-top:\s*80px;?\s*', r'\1style="', content)
    
    if '<div id="app-layout">' in content and not '</main>' in content:
        content = content.replace('</body>', '  </main>\n  </div>\n</body>')
        
    if original != content:
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Updated {filename}")
