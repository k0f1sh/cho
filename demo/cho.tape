Output cho-demo.gif

Set Shell "bash"
Set FontSize 28
Set Width 1200
Set Height 600
Set Padding 30
Set WindowBar Colorful
Set TypingSpeed 45ms
Set Theme { "name": "cho", "black": "#1e1e2e", "red": "#f38ba8", "green": "#a6e3a1", "yellow": "#f9e2af", "blue": "#89b4fa", "magenta": "#cba6f7", "cyan": "#94e2d5", "white": "#cdd6f4", "brightBlack": "#585b70", "brightRed": "#f38ba8", "brightGreen": "#a6e3a1", "brightYellow": "#f9e2af", "brightBlue": "#89b4fa", "brightMagenta": "#cba6f7", "brightCyan": "#94e2d5", "brightWhite": "#ffffff", "background": "#1e1e2e", "foreground": "#cdd6f4", "selectionBackground": "#45475a", "cursorColor": "#f5e0dc" }

Hide
Type `export PATH=/vhs/target/debug:$PATH`
Enter
Type "cd demo"
Enter
Type "clear"
Enter
Show

Type@25ms "# Backup files: name size created_at"
Enter
Sleep 800ms
Type "cat files.txt"
Sleep 400ms
Enter
Sleep 3s

Ctrl+L
Type@25ms "# Find backups larger than 1 GB"
Enter
Sleep 800ms
Type `cho '(f (bs/>= $2 "1GB"))' < files.txt`
Sleep 500ms
Enter
Sleep 3s

Ctrl+L
Type@25ms "# Calculate expiration dates: created_at + 7 days"
Enter
Sleep 800ms
Type `cho '(p $1 (dt/add $3 (du/d 7)))' < files.txt`
Sleep 500ms
Enter
Sleep 3s

Ctrl+L
Type@25ms "# Build a retention schedule for large backups"
Enter
Sleep 800ms
Type `cho '(f (bs/>= $2 "1GB"))`
Enter
Type `(p (s/join " | "`
Enter
Type `  (-> $1 (s/replace-all "-" "_") s/upper) $2`
Enter
Type `  (dt/fmt (dt/add $3 (du/d 7)) "%Y-%m-%d %H:%M UTC")))`
Enter
Type `' < files.txt`
Sleep 500ms
Enter
Sleep 5s
